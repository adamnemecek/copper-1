use super::scene::Scene;

extern crate rand;
use rand::prelude::*;
use rand::{Rng, SeedableRng};

use crate::display::framebuffers::FboMap;
use crate::entities::{
    AnimatedEntity,
    Entity,
    Camera,
    Light,
    Player,
    Ground,
    Skybox,
    Terrain,
    DebugEntity,
    WaterTile,
};
use crate::guis::{
    GuiPanel,
    TextMaterial,
};
use crate::math::{Matrix4f, Vector3f, Vector2f};
use crate::models::{
    ResourceManager,
    Models,
    ModelType,
    CorrectionTransform,
};
use crate::particles::{
    AdvancedParticleSystem,
    ParticleSystemProps,
}; 

pub fn init_scene_resources(resource_manager: &mut ResourceManager) {
    //resource_manager.init(&Models::TREE);
    resource_manager.init(&Models::FERN);
    //resource_manager.init(&Models::GRASS);
    //resource_manager.init(&Models::FLOWERS);
    resource_manager.init(&Models::TOON_ROCKS);
    resource_manager.init(&Models::BOBBLE_TREE);
    resource_manager.init(&Models::LOW_POLY_TREE);    
    resource_manager.init(&Models::CRATE);
    resource_manager.init(&Models::LAMP);
    resource_manager.init(&Models::BARREL);
    resource_manager.init(&Models::BOULDER);
    resource_manager.init(&Models::LANTERN);

    resource_manager.init(&Models::PLAYER);
    resource_manager.init_player(CorrectionTransform::create_coord_correction(Matrix4f::get_rotation(-90.0, 0.0, 0.0)));

    resource_manager.init_terrain_textures();
    resource_manager.init_terrain_model();

    resource_manager.init_skybox();
    resource_manager.init_water();

    resource_manager.init_quad_model();
    resource_manager.init_gui_textures();
    resource_manager.init_fonts();

    resource_manager.init_particle_model();
    resource_manager.init_particle_textures();
    // debug entity
    resource_manager.init_debug_cuboid_model();
}

pub fn create_scene(resource_manager: &mut ResourceManager, _framebuffers: &FboMap) -> Scene {
    let mut entities = Vec::new();    
    let mut rng: StdRng = SeedableRng::seed_from_u64(0xb000u64);
    const X_WIDTH: f32 = 1000.0;
    const Z_WIDTH: f32 = -1000.0;
    
    let mut terrains = Vec::new();    
    for i in -2..2 {
        for j in -2..2 {            
            let terrain = Terrain::new(i, j, resource_manager.terrain_pack(), resource_manager.blend_texture(), resource_manager.terrain_model());
            terrains.push(terrain);
        }
    }
    let ground = Ground { terrains };

    for _ in 0..100 {
        // let r_pos = ground.create_pos_on_terrain(rng.gen::<f32>() * X_WIDTH - X_WIDTH/2.0, rng.gen::<f32>() * Z_WIDTH);
        // let r_rot = Vector3f::new(0.0, 0.0, 0.0);
        // entities.push(Entity::new(resource_manager.model(ModelType::Tree), r_pos, r_rot, 3.0));

        let r_pos = ground.create_pos_on_terrain(rng.gen::<f32>() * X_WIDTH - X_WIDTH/2.0, rng.gen::<f32>() * Z_WIDTH);
        let r_rot = Vector3f::new(0.0, 0.0, 0.0);
        entities.push(Entity::new(resource_manager.model(ModelType::LowPolyTree), r_pos, r_rot, 0.5));

        let r_pos = ground.create_pos_on_terrain(rng.gen::<f32>() * X_WIDTH - X_WIDTH/2.0, rng.gen::<f32>() * Z_WIDTH);
        let r_rot = Vector3f::new(0.0, rng.gen::<f32>() * 180.0, 0.0);
        let fern_model = resource_manager.model(ModelType::Fern);
        let atlas_texture_index: usize = rng.gen_range(0, fern_model.texture.number_of_rows_in_atlas * fern_model.texture.number_of_rows_in_atlas);
        entities.push(Entity::new_with_texture_atlas(fern_model, r_pos, r_rot, 0.6, atlas_texture_index));

        // let r_pos = ground.create_pos_on_terrain(rng.gen::<f32>() * X_WIDTH - X_WIDTH/2.0, rng.gen::<f32>() * Z_WIDTH);
        // let r_rot = Vector3f::new(0.0, rng.gen::<f32>() * 180.0, 0.0);
        // entities.push(Entity::new(resource_manager.model(ModelType::Grass), r_pos, r_rot, 1.0));

        // let r_pos = ground.create_pos_on_terrain(rng.gen::<f32>() * X_WIDTH - X_WIDTH/2.0, rng.gen::<f32>() * Z_WIDTH);
        // let r_rot = Vector3f::new(0.0, rng.gen::<f32>() * 180.0, 0.0);
        // entities.push(Entity::new(resource_manager.model(ModelType::Flowers), r_pos, r_rot, 1.0));

        let r_pos = ground.create_pos_on_terrain(rng.gen::<f32>() * X_WIDTH - X_WIDTH/2.0, rng.gen::<f32>() * Z_WIDTH);
        let r_rot = Vector3f::new(0.0, rng.gen::<f32>() * 180.0, 0.0);
        entities.push(Entity::new(resource_manager.model(ModelType::BobbleTree), r_pos, r_rot, 0.5));

        let r_pos = ground.create_pos_on_terrain(rng.gen::<f32>() * X_WIDTH - X_WIDTH/2.0, rng.gen::<f32>() * Z_WIDTH);
        let r_rot = Vector3f::new(0.0, rng.gen::<f32>() * 180.0, 0.0);
        entities.push(Entity::new(resource_manager.model(ModelType::ToonRocks), r_pos, r_rot, 1.0));
    }    

    ///////////// animated player but without all the fancy shaders ///////////////////////
    let player_entity = AnimatedEntity::new(resource_manager.player_model(), ground.create_pos_on_terrain(150.0, -250.0), Vector3f::new(0.0, 0.0, 0.0), 0.3);
    let player = Player::new_animated(player_entity);

    //////////// non animated player with all shaders ///////////////////////////
    // let player_entity = Entity::new(resource_manager.model(ModelType::Player), ground.create_pos_on_terrain(150.0, -250.0), Vector3f::new(0.0, 180.0, 0.0), 0.3);    
    // let player = Player::new(player_entity);

    let mut box_pos = ground.create_pos_on_terrain(0.0, -150.0);
    box_pos.y += 4.0;
    let box_entity = Entity::new(resource_manager.model(ModelType::Crate), box_pos, Vector3f::new(0.0, 0.0, 0.0), 5.0);
    entities.push(box_entity);

    let water_tiles = vec![
        // put the water slightly below 0 to reduce z-fighting since a lot of terrain is at 0
        WaterTile::new(Vector3f::new(150.0, -0.2, -250.0), resource_manager.water_model()),
    ];

    let mut normal_mapped_entities = Vec::new();   
    normal_mapped_entities.push(Entity::new(resource_manager.model(ModelType::Barrel), ground.create_pos_above_terrain(150.0, 10.0, -255.0), Vector3f::zero(), 0.5));
    normal_mapped_entities.push(Entity::new(resource_manager.model(ModelType::Boulder), ground.create_pos_above_terrain(140.0, 10.0, -255.0), Vector3f::zero(), 0.5));

    let debug_entity = DebugEntity::new(resource_manager.debug_cuboid_model());

    let mut camera = Camera::default();
    camera.position = Vector3f::new(0.0, 80.0, 5.0);

    let skybox = Skybox::new(resource_manager.skybox(), 0.0);

    let texts = vec![
        resource_manager.create_gui_text("hello\nworld", 
            ResourceManager::COPPER_SDF_FONT_TYPE, 4, Vector2f::new(-0.8, -0.60), 
            TextMaterial {
                color: Vector3f::new(1.0, 0.0, 0.0), 
                width: 0.5, edge: 0.3,
                outline_width: 0.5, outline_edge: 0.4,
                ..TextMaterial::default()
            }
        ),
        resource_manager.create_gui_text("Made with Rust", 
            ResourceManager::COPPER_SDF_FONT_TYPE, 4, Vector2f::new(0.3, -0.95), 
            TextMaterial {
                color: Vector3f::new(0.0, 0.0, 1.0), 
                outline_color: Vector3f::new(0.0, 0.0, 0.0),
                offset: Vector2f::new(-0.002, -0.002),
                outline_width: 0.5, outline_edge: 0.4,
                ..TextMaterial::default()
            }
        ),
    ];

    let healthbar = resource_manager.get_gui_texture(ResourceManager::HEALTHBAR_TEXTURE);
    let gui_background = resource_manager.get_gui_texture(ResourceManager::GUI_BACKGROUND_TEXTURE);
    //let shadow_map = framebuffers.shadowmap_fbo.depth_texture;
    let guis = vec!{
        GuiPanel::new(gui_background, Vector2f::new(-0.73, -0.7), Vector2f::new(0.25, 0.25)),
        GuiPanel::new(healthbar, Vector2f::new(-0.75, -0.75), Vector2f::new(0.2, 0.2)),
        //GuiPanel::new(shadow_map, Vector2f::new(0.7, 0.7), Vector2f::new(0.3, 0.3)),
    };
    
    // add lantern which isnt a light source but has extra_info_map
    entities.push(Entity::new(resource_manager.model(ModelType::Lantern), ground.create_pos_on_terrain(130.0, -190.0), Vector3f::new(0.0, 0.0, 0.0), 1.0));

    let lights = vec!{        
        Light::new_infinite(Vector3f::new(5000.0, 10000.0, 5000.0), Vector3f::new(1.0, 1.0, 1.0)), // sunlight, no attenuation
        Light::new_point(ground.create_pos_above_terrain(185.0,12.5,-293.0), Vector3f::new(2.0, 0.0, 0.0), Vector3f::new(1.0, 0.01, 0.002)),
        Light::new_point(ground.create_pos_above_terrain(370.0,14.0,-300.0), Vector3f::new(0.0, 2.0, 2.0), Vector3f::new(1.0, 0.01, 0.002)),
        Light::new_point(ground.create_pos_above_terrain(120.0,14.0,-240.0), Vector3f::new(2.0, 2.0, 0.0), Vector3f::new(1.0, 0.01, 0.002)),        
    };
    // add lamps 
    entities.push(Entity::new(resource_manager.model(ModelType::Lamp), ground.create_pos_on_terrain(185.0, -293.0), Vector3f::new(0.0, 0.0, 0.0), 1.0));
    entities.push(Entity::new(resource_manager.model(ModelType::Lamp), ground.create_pos_on_terrain(370.0, -300.0), Vector3f::new(0.0, 0.0, 0.0), 1.0));
    entities.push(Entity::new(resource_manager.model(ModelType::Lamp), ground.create_pos_on_terrain(120.0, -240.0), Vector3f::new(0.0, 0.0, 0.0), 1.0));

    // particle effect systems
    let mut particle_spawn_point = player.position().clone();
    particle_spawn_point.x += 10.0;
    particle_spawn_point.y += 10.0;
    let particle_system = AdvancedParticleSystem::new(resource_manager.particle_model(), resource_manager.particle_texture(ResourceManager::PARTICLE_ATLAS),
        ParticleSystemProps { 
            particles_per_sec: 60.0, speed: 15.0, scale: 2.5, 
            gravity_effect: 0.5, life_length: 1.5, 
            speed_error: 0.3, life_error: 0.3, scale_error: 0.3, 
            randomize_rotation: true, direction: Some((Vector3f::new(0.0, 1.0, 0.0), 45.0)),
            additive_blending: false,
        }
    );
    let mut particle_spawn_point_fire = player.position().clone();
    particle_spawn_point_fire.x -= 50.0;
    particle_spawn_point_fire.z -= 30.0;
    let particle_system_fire = AdvancedParticleSystem::new(resource_manager.particle_model(), resource_manager.particle_texture(ResourceManager::FIRE_ATLAS),
        ParticleSystemProps { 
            particles_per_sec: 60.0, speed: 15.0, scale: 7.0, 
            gravity_effect: 0.0, life_length: 1.0, 
            speed_error: 0.3, life_error: 0.7, scale_error: 0.5, 
            randomize_rotation: true, direction: Some((Vector3f::new(0.0, 1.0, 0.0), 65.0)),
            additive_blending: true,
        }
    );
    let mut particle_spawn_point_smoke = player.position().clone();
    particle_spawn_point_smoke.z += 50.0;
    particle_spawn_point_smoke.y += 1.0;
    let particle_system_smoke = AdvancedParticleSystem::new(resource_manager.particle_model(), resource_manager.particle_texture(ResourceManager::SMOKE_ATLAS),
        ParticleSystemProps { 
            particles_per_sec: 30.0, speed: 15.0, scale: 6.5, 
            gravity_effect: 0.05, life_length: 1.5, 
            speed_error: 0.3, life_error: 0.3, scale_error: 0.1, 
            randomize_rotation: true, direction: Some((Vector3f::new(0.0, 1.0, 0.0), 50.0)),
            additive_blending: false,
        }
    );
    let particle_systems = vec![
        (particle_system, particle_spawn_point),
        (particle_system_fire, particle_spawn_point_fire),
        (particle_system_smoke, particle_spawn_point_smoke),
    ];

    Scene {
        entities, 
        normal_mapped_entities, 
        ground, 
        player,        
        quad_model: resource_manager.quad_model(), 
        water: water_tiles,
        debug_entity,
        camera,
        skybox,
        texts,
        guis,
        lights,
        particle_systems,
        uses_post_processing: true,
        entities_with_env_map: Vec::new(),
    }
}