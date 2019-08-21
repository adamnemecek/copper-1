use crate::display::Display;
use crate::entities::{
    Camera,
    Entity,
    Light,
    Terrain,
};
use crate::gl;
use crate::math::{
    Matrix4f,
    Quaternion,
    Vector3f,
};
use crate::models::{
    RawModel,
    TexturedModel,
};
use super::shadow_box::ShadowBox;
use super::shadow_shader::ShadowShader;

pub struct ShadowMapRenderer {
    shadow_shader: ShadowShader,
    pub shadow_box: ShadowBox,
    world_to_lightspace: Matrix4f,    
    bias: Matrix4f,
    vp_matrix: Matrix4f,
    mvp_matrix: Matrix4f,
    //test_proj_matrix: Matrix4f,
}

impl ShadowMapRenderer {

    pub fn new(aspect_ratio: f32) -> Self {
        let shadow_box = ShadowBox::new(aspect_ratio, Display::FOV_HORIZONTAL, Display::NEAR, -ShadowBox::SHADOW_DISTANCE);
        let world_to_lightspace = Matrix4f::identity();        
        let bias = ShadowMapRenderer::create_bias_matrix();
        let shadow_shader = ShadowShader::new();
        let vp_matrix = Matrix4f::identity();
        let mvp_matrix = Matrix4f::identity();
        //let proj_mat = Matrix4f::create_projection_matrix(-50.0, -100.0, Display::FOV_HORIZONTAL, aspect_ratio);
        ShadowMapRenderer {
            shadow_shader,
            shadow_box,
            world_to_lightspace,            
            bias,
            vp_matrix,
            mvp_matrix,
            //test_proj_matrix: proj_mat,
        }
    }

    pub fn start_render(&mut self, camera: &Camera, sun: &Light) {        
        // testing with thinmatrix impl
        // self.shadow_box.update(camera, light_pitch_dg, light_yaw_dg);
        self.update_world_to_lightspace(&sun.position);
        self.shadow_box.update(camera, &self.world_to_lightspace);
        //self.shadow_box.update_odd(camera, &self.world_to_lightspace);
        //self.update_world_to_lightspace(light_pitch_dg, light_yaw_dg);
        
        gl::enable(gl::DEPTH_TEST);
        gl::clear(gl::DEPTH_BUFFER_BIT);
        self.shadow_shader.start();

        self.vp_matrix.make_identity();
        self.vp_matrix.pre_multiply_in_place(&self.world_to_lightspace);
        self.vp_matrix.pre_multiply_in_place(&self.shadow_box.ortho_proj_mat);
        // self.vp_matrix.multiply_in_place(&self.test_proj_matrix);
        // let cam_view = Matrix4f::create_view_matrix(camera);
        // self.vp_matrix.multiply_in_place(&cam_view);
    }

    pub fn prepare_textured_model(&mut self, model: &TexturedModel) {
        gl::bind_vertex_array(model.raw_model.vao_id);
        gl::enable_vertex_attrib_array(RawModel::POS_ATTRIB);
    }

    pub fn render(&mut self, entities: &Vec<&Entity>) {        
        for entity in entities.iter() {      
            self.render_entity(entity);
        }
    }

    pub fn render_entity(&mut self, entity: &Entity) {
        self.mvp_matrix.make_identity();
        self.mvp_matrix.post_multiply_in_place(&self.vp_matrix);
        let transform_mat = Matrix4f::create_transform_matrix(&entity.position, &entity.rotation_deg, entity.scale);
        self.mvp_matrix.post_multiply_in_place(&transform_mat);
        self.shadow_shader.load_mvp_matrix(&self.mvp_matrix);

        gl::draw_elements(gl::TRIANGLES, entity.model.raw_model.vertex_count, gl::UNSIGNED_INT);            
    }

    pub fn cleanup_textured_model(&mut self) {
        gl::disable_vertex_attrib_array(RawModel::POS_ATTRIB);
        gl::bind_vertex_array(0);
    }

    pub fn render_terrain(&mut self, terrains: &Vec<Terrain>) {
        for terrain in terrains.iter() {
            gl::bind_vertex_array(terrain.model.raw_model.vao_id);
            gl::enable_vertex_attrib_array(RawModel::POS_ATTRIB);
            
            let terrain_pos = Vector3f::new(terrain.x as f32, 0.0, terrain.z as f32);
            let terrain_rot = Vector3f::new(0.0, 0.0, 0.0);
            let transform_mat = Matrix4f::create_transform_matrix(&terrain_pos, &terrain_rot, 1.0);

            self.mvp_matrix.make_identity();
            self.mvp_matrix.pre_multiply_in_place(&transform_mat);
            self.mvp_matrix.pre_multiply_in_place(&self.vp_matrix);

            self.shadow_shader.load_mvp_matrix(&self.mvp_matrix);
            gl::draw_elements(gl::TRIANGLES, terrain.model.raw_model.vertex_count, gl::UNSIGNED_INT);

            gl::disable_vertex_attrib_array(RawModel::POS_ATTRIB);
        }
        gl::bind_vertex_array(0);
    }

    pub fn stop_render(&mut self) {
        self.shadow_shader.stop();
    }

    pub fn get_to_shadow(&self) -> Matrix4f {
        let mut res = Matrix4f::identity();
        res.pre_multiply_in_place(&self.world_to_lightspace);
        res.pre_multiply_in_place(&self.shadow_box.ortho_proj_mat);
        res.pre_multiply_in_place(&self.bias);
        res
    }

    fn update_world_to_lightspace(&mut self, sun_direction: &Vector3f) {
        let center = &self.shadow_box.world_space_center;        
        let mut normalized_sun_dir = sun_direction.clone();
        normalized_sun_dir.normalize();
        let sun_position = center + ((ShadowBox::SHADOW_DISTANCE / 2.0) * &normalized_sun_dir);
        // y axis up could be the same direction as the light .. so we rotate the sun direction by 90degs to get up
        // what if light is behind ?
        let mut up = Vector3f::POS_Y_AXIS;
        if Vector3f::parallel(&up, &normalized_sun_dir) {
            up = Vector3f::POS_Z_AXIS;
        }
        //let up = Quaternion::rotate_vector(&normalized_sun_dir, &Quaternion::from_angle_axis(90.0, &Vector3f::POS_X_AXIS));        
        self.world_to_lightspace = Matrix4f::look_at(&sun_position, center, &up);
    }

    fn update_world_to_lightspace0(&mut self, pitch: f32, yaw: f32) {
        self.world_to_lightspace.make_identity();        
        let center = &self.shadow_box.world_space_center;
        self.world_to_lightspace.translate(&(-center));
        // check create_view_matrix for explanation of why the signs are so odd here
        // the idea is again the same as in view matrix .. we want to transform from world coords to this reference frame
        // so we should take the inverse of the model matrix of light space .. but there are issues with just an inverse as explained in comment to create_view_matrix
        let angles = Vector3f::new(pitch, -yaw, 0.0);
        self.world_to_lightspace.rotate(&angles);
    }

    // we want to use the lightspace transform in a shader to sample from the depth map
    // the projection to lightspace ndc coords will leave us in the unit cube [-1,1]
    // but a texture has coords in range [0,1] so we use the bias matrix to apply the conversion directly to the matrix
    fn create_bias_matrix() -> Matrix4f {
        let mut bias = Matrix4f::identity();
        let s = Vector3f::new(0.5, 0.5, 0.5);
        let t = Vector3f::new(0.5, 0.5, 0.5);
        bias.scale(&s);
        bias.translate(&t);
        bias
    }
}