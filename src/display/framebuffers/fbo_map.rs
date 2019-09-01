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

    const REFLECTION_FBO_WIDTH: usize = 1280;
    const REFLECTION_FBO_HEIGHT: usize = 720;

    const REFRACTION_FBO_WIDTH: usize = 1280;
    const REFRACTION_FBO_HEIGHT: usize = 720;

    pub const SHADOW_MAP_SIZE: usize = 4096;

    pub fn new(display: &Display) -> Self {
        let mut fbos = HashMap::new();
        fbos.insert(Self::REFLECTION_FBO, FramebufferObject::new(Self::REFLECTION_FBO_WIDTH, Self::REFLECTION_FBO_HEIGHT, FboFlags::COLOR));
        fbos.insert(Self::REFRACTION_FBO, FramebufferObject::new(Self::REFRACTION_FBO_WIDTH, Self::REFRACTION_FBO_HEIGHT, FboFlags::COLOR | FboFlags::DEPTH));
        fbos.insert(Self::SHADOW_MAP_FBO, FramebufferObject::new(Self::SHADOW_MAP_SIZE, Self::SHADOW_MAP_SIZE, FboFlags::SHADOW_DEPTH));        

        display.restore_default_framebuffer();
        FboMap {
            fbos
        }
    }
}