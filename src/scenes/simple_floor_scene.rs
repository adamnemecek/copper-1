use super::scene::Scene;

use crate::display::framebuffers::FboMap;
use crate::entities::{
    Entity,
    Camera,
    Light,
    Player,
    Ground,
    Skybox,
    DebugEntity,
};
use crate::guis::GuiPanel;
use crate::math::{Vector3f, Vector2f};
use crate::models::{
    ResourceManager,
    Models,
    ModelType,
    TextureId,
};

pub fn init_scene_resources(resource_manager: &mut ResourceManager) {
    resource_manager.init(&Models::PLAYER);
    
    resource_manager.init_skybox();
    resource_manager.init(&Models::FLOOR_TILE);
    resource_manager.init_quad_model();

    // debug entity
    resource_manager.init_debug_cuboid_model();
}

pub fn create_scene(resource_manager: &mut ResourceManager, framebuffers: &FboMap) -> Scene {

    let mut entities = Vec::new();
    let tile_size = 10.0;
    const LO: isize = -10;
    const HI: isize = 10;
    for x in LO..=HI {
        for z in LO..=HI {            
            let flat_floor_tile = Entity::new(resource_manager.model(ModelType::FloorTile), 
                Vector3f::new((x as f32) * 2.0 * tile_size, 0.0, (z as f32) * 2.0 * tile_size), 
                Vector3f::zero(), 
                tile_size);
            entities.push(flat_floor_tile);
        }
    }
    
    let terrains = Vec::new();    
    let ground = Ground { terrains };

    //let player_entity = Entity::new(resource_manager.model(ModelType::Player), ground.create_pos_on_terrain(150.0, -250.0), Vector3f::new(0.0, 180.0, 0.0), 0.3);
    let player_entity = Entity::new(resource_manager.model(ModelType::Player), Vector3f::new(0.0, 0.0, 0.0), Vector3f::new(0.0, 180.0, 0.0), 0.3);
    let player = Player::new(player_entity);
    
    let water_tiles = Vec::new();
    let normal_mapped_entities = Vec::new();

    let mut debug_entity = DebugEntity::new(resource_manager.debug_cuboid_model());
    debug_entity.position.y = 10.0;

    let mut camera = Camera::new(20.0, 50.0);
    camera.position = Vector3f::new(0.0, 0.0, 0.0);

    let skybox = Skybox::new(resource_manager.skybox(), 0.0);

    let texts = Vec::new();
    
    let lights = vec!{
        //Light::new_infinite(Vector3f::new(0.0, 10000.0, 0.0), Vector3f::new(0.8, 0.8, 0.8)), // sunlight, no attenuation
        Light::new_infinite(Vector3f::new(5000.0, 10000.0, -5000.0), Vector3f::new(0.8, 0.8, 0.8)), // sunlight, no attenuation
    };

    let particle_systems = Vec::new();

    let shadow_map = framebuffers.fbos[FboMap::SHADOW_MAP_FBO].depth_texture.expect("Must have shadowmap to show it in gui");
    let guis = vec!{
        GuiPanel::new(TextureId::FboTexture(shadow_map), Vector2f::new(0.6, 0.6), Vector2f::new(0.4, 0.4)),
    };

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
        uses_post_processing: false,
        entities_with_env_map: Vec::new(),
    }
}