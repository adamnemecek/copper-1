use crate::gl;
use crate::entities::{
    Entity,
    Camera,
    Light,
};
use crate::shaders::NormalMapStaticShader;
use crate::math::{
    Matrix4f,
    Vector3f,
    Vector4f,
};
use crate::models::{
    TexturedModel,
    RawModel,
};

pub struct NormalMapEntityRenderer {
    shader: NormalMapStaticShader,
}

impl NormalMapEntityRenderer {    
    
    pub fn new(projection_matrix: &Matrix4f) -> NormalMapEntityRenderer {     
        let mut shader = NormalMapStaticShader::new();
        shader.start();
        shader.load_projection_matrix(projection_matrix);
        shader.connect_texture_units();
        shader.stop();
        NormalMapEntityRenderer {
            shader,
        }
    }
    
    pub fn start_render(&mut self, lights: &Vec<Light>, camera: &Camera, sky_color: &Vector3f) {
        self.shader.start();
        self.shader.load_lights(lights);
        self.shader.load_view_matrix(camera);
        self.shader.load_sky_color(sky_color);
    }

    pub fn stop_render(&mut self) {
        self.shader.stop();
    }

    pub fn prepare_textured_model(&mut self, textured_model: &TexturedModel, clip_plane: &Vector4f) {
        if textured_model.texture.has_transparency {
            gl::helper::disable_culling();
        }

        gl::bind_vertex_array(textured_model.raw_model.vao_id);
        gl::enable_vertex_attrib_array(RawModel::POS_ATTRIB);
        gl::enable_vertex_attrib_array(RawModel::TEX_COORD_ATTRIB);
        gl::enable_vertex_attrib_array(RawModel::NORMAL_ATTRIB);
        gl::enable_vertex_attrib_array(RawModel::TANGENT_ATTRIB);

        self.shader.load_shine_variables(textured_model.texture.shine_damper, textured_model.texture.reflectivity);
        self.shader.load_uses_fake_lighting(textured_model.texture.uses_fake_lighting);
        self.shader.load_atlas_number_of_rows(textured_model.texture.number_of_rows_in_atlas);

        // clip plane for water 
        self.shader.load_clip_plane(clip_plane);

        gl::active_texture(gl::TEXTURE0); // activate bank 0
        gl::bind_texture(gl::TEXTURE_2D, textured_model.texture.tex_id.unwrap());
        gl::active_texture(gl::TEXTURE1); // activate bank 1
        gl::bind_texture(gl::TEXTURE_2D, textured_model.normal_map_tex_id.expect("A normal mapped entity must have a normal map texture").unwrap());
    }

    pub fn render(&mut self, entity: &Entity) {
        // load transform matrix into shader
        let transform_mat = Matrix4f::create_transform_matrix(&entity.position, &entity.rotation_deg, entity.scale);
        self.shader.load_transformation_matrix(&transform_mat);
        self.shader.load_atlas_offset(&entity.get_atlas_offset());
        
        gl::draw_elements(gl::TRIANGLES, entity.model.raw_model.vertex_count, gl::UNSIGNED_INT);
    }

    pub fn unprepare_textured_model(&self, textured_model: &TexturedModel) {
        if textured_model.texture.has_transparency {
            gl::helper::enable_backface_culling(); // restore backbace culling for next model
        }
        gl::disable_vertex_attrib_array(RawModel::POS_ATTRIB);
        gl::disable_vertex_attrib_array(RawModel::TEX_COORD_ATTRIB);
        gl::disable_vertex_attrib_array(RawModel::NORMAL_ATTRIB);
        gl::disable_vertex_attrib_array(RawModel::TANGENT_ATTRIB);

        gl::bind_vertex_array(0);
        gl::bind_texture(gl::TEXTURE_2D, 0);
    }
}