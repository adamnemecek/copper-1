use super::shader_program::ShaderProgram;
use crate::math::{
    Matrix4f,
};
use crate::models::RawModel;

pub struct ParticleShader {
    program: ShaderProgram,
    location_proj_mat: i32,
    location_model_view_mat: i32,
}

impl ParticleShader {
    pub fn new() -> Self {
        let (
            mut location_proj_mat,
            mut location_model_view_mat,
        ) = Default::default();

        let program = ShaderProgram::new(
            "res/shaders/particleVertShader.glsl", 
            "res/shaders/particleFragShader.glsl", 
            |shader_program| {
                shader_program.bind_attribute(RawModel::POS_ATTRIB, "position");
            }, 
            |shader_program| {
                location_proj_mat = shader_program.get_uniform_location("projection_matrix");
                location_model_view_mat = shader_program.get_uniform_location("model_view_matrix");
            }
        );
        ParticleShader {
            program,
            location_proj_mat,
            location_model_view_mat,
        }
    }

    pub fn start(&mut self) {
        self.program.start();
    }

    pub fn stop(&mut self) {
        self.program.stop();
    }

    pub fn load_projection_matrix(&mut self, proj_mat: &Matrix4f) {
        ShaderProgram::load_matrix(self.location_proj_mat, proj_mat);
    }

    pub fn load_model_view_matrix(&mut self, model_view_mat: &Matrix4f) {
        ShaderProgram::load_matrix(self.location_model_view_mat, model_view_mat);
    }
}