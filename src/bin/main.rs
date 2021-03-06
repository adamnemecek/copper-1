extern crate copper;

use copper::animations::animator::Animator;
use copper::entities::player::{
    Player,
    PlayerEntityType,
};
use copper::display::{
    Display,
    framebuffers::FboMap,
};
use copper::renderers::{
    master_renderer::MasterRenderer,
    master_renderer::RenderGroup,
    gui_renderer::GuiRenderer,
};
use copper::models::{
    ResourceManager,
};
use copper::particles::{
    ParticleMaster,
};
use copper::post_processing::post_processing::PostProcessing;
use copper::mouse_picker::MousePicker;
use copper::scenes::{
    scene::Scene,
    all_scene::*,
    //test_scene::*,
    //environment_map_scene::*,
    load_screen::*,
};
use copper::gl;

use std::thread;
use std::time::Duration;

fn main() {
    let mut display = Display::create();
    let mut framebuffers = FboMap::new_rendering_fbos(&display);
    let mut resource_manager = ResourceManager::default();
    let mut gui_renderer = GuiRenderer::new();
    
    init_resourced_for_load_screen(&mut resource_manager);
    while resource_manager.are_textures_loading() && !display.is_close_requested() {
        thread::sleep(Duration::from_millis(10));
    }
    if display.is_close_requested() {
        return;
    }
    let load_screen = create_load_screen(&mut resource_manager);

    let mut resource_init_started = false;
    while (!resource_init_started || resource_manager.are_textures_loading()) && !display.is_close_requested() {        
        gui_renderer.render(&load_screen.guis, &load_screen.gui_model.raw_model, &load_screen.texts);
        display.update_display();
        if !resource_init_started {
            init_scene_resources(&mut resource_manager);
            resource_init_started = true;
        }
    }
    if display.is_close_requested() {
        return;
    }

    let mut scene = create_scene(&mut resource_manager, &framebuffers);
    
    let mut master_renderer = MasterRenderer::new(&display.projection_matrix, display.get_aspect_ratio());    
    
    let mut mouse_picker = MousePicker::new();

    let animator = Animator::default();

    // particle effects master
    let mut particle_master = ParticleMaster::new(&display.projection_matrix);
    let mut post_processing = PostProcessing::new(scene.quad_model.clone(), &display);
        
    while !display.is_close_requested() {

        update_animations(&animator, &mut scene.player, &display);

        scene.camera.move_camera(&display, &scene.player);
        
        update_mouse_picker_and_move_lamp(&mut mouse_picker, &display, &mut scene);

        spin_around_normal_mapped_entities(&mut scene, &display);
        
        particle_master.emit_particles(&scene.particle_systems, &display);
        
        particle_master.update(&display, &scene.camera);

        scene.player.move_player(&display, &scene.ground);

        scene.skybox.increase_rotation(&display);

        master_renderer.render(&scene.lights, &mut scene.camera, &scene.entities, &scene.normal_mapped_entities, &scene.ground.terrains, 
            &scene.player, &scene.water, &scene.skybox, &display, &mut framebuffers, &mut particle_master, &mut scene.entities_with_env_map, &mut scene.debug_entity);

        do_post_processing(scene.uses_post_processing, &mut post_processing, &mut framebuffers, &display);

        gui_renderer.render(&scene.guis, &scene.quad_model.raw_model, &scene.texts);

        display.update_display();
    }
}

fn update_animations(animator: &Animator, player: &mut Player, display: &Display) {
    let moving = player.is_moving();
    if let PlayerEntityType::AnimatedModelEntity(animated_model) = &mut player.entity {
        
        if moving {
            animated_model.model.animation.play();
        } else {
            animated_model.model.animation.stop();
        }

        animator.update_animation(animated_model, display);
    }
}

fn do_post_processing(uses_post_processing: bool, post_processing: &mut PostProcessing, framebuffers: &mut FboMap, display: &Display) {    
    
    gl::helper::push_debug_group(RenderGroup::POST_PROCESSING.id, RenderGroup::POST_PROCESSING.name);

    if uses_post_processing {
        do_anti_aliasing_for_fbo(post_processing, framebuffers, display);
        post_processing.do_post_processing(display);
    } else {
        do_anti_aliasing_to_screen(framebuffers, display);
    }

    gl::helper::pop_debug_group();
}

fn do_anti_aliasing_for_fbo(post_processing: &mut PostProcessing, framebuffers: &mut FboMap, display: &Display) {
    let camera_multisampled_fbo = framebuffers.fbos.get_mut(FboMap::CAMERA_TEXTURE_FBO_MULTI).expect("A multisampled fbo must be present MSAA processing of camera output");

    // create the target fbo that will later be read from in post processing shaders
    let mut camera_texture_fbo = post_processing.post_processing_fbos.fbos.get_mut(FboMap::CAMERA_TEXTURE_FBO).expect("A camera texture fbo is needed to write the resolved MSAA camera output to");
    camera_multisampled_fbo.resolve_to_fbo(gl::COLOR_ATTACHMENT0, &mut camera_texture_fbo, display);

    let mut camera_brightness_fbo = post_processing.post_processing_fbos.fbos.get_mut(FboMap::CAMERA_BRIGHTNESS_FBO).expect("A post processing brightness fbo is needed to write the glow brightness into");
    camera_multisampled_fbo.resolve_to_fbo(gl::COLOR_ATTACHMENT1, &mut camera_brightness_fbo, display);
}

fn do_anti_aliasing_to_screen(framebuffers: &mut FboMap, display: &Display) {
    let camera_multisampled_fbo = framebuffers.fbos.get_mut(FboMap::CAMERA_TEXTURE_FBO_MULTI).expect("A multisampled fbo must be present MSAA processing of camera output");
    camera_multisampled_fbo.resolve_to_screen(&display);
}

fn update_mouse_picker_and_move_lamp(mouse_picker: &mut MousePicker, display: &Display, scene: &mut Scene) {
    if let Some(selected_pos) = mouse_picker.update(&display, &display.projection_matrix, &scene.camera, &scene.ground) {            
        let last_pos = scene.entities.len()-1;
        scene.entities[last_pos].set_position(&selected_pos);
        scene.lights[3].position = selected_pos;
        scene.lights[3].position.y += 14.0; 
    }
}

fn spin_around_normal_mapped_entities(scene: &mut Scene, display: &Display) {
    const SPEED: f32 = 20.0;
    for idx in 0..scene.normal_mapped_entities.len() {
        scene.normal_mapped_entities[idx].increase_rotation(0.0, 0.0, SPEED * display.frame_time_sec);
    }
}