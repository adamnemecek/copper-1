use std::collections::HashMap;
use crate::display::{
    Display,
    WallClock,
    framebuffers::{
        FboMap,
    }
};
use crate::gl;
use crate::entities::*;
use crate::math::{
    Matrix4f,
    Vector3f,
    Vector4f,
};
use crate::models::{
    TexturedModel,
};
use crate::particles::ParticleMaster;
use super::shadowmap_renderer::ShadowMapRenderer;
use super::entity_renderer::EntityRenderer;
use super::normal_map_entity_renderer::NormalMapEntityRenderer;
use super::terrain_renderer::TerrainRenderer;
use super::skybox_renderer::SkyboxRenderer;
use super::water_renderer::WaterRenderer;
use super::debug_renderer::DebugRenderer;
use super::env_map_renderer::EnvMapRenderer;
use super::animated_entity_renderer::AnimatedEntityRenderer;

pub struct RenderGroup {
    pub id: u32,
    pub name: &'static str,
}

impl RenderGroup {    
    pub const SHADOW_MAP_PASS: RenderGroup = RenderGroup {id: 0, name: "ShadowMapPass"};
    pub const REFLECT_REFRACT_PASS: RenderGroup = RenderGroup {id: 1, name: "ReflectRefractPass"};
    pub const DRAW_ENTITIES: RenderGroup = RenderGroup {id: 2, name: "EntityDrawPass"};
    pub const DRAW_NORMAL_MAP_ENTITIES: RenderGroup = RenderGroup {id: 3, name: "NormalMapEntityDrawPass"};
    pub const DRAW_TERRAIN: RenderGroup = RenderGroup {id: 4, name: "TerrainDraw"};
    pub const DRAW_SKYBOX: RenderGroup = RenderGroup {id: 5, name: "Skybox"};
    pub const DRAW_WATER: RenderGroup = RenderGroup {id: 6, name: "WaterSurfaceDraw"};
    pub const PARTICLE_EFFECTS_PASS: RenderGroup = RenderGroup {id: 7, name: "ParticleEffects"};
    pub const POST_PROCESSING: RenderGroup = RenderGroup {id: 8, name: "PostProcessing"};
    pub const DRAW_GUI: RenderGroup = RenderGroup {id: 9, name: "GuiOverlayDraw"};
}

pub struct MasterRenderer {    
    entity_renderer: EntityRenderer,
    normal_map_entity_renderer: NormalMapEntityRenderer,
    terrain_renderer: TerrainRenderer,
    skybox_renderer: SkyboxRenderer,
    water_renderer: WaterRenderer,
    shadowmap_renderer: ShadowMapRenderer,
    env_map_renderer: EnvMapRenderer,
    animated_entity_renderer: AnimatedEntityRenderer,
}

impl MasterRenderer {

    const SKY_COLOR: Vector3f = Vector3f{ x: 0.5444, y: 0.62, z: 0.69 };

    pub fn new(projection_matrix: &Matrix4f, aspect_ratio: f32) -> MasterRenderer {
        let entity_renderer = EntityRenderer::new(projection_matrix);
        let normal_map_entity_renderer = NormalMapEntityRenderer::new(projection_matrix);
        let terrain_renderer = TerrainRenderer::new(projection_matrix);
        let skybox_renderer = SkyboxRenderer::new(projection_matrix);
        let water_renderer = WaterRenderer::new(projection_matrix, &MasterRenderer::SKY_COLOR);
        let shadowmap_renderer = ShadowMapRenderer::new(aspect_ratio);
        let _debug_renderer = DebugRenderer::new(projection_matrix);
        let env_map_renderer = EnvMapRenderer::new(projection_matrix);
        let animated_entity_renderer = AnimatedEntityRenderer::new(projection_matrix);

        MasterRenderer {
            entity_renderer,
            normal_map_entity_renderer,
            terrain_renderer,
            skybox_renderer,
            water_renderer,
            shadowmap_renderer,
            env_map_renderer,
            animated_entity_renderer,
        }
    }
    
    pub fn render(&mut self, lights: &Vec<Light>, camera: &mut Camera, entities: &Vec<Entity>, normal_mapped_entities: &Vec<Entity>, terrains: &Vec<Terrain>, 
                player: &Player, water_tiles: &Vec<WaterTile>, skybox: &Skybox, display: &Display, framebuffers: &mut FboMap, particle_master: &mut ParticleMaster, 
                entities_with_env_map: &Vec<Entity>, _debug_entity: &mut DebugEntity) {

        self.do_shadowmap_render_passes(camera, framebuffers, entities, normal_mapped_entities, player, lights, terrains);

        self.do_water_render_passes(water_tiles, camera, framebuffers, entities, normal_mapped_entities, terrains, player, lights, skybox, display);
        
        let camera_tex_fbo = framebuffers.fbos.get_mut(FboMap::CAMERA_TEXTURE_FBO_MULTI).expect("Must have a camera output fbo to which to render the scene for post processing");
        camera_tex_fbo.bind(); // we will unbind it later after particle effects are drawn

        let above_infinity_plane = Vector4f::new(0.0, -1.0, 0.0, 10_000.0);
        self.render_pass(lights, camera, entities, normal_mapped_entities, terrains, player, skybox, &display.wall_clock, &above_infinity_plane);
        // render water
        self.water_renderer.render(water_tiles, framebuffers, camera, display, lights);

        // render entities which have an env map -> for the time being this happens outside of render pass but needs to be integrated at some point
        self.env_map_renderer.render(entities_with_env_map, camera, &skybox.model.day_texture_id);

        // render particles
        particle_master.render(&camera);
        display.restore_default_framebuffer();

        //let obb_ref = &self.shadowmap_renderer.shadow_box.frustum_corners;
        //self.debug_renderer.render(debug_entity, camera, obb_ref); 
        //debug_entity.position = self.shadowmap_renderer.shadow_box.world_space_center.clone();
        //debug_entity.scale = Vector3f::new(100.0, 100.0, 100.0);
        //debug_entity.scale = 0.80 * Vector3f::new(self.shadowmap_renderer.shadow_box.width, self.shadowmap_renderer.shadow_box.height, self.shadowmap_renderer.shadow_box.length);
        //self.debug_renderer.render_cube(debug_entity, camera);
    }

    fn do_shadowmap_render_passes(&mut self, camera: &mut Camera, framebuffers: &mut FboMap, entities: &Vec<Entity>, 
                normal_mapped_entities: &Vec<Entity>, player: &Player, lights: &Vec<Light>, terrains: &Vec<Terrain>) {
        
        gl::helper::push_debug_group(RenderGroup::SHADOW_MAP_PASS.id, RenderGroup::SHADOW_MAP_PASS.name);

        let shadowmap_fbo = framebuffers.fbos.get_mut(FboMap::SHADOW_MAP_FBO).expect("Must have shadowmap fbo to render shadowmaps");
        shadowmap_fbo.bind();
        self.shadowmap_renderer.start_render(camera, &lights[0]);
        self.shadowmap_renderer.shadow_params.shadow_map_texture = shadowmap_fbo.depth_texture.expect("A shadowmup must have a depth texture or crash");

        // render into the shadowmap depth buffer all the entities that we want to cast shadows
        let entity_by_tex = MasterRenderer::group_entities_by_tex(entities);
        for (tex_model, entity_group) in entity_by_tex {
            self.shadowmap_renderer.prepare_textured_model(tex_model);
            self.shadowmap_renderer.render(&entity_group);
            self.shadowmap_renderer.cleanup_textured_model();
        }

        let norm_entity_by_tex = MasterRenderer::group_entities_by_tex(normal_mapped_entities);
        for (tex_model, entity_group) in norm_entity_by_tex {
            self.shadowmap_renderer.prepare_textured_model(tex_model);
            self.shadowmap_renderer.render(&entity_group);
            self.shadowmap_renderer.cleanup_textured_model();
        }

        if let player::PlayerEntityType::StaticModelEntity(entity) = &player.entity {
            self.shadowmap_renderer.prepare_textured_model(&entity.model);
            self.shadowmap_renderer.render_entity(entity);
            self.shadowmap_renderer.cleanup_textured_model();
        }

        self.shadowmap_renderer.render_terrain(terrains);

        self.shadowmap_renderer.stop_render();

        gl::helper::pop_debug_group();
    }

    fn do_water_render_passes(&mut self, water_tiles: &Vec<WaterTile>, camera: &mut Camera, framebuffers: &mut FboMap,
                entities: &Vec<Entity>, normal_mapped_entities: &Vec<Entity>, terrains: &Vec<Terrain>, player: &Player, lights: &Vec<Light>,
                skybox: &Skybox, display: &Display) {

        if water_tiles.is_empty() {
            return;
        }

        gl::helper::push_debug_group(RenderGroup::REFLECT_REFRACT_PASS.id, RenderGroup::REFLECT_REFRACT_PASS.name);
        // enable clip plane                    
        gl::enable(gl::CLIP_DISTANCE0);

        let water_height = WaterTile::get_water_height(water_tiles);
        let tiny_overlap = 0.07; // to prevent glitches near the edge of the water
        let above_water_clip_plane = Vector4f::new(0.0, -1.0, 0.0, water_height + tiny_overlap);
        let below_water_clip_plane = Vector4f::new(0.0, 1.0, 0.0, -water_height + tiny_overlap);        
        
        camera.set_to_reflected_ray_camera_origin(water_height);
        let reflection_fbo = framebuffers.fbos.get_mut(FboMap::REFLECTION_FBO).expect("Must have reflection fbo for water render");
        reflection_fbo.bind();
        self.render_pass(lights, camera, entities, normal_mapped_entities, terrains, player, skybox, &display.wall_clock, &below_water_clip_plane);
        camera.set_to_reflected_ray_camera_origin(water_height);

        // we should also move camera before refraction to account for refracted angle?
        let refraction_fbo = framebuffers.fbos.get_mut(FboMap::REFRACTION_FBO).expect("Must have refraction fbo for water render");
        refraction_fbo.bind();
        self.render_pass(lights, camera, entities, normal_mapped_entities, terrains, player, skybox, &display.wall_clock, &above_water_clip_plane);

        gl::disable(gl::CLIP_DISTANCE0); // apparently this doesnt work on all drivers?   

        gl::helper::pop_debug_group();     
    }

    fn render_pass(&mut self, lights: &Vec<Light>, camera: &Camera, entities: &Vec<Entity>, normal_mapped_entities: &Vec<Entity>, terrains: &Vec<Terrain>, 
                player: &Player, skybox: &Skybox, wall_clock: &WallClock, clip_plane: &Vector4f) {

        gl::helper::push_debug_group(RenderGroup::DRAW_ENTITIES.id, RenderGroup::DRAW_ENTITIES.name);
        self.prepare();

        // render entites
        self.entity_renderer.start_render(lights, camera, &MasterRenderer::SKY_COLOR, &self.shadowmap_renderer.get_to_shadow(), &self.shadowmap_renderer.shadow_params);
        let groups_by_tex = MasterRenderer::group_entities_by_tex(entities);
        for (textured_model, entity_vec) in groups_by_tex.iter() {
            self.entity_renderer.prepare_textured_model(textured_model, clip_plane);
            for entity in entity_vec {
                // load transform matrix into shader
                self.entity_renderer.render(entity);
            }
            self.entity_renderer.unprepare_textured_model(textured_model);
        }        
        // render player
        if !player.is_invisible_immovable {
            match &player.entity {
                player::PlayerEntityType::StaticModelEntity(entity) => {
                    self.entity_renderer.prepare_textured_model(&entity.model, clip_plane); 
                    self.entity_renderer.render(entity);
                    self.entity_renderer.unprepare_textured_model(&entity.model);
                },
                player::PlayerEntityType::AnimatedModelEntity(entity) => {
                    self.animated_entity_renderer.render(entity, camera);
                },
            }
        }

        self.entity_renderer.stop_render();
        gl::helper::pop_debug_group();     

        gl::helper::push_debug_group(RenderGroup::DRAW_NORMAL_MAP_ENTITIES.id, RenderGroup::DRAW_NORMAL_MAP_ENTITIES.name);
        // render normal mapped entites
        self.normal_map_entity_renderer.start_render(lights, camera, &MasterRenderer::SKY_COLOR);
        let groups_by_tex = MasterRenderer::group_entities_by_tex(normal_mapped_entities);
        for (textured_model, entity_vec) in groups_by_tex.iter() {
            self.normal_map_entity_renderer.prepare_textured_model(textured_model, clip_plane);
            for entity in entity_vec {
                // load transform matrix into shader
                self.normal_map_entity_renderer.render(entity);
            }
            self.normal_map_entity_renderer.unprepare_textured_model(textured_model);
        }
        self.normal_map_entity_renderer.stop_render(); 
        gl::helper::pop_debug_group();

        // render terrain
        gl::helper::push_debug_group(RenderGroup::DRAW_TERRAIN.id, RenderGroup::DRAW_TERRAIN.name);
        self.terrain_renderer.start_render(lights, camera, &MasterRenderer::SKY_COLOR, &self.shadowmap_renderer.get_to_shadow(), &self.shadowmap_renderer.shadow_params);
        for terrain in terrains.iter() {
            self.terrain_renderer.prepare_terrain(terrain, clip_plane);
            self.terrain_renderer.render(terrain);
            self.terrain_renderer.unprepare_terrain();
        }
        self.terrain_renderer.stop_render();
        gl::helper::pop_debug_group();

        gl::helper::push_debug_group(RenderGroup::DRAW_SKYBOX.id, RenderGroup::DRAW_SKYBOX.name);
        self.skybox_renderer.render(camera, skybox, &MasterRenderer::SKY_COLOR, wall_clock, clip_plane);
        gl::helper::pop_debug_group();
    }
    
    fn prepare(&self) {
        gl::helper::enable_backface_culling();
        gl::enable(gl::DEPTH_TEST);
        let (Vector3f{x : r, y : g, z : b}, a) = (MasterRenderer::SKY_COLOR, 1.0);
        gl::clear_color(r, g, b, a);
        gl::clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }

    fn group_entities_by_tex<'b>(entities: &'b Vec<Entity>) -> HashMap<&'b TexturedModel, Vec<&'b Entity>> {
        let mut groups_by_tex = HashMap::new();

        for entity in entities.iter() {
            let group = groups_by_tex.entry(&entity.model).or_insert(Vec::new());
            group.push(entity);
        }

        groups_by_tex
    }
}