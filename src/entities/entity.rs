use crate::math::Vector3f;
use crate::models::TexturedModel;

pub struct Entity<'a> {
    pub model: &'a TexturedModel,
    pub position: Vector3f,
    pub rotation_deg: Vector3f,
    pub scale: f32,
}

impl<'a> Entity<'a> {
    pub fn new(model: &'a TexturedModel, position: Vector3f, rotation_deg: Vector3f, scale: f32) -> Entity<'a> {
        Entity {
            model,
            position,
            rotation_deg,
            scale,
        }
    }

    pub fn increase_position(&mut self, dx: f32, dy: f32, dz: f32) {
        self.position.x += dx;
        self.position.y += dy;
        self.position.z += dz;
    }

    pub fn increase_rotation(&mut self, drx: f32, dry: f32, drz: f32) {
        self.rotation_deg.x += drx;
        self.rotation_deg.y += dry;
        self.rotation_deg.z += drz;
    }
}