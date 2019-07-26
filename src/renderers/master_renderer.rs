use std::collections::HashMap;
use crate::display::{
    Display,
    Framebuffers,
    FramebufferObject,
    WallClock,
};
use crate::gl;
use crate::entities::{
    Entity,
    Camera,
    Light,
    Terrain,
    Player,
    Skybox,
    WaterTile,
};
use crate::math::{
    Matrix4f,
    Vector3f,
    Vector4f,
};
use crate::models::{
    TexturedModel,
};
use crate::shadows::shadowmap_renderer::ShadowMapRenderer;
use super::entity_renderer::EntityRenderer;
use super::normal_map_entity_renderer::NormalMapEntityRenderer;
use super::terrain_renderer::TerrainRenderer;
use super::skybox_renderer::SkyboxRenderer;
use super::water_renderer::WaterRenderer;

pub struct MasterRenderer {    
    entity_renderer: EntityRenderer,
    normal_map_entity_renderer: NormalMapEntityRenderer,
    terrain_renderer: TerrainRenderer,
    skybox_renderer: SkyboxRenderer,
    water_renderer: WaterRenderer,
    shadowmap_renderer: ShadowMapRenderer,
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
        
        MasterRenderer {
            entity_renderer,
            normal_map_entity_renderer,
            terrain_renderer,
            skybox_renderer,
            water_renderer,
            shadowmap_renderer,
        }
    }
    
    pub fn render(&mut self, lights: &Vec<Light>, camera: &mut Camera, entities: &Vec<Entity>, normal_mapped_entities: &Vec<Entity>, terrains: &Vec<Terrain>, 
                player: &Player, water_tiles: &Vec<WaterTile>, skybox: &Skybox, display: &Display, framebuffers: &mut Framebuffers) {

        self.do_shadowmap_render_passes(camera, framebuffers, entities, normal_mapped_entities, terrains, player, lights);

        self.do_water_render_passes(water_tiles, camera, framebuffers, entities, normal_mapped_entities, terrains, player, lights, skybox, display);
        display.restore_default_framebuffer();

        let above_infinity_plane = Vector4f::new(0.0, -1.0, 0.0, 10_000.0);
        self.render_pass(lights, camera, entities, normal_mapped_entities, terrains, player, skybox, &display.wall_clock, &above_infinity_plane);
        // render water
        self.water_renderer.render(water_tiles, framebuffers, camera, display, lights);
    }

    fn do_shadowmap_render_passes(&mut self, camera: &mut Camera, framebuffers: &mut Framebuffers, entities: &Vec<Entity>, 
                normal_mapped_entities: &Vec<Entity>, terrains: &Vec<Terrain>, player: &Player, lights: &Vec<Light>) {
        
        framebuffers.shadowmap_fbo.bind();
        self.shadowmap_renderer.start_render(camera, &lights[0]);

        self.shadowmap_renderer.render(entities);
        self.shadowmap_renderer.render(normal_mapped_entities);
        self.shadowmap_renderer.render_terrain(terrains);
        self.shadowmap_renderer.render_player(player);

        self.shadowmap_renderer.stop_render();        
    }

    fn do_water_render_passes(&mut self, water_tiles: &Vec<WaterTile>, camera: &mut Camera, framebuffers: &mut Framebuffers,
                entities: &Vec<Entity>, normal_mapped_entities: &Vec<Entity>, terrains: &Vec<Terrain>, player: &Player, lights: &Vec<Light>,
                skybox: &Skybox, display: &Display) {
        // enable clip plane                    
        gl::enable(gl::CLIP_DISTANCE0); 

        let water_height = WaterTile::get_water_height(water_tiles);
        let tiny_overlap = 0.07; // to prevent glitches near the edge of the water
        let above_water_clip_plane = Vector4f::new(0.0, -1.0, 0.0, water_height + tiny_overlap);
        let below_water_clip_plane = Vector4f::new(0.0, 1.0, 0.0, -water_height + tiny_overlap);        
        
        camera.set_to_reflected_ray_camera_origin(water_height);
        framebuffers.reflection_fbo.bind();
        self.render_pass(lights, camera, entities, normal_mapped_entities, terrains, player, skybox, &display.wall_clock, &below_water_clip_plane);
        camera.set_to_reflected_ray_camera_origin(water_height);

        // we should also move camera before refraction to account for refracted angle?
        framebuffers.refraction_fbo.bind();
        self.render_pass(lights, camera, entities, normal_mapped_entities, terrains, player, skybox, &display.wall_clock, &above_water_clip_plane);

        gl::disable(gl::CLIP_DISTANCE0); // apparently this doesnt work on all drivers?        
    }

    fn render_pass(&mut self, lights: &Vec<Light>, camera: &Camera, entities: &Vec<Entity>, normal_mapped_entities: &Vec<Entity>, terrains: &Vec<Terrain>, 
                player: &Player, skybox: &Skybox, wall_clock: &WallClock, clip_plane: &Vector4f) {
        self.prepare();

        // render entites
        self.entity_renderer.start_render(lights, camera, &MasterRenderer::SKY_COLOR);
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
        self.entity_renderer.prepare_textured_model(&player.entity.model, clip_plane); 
        self.entity_renderer.render(&player.entity);
        self.entity_renderer.unprepare_textured_model(&player.entity.model);

        self.entity_renderer.stop_render();

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

        // render terrain
        self.terrain_renderer.start_render(lights, camera, &MasterRenderer::SKY_COLOR);
        for terrain in terrains.iter() {
            self.terrain_renderer.prepare_terrain(terrain, clip_plane);
            self.terrain_renderer.render(terrain);
            self.terrain_renderer.unprepare_terrain();
        }
        self.terrain_renderer.stop_render();

        self.skybox_renderer.render(camera, skybox, &MasterRenderer::SKY_COLOR, wall_clock, clip_plane);
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