use super::shader_program::ShaderProgram;
use crate::entities::{
    Camera,
    Light,
};
use crate::models::RawModel;
use crate::math::{
    Matrix4f,
    Vector2f,
    Vector3f,
    Vector4f,
};

const NUM_LIGHTS: usize = 4;

pub struct NormalMapStaticShader {
    program: ShaderProgram,
    location_transformation_matrix: i32,
    location_projection_matrix: i32,
    location_view_matrix: i32,
    location_light_pos: [i32; NUM_LIGHTS],
    location_light_color: [i32; NUM_LIGHTS],
    location_shine_damper: i32,
    location_reflectivity: i32,
    location_uses_fake_lighting: i32,
    location_sky_color: i32,
    location_number_of_rows: i32,
    location_texture_offset: i32,
    location_attenuation: [i32; NUM_LIGHTS],
    location_clip_plane: i32,
    location_texture: i32,
    location_normal_map: i32,
}

impl NormalMapStaticShader {
    pub fn new() -> NormalMapStaticShader {
        let (
            mut location_transformation_matrix, 
            mut location_projection_matrix,
            mut location_view_matrix,
            mut location_light_pos,
            mut location_light_color,
            mut location_shine_damper,
            mut location_reflectivity,
            mut location_uses_fake_lighting,
            mut location_sky_color,
        ) = Default::default();

        let (
            mut location_number_of_rows, 
            mut location_texture_offset,
            mut location_attenuation,
            mut location_clip_plane,
            mut location_texture,
            mut location_normal_map,
        ) = Default::default();
        
        let shader_program = ShaderProgram::new(
            "res/shaders/normalMappedVertShader.glsl",
            None,
            "res/shaders/normalMappedFragShader.glsl", 
            |shader_prog| {
                shader_prog.bind_attribute(RawModel::POS_ATTRIB, "pos");
                shader_prog.bind_attribute(RawModel::TEX_COORD_ATTRIB, "tex_coord");
                shader_prog.bind_attribute(RawModel::NORMAL_ATTRIB, "normal");
                shader_prog.bind_attribute(RawModel::TANGENT_ATTRIB, "tangents");
            },
            |shader_prog| {                
                location_transformation_matrix = shader_prog.get_uniform_location("transform");
                location_projection_matrix = shader_prog.get_uniform_location("projection_matrix");
                location_view_matrix = shader_prog.get_uniform_location("view_matrix");
                // diffuse lighting
                location_light_pos = [0i32; NUM_LIGHTS];
                location_light_color = [0i32; NUM_LIGHTS];
                for i in 0..NUM_LIGHTS {
                    // TODO: maybe we should optimize these string allocations that we keep doing
                    location_light_pos[i] = shader_prog.get_uniform_location(&format!("light_pos[{}]", i));
                    location_light_color[i] = shader_prog.get_uniform_location(&format!("light_color[{}]", i));
                }
                // specular lighting
                location_shine_damper = shader_prog.get_uniform_location("shine_damper");
                location_reflectivity = shader_prog.get_uniform_location("reflectivity");
                // bad grass model hack
                location_uses_fake_lighting = shader_prog.get_uniform_location("uses_fake_lighting");
                // fog unfirom
                location_sky_color = shader_prog.get_uniform_location("sky_color");
                // atlas uniforms
                location_number_of_rows = shader_prog.get_uniform_location("number_of_rows");
                location_texture_offset = shader_prog.get_uniform_location("texture_offset");
                // point light attenuation
                location_attenuation = [0i32; NUM_LIGHTS];
                for i in 0..NUM_LIGHTS {
                    location_attenuation[i] = shader_prog.get_uniform_location(&format!("attenuation[{}]", i));
                }
                location_clip_plane = shader_prog.get_uniform_location("clip_plane");
                // setting up uniforms to bind samplers to texture units
                location_texture = shader_prog.get_uniform_location("texture_sampler");
                location_normal_map = shader_prog.get_uniform_location("normal_map_sampler");
        });

        NormalMapStaticShader {
            program: shader_program,
            location_transformation_matrix,
            location_projection_matrix,
            location_view_matrix,
            location_light_pos,
            location_light_color,
            location_shine_damper,
            location_reflectivity,
            location_uses_fake_lighting,
            location_sky_color,
            location_number_of_rows,
            location_texture_offset,
            location_attenuation,
            location_clip_plane,
            location_texture,
            location_normal_map,
        }
    }

    pub fn start(&mut self) {
        self.program.start();
    }

    pub fn stop(&mut self) {
        self.program.stop();
    }

    pub fn load_atlas_number_of_rows(&mut self, number_of_rows: usize) {
        ShaderProgram::load_float(self.location_number_of_rows, number_of_rows as f32);
    }

    pub fn load_atlas_offset(&mut self, offset: &Vector2f) {
        ShaderProgram::load_vector2d(self.location_texture_offset, offset);
    }

    pub fn load_sky_color(&mut self, sky_color: &Vector3f) {
        ShaderProgram::load_vector3d(self.location_sky_color, sky_color);
    }

    pub fn load_uses_fake_lighting(&mut self, uses_fake: bool) {
        ShaderProgram::load_bool(self.location_uses_fake_lighting, uses_fake);
    }

    pub fn load_shine_variables(&mut self, shine_damper: f32, reflectivity: f32) {
        ShaderProgram::load_float(self.location_shine_damper, shine_damper);
        ShaderProgram::load_float(self.location_reflectivity, reflectivity);
    }

    pub fn load_lights(&mut self, lights: &Vec<Light>) {        
        for i in 0..NUM_LIGHTS {
            if i < lights.len() {
                ShaderProgram::load_vector3d(self.location_light_pos[i], &lights[i].position);
                ShaderProgram::load_vector3d(self.location_light_color[i], &lights[i].color);
                ShaderProgram::load_vector3d(self.location_attenuation[i], &lights[i].attenuation);
            } else {
                // no light data means fewer than NUM_LIGHTS affect object
                ShaderProgram::load_vector3d(self.location_light_pos[i], &Vector3f::ZERO);
                ShaderProgram::load_vector3d(self.location_light_color[i], &Vector3f::ZERO);
                ShaderProgram::load_vector3d(self.location_attenuation[i], &Vector3f::POS_X_AXIS);
            }
        }
    }

    pub fn load_transformation_matrix(&mut self, transform_matrix: &Matrix4f) {
        ShaderProgram::load_matrix(self.location_transformation_matrix, transform_matrix);
    }

    pub fn load_projection_matrix(&mut self, projection_matrix: &Matrix4f) {
        ShaderProgram::load_matrix(self.location_projection_matrix, projection_matrix);
    }

    pub fn load_view_matrix(&mut self, camera: &Camera) {
        let view_matrix = Matrix4f::create_view_matrix(camera);
        ShaderProgram::load_matrix(self.location_view_matrix, &view_matrix);
    }

    pub fn load_clip_plane(&mut self, clip_plane: &Vector4f) {
        ShaderProgram::load_vector4d(self.location_clip_plane, clip_plane);
    }

    pub fn connect_texture_units(&mut self) {
        ShaderProgram::load_int(self.location_texture, 0);
        ShaderProgram::load_int(self.location_normal_map, 1);
    }
}