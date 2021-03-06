use super::framebuffer_object::{
    FramebufferObject,
    FboFlags,
};

use crate::display::Display;

use std::collections::HashMap;

pub struct FboMap {    
    pub fbos: HashMap<&'static str, FramebufferObject>,
}

impl FboMap {
    pub const REFLECTION_FBO: &'static str = "ReflectionFBO";
    pub const REFRACTION_FBO: &'static str = "RefractionFBO";
    pub const SHADOW_MAP_FBO: &'static str = "ShadowMapFBO";
    pub const CAMERA_TEXTURE_FBO_MULTI: &'static str = "CameraTextureMultisampled";
    // used for rendering the scene to a texture that can later be operated on with post processing
    pub const CAMERA_TEXTURE_FBO: &'static str = "CameraTexture";
    pub const CAMERA_BRIGHTNESS_FBO: &'static str = "CameraBrightnessTexture";

    const REFLECTION_FBO_WIDTH: usize = 1280;
    const REFLECTION_FBO_HEIGHT: usize = 720;

    const REFRACTION_FBO_WIDTH: usize = 1280;
    const REFRACTION_FBO_HEIGHT: usize = 720;

    pub const SHADOW_MAP_SIZE: usize = 4096;

    pub fn new_postprocessing_fbos(display: &Display) -> Self {
        let mut fbos = HashMap::new();
        let display_size = display.get_size();
        let camera_texture_fbo = FramebufferObject::new(display_size.width, display_size.height, FboFlags::COLOR_TEX | FboFlags::DEPTH_TEX, 1);
        let camera_brightness_fbo = FramebufferObject::new(display_size.width, display_size.height, FboFlags::COLOR_TEX, 1);                
        display.restore_default_framebuffer();

        fbos.insert(Self::CAMERA_TEXTURE_FBO, camera_texture_fbo);
        fbos.insert(Self::CAMERA_BRIGHTNESS_FBO, camera_brightness_fbo);

        FboMap {
            fbos
        }
    }

    pub fn new_rendering_fbos(display: &Display) -> Self {
        let mut fbos = HashMap::new();
        fbos.insert(Self::REFLECTION_FBO, FramebufferObject::new(Self::REFLECTION_FBO_WIDTH, Self::REFLECTION_FBO_HEIGHT, FboFlags::COLOR_TEX, 1));
        fbos.insert(Self::REFRACTION_FBO, FramebufferObject::new(Self::REFRACTION_FBO_WIDTH, Self::REFRACTION_FBO_HEIGHT, FboFlags::COLOR_TEX | FboFlags::DEPTH_TEX, 1));
        fbos.insert(Self::SHADOW_MAP_FBO, FramebufferObject::new(Self::SHADOW_MAP_SIZE, Self::SHADOW_MAP_SIZE, FboFlags::SHADOW_DEPTH, 0));
        // TODO: what if screen size changes 
        let display_size = display.get_size();
        fbos.insert(Self::CAMERA_TEXTURE_FBO_MULTI, FramebufferObject::new(display_size.width, display_size.height, FboFlags::MULTISAMPLED | FboFlags::COLOR_RENDERBUF | FboFlags::DEPTH_RENDERBUF, 2));
                
        display.restore_default_framebuffer();
        FboMap {
            fbos
        }
    }

    pub fn insert(&mut self, name: &'static str, fbo: FramebufferObject) {
        self.fbos.insert(name, fbo);
    }
}