use crate::gl;
use texture_lib::texture_loader::{
    load_rgba_2d_texture,
    Texture2DRGBA,
};
use crate::math::utils::f32_min;
use super::texture_id::TextureId;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::sync::mpsc;
use threadpool::ThreadPool;

pub struct ModelLoader {    
    vao_list: Vec<u32>,
    vbo_list: Vec<u32>,
    tex_list: Vec<u32>,
    texture_loading_rcv: mpsc::Receiver<TextureResult>,
    loaded_texture_snd: mpsc::Sender<TextureResult>,
    pub texture_token_map: HashMap<u32, u32>,
    texture_token_gen: u32,
    pub loading_texture_cnt: u32,
    // for cubemaps we need all 6 faces loaded before we can call the graphics functions
    cubemap_token_gen: u32,
    unprocessed_cubemap_textures: HashMap<u32, Vec<TextureResult>>,
    thread_pool: ThreadPool,
}

// the fields are Texture, temp_tex_id, params, texture_order (used for cubemaps)
type TextureResult = (Texture2DRGBA, u32, TextureParams, ExtraInfo);

#[derive(Default)]
pub struct ExtraInfo {
    is_cubemap: bool,
    order: usize,
    cubemap_token: u32,
}

#[derive(Default)]
pub struct TextureParams {
    reverse_texture_data: bool,
    use_mipmap: bool,
    mipmap_lod: f32,
    use_anisotropic_filtering: bool,
}

impl TextureParams {
    const DEFAULT_ANISOTROPIC_AMOUNT: f32 = 4.0;

    pub fn mipmapped_texture(mipmap_lod: f32) -> TextureParams {
        TextureParams {
            use_mipmap: true,
            mipmap_lod,
            ..Default::default()
        }
    }
    pub fn anisotropic_texture() -> TextureParams {
        TextureParams {
            use_mipmap: true,
            mipmap_lod: 0.0,
            use_anisotropic_filtering: true,
            ..Default::default()
        }
    }
}

impl Default for ModelLoader {
    fn default() -> Self {
        let (transmitter, receiver) = mpsc::channel();
        let pool = ThreadPool::new(8);
        ModelLoader {
            vao_list: Vec::new(),
            vbo_list: Vec::new(),
            tex_list: Vec::new(),
            texture_loading_rcv: receiver,
            loaded_texture_snd: transmitter,
            texture_token_map: HashMap::new(),
            texture_token_gen: 0,
            cubemap_token_gen: 0,
            unprocessed_cubemap_textures: HashMap::new(),
            loading_texture_cnt: 0,
            thread_pool: pool,
        }
    }
}

impl ModelLoader {
    pub fn new() -> ModelLoader {
        // some fancy disambiguation syntax here equivalnet to Default::default() and here also to RawModel::default since no multiple functions with same name
        // just in case you were wondering what Default::default does haha
        <ModelLoader as Default>::default()
    }

    pub fn update_resource_state(&mut self) {
        let recv_res = self.texture_loading_rcv.try_recv();
        if let Ok(texture_result) = recv_res {
            if texture_result.3.is_cubemap {
                let cubemap_token = texture_result.3.cubemap_token;
                let unprocessed_textures = self.unprocessed_cubemap_textures.get_mut(&cubemap_token).expect("Cubemap id must exist in the map. Make sure the entry is created as the token is generated");
                unprocessed_textures.push(texture_result);
                if unprocessed_textures.len() == 6 {
                    let cubemap_id = self.load_cube_map_into_graphics_lib(cubemap_token);
                    self.texture_token_map.insert(cubemap_token, cubemap_id);                    
                }
                self.loading_texture_cnt -= 1;
            } else {
                let tex_id = self.load_texture_into_graphics_lib(texture_result.0, texture_result.2);
                self.texture_token_map.insert(texture_result.1, tex_id);
                self.loading_texture_cnt -= 1;
            }
        } else if let Err(mpsc::TryRecvError::Disconnected) = recv_res {
            panic!("The generation side of texture loading has disconnected. This shouldnt happen")
        }
    }

    
    pub fn resolve(&self, texture_id: TextureId) -> TextureId {
        match texture_id {
            TextureId::Loading(tex_id) => { 
                let graphics_lib_tex_id = self.texture_token_map.get(&tex_id).expect("Requested a texture id that doesn't exist. This should never happen as all textureIds should only be generated by this struct");
                TextureId::Loaded(*graphics_lib_tex_id)
            },
            TextureId::Loaded(id) => TextureId::Loaded(id),
            TextureId::Empty => panic!("Not permitted to resolve an empty texture"),
            TextureId::FboTexture(_) => panic!("Not permitted to resolve a texture attachment of a framebuffer object"),
        }
    }

    

    pub fn load_to_vao_with_normal_map(&mut self, positions: &[f32], texture_coords: &[f32], indices: &[u32], normals: &[f32], tangents: &[f32]) -> RawModel {
        let vao_id = self.create_vao();
        self.bind_indices_buffer(indices);
        self.store_data_in_attribute_list(RawModel::POS_ATTRIB, 3, positions);
        self.store_data_in_attribute_list(RawModel::TEX_COORD_ATTRIB, 2, texture_coords);
        self.store_data_in_attribute_list(RawModel::NORMAL_ATTRIB, 3, normals);
        self.store_data_in_attribute_list(RawModel::TANGENT_ATTRIB, 4, tangents);
        self.unbind_vao();
        RawModel::new(vao_id, indices.len())
    }

    pub fn load_to_vao(&mut self, positions: &[f32], texture_coords: &[f32], indices: &[u32], normals: &[f32]) -> RawModel {
        let vao_id = self.create_vao();
        self.bind_indices_buffer(indices);
        self.store_data_in_attribute_list(RawModel::POS_ATTRIB, 3, positions);
        self.store_data_in_attribute_list(RawModel::TEX_COORD_ATTRIB, 2, texture_coords);
        self.store_data_in_attribute_list(RawModel::NORMAL_ATTRIB, 3, normals);
        self.unbind_vao();
        RawModel::new(vao_id, indices.len())
    }

    pub fn load_animated_model_to_vao(&mut self, positions: &[f32], texture_coords: &[f32], indices: &[u32], normals: &[f32], joint_weights: &[f32], joint_indices: &[i32]) -> RawModel {
        let vao_id = self.create_vao();
        self.bind_indices_buffer(indices);
        self.store_data_in_attribute_list(RawModel::POS_ATTRIB, 3, positions);
        self.store_data_in_attribute_list(RawModel::TEX_COORD_ATTRIB, 2, texture_coords);
        self.store_data_in_attribute_list(RawModel::NORMAL_ATTRIB, 3, normals);
        self.store_data_in_attribute_list(RawModel::JOINT_IDX_ATTRIB, 4, joint_indices);
        self.store_data_in_attribute_list(RawModel::JOINT_WEIGHT_ATTRIB, 4, joint_weights);
        self.unbind_vao();
        RawModel::new(vao_id, indices.len())
    }

    pub fn load_simple_model_to_vao(&mut self, positions: &[f32], dimension: u32) -> RawModel {
        let vao_id = self.create_vao();        
        self.store_data_in_attribute_list(RawModel::POS_ATTRIB, dimension, positions);        
        self.unbind_vao();
        RawModel::new(vao_id, positions.len() / 2)
    }

    pub fn load_dynamic_model_with_indices_to_vao(&mut self, unique_vertex_count: usize, indices: &[u32], dimension: u32) -> DynamicVertexIndexedModel {
        let vao_id = self.create_vao(); 
        self.bind_indices_buffer(indices);     
        let stream_draw_vbo = self.create_empty_float_vbo_for_attrib(RawModel::POS_ATTRIB, unique_vertex_count, dimension);
        self.unbind_vao();
        DynamicVertexIndexedModel {
            raw_model: RawModel::new(vao_id, indices.len()),
            stream_draw_vbo,
        }
    }

    pub fn load_quads_mesh_to_vao(&mut self, positions: &[f32], texture_coords: &[f32]) -> RawModel {
        let vao_id = self.create_vao(); 
        self.store_data_in_attribute_list(RawModel::POS_ATTRIB, 2, positions);        
        self.store_data_in_attribute_list(RawModel::TEX_COORD_ATTRIB, 2, texture_coords);   
        self.unbind_vao();
        RawModel::new(vao_id, positions.len() / 2)
    }

    pub fn load_cube_map(&mut self, cube_map_folder: &str) -> TextureId {
        self.cubemap_token_gen += 1;
        let cubemap_token = self.cubemap_token_gen;
        self.unprocessed_cubemap_textures.insert(cubemap_token, Vec::new());
        for i in 1..=6 {
            let filename = format!("{}/{}.png", cube_map_folder, i);
            self.load_texture_internal(&filename, TextureParams::default(), ExtraInfo { is_cubemap: true, order: i, cubemap_token});
        }
        TextureId::Loading(cubemap_token)
    }

    fn load_cube_map_into_graphics_lib(&mut self, loading_cubemap_id: u32) -> u32 {
        let cubemap_id = gl::gen_texture();
        self.tex_list.push(cubemap_id);
        gl::active_texture(gl::TEXTURE0);
        gl::bind_texture(gl::TEXTURE_CUBE_MAP, cubemap_id);

        let textures_for_cubemap = self.unprocessed_cubemap_textures.get_mut(&loading_cubemap_id).expect("Called the load with a bad cubemap id");
        assert!(textures_for_cubemap.len() == 6, "Must have 6 loaded textures for a cubemap");

        for tex_result in textures_for_cubemap {
            let face = tex_result.3.order;
            let width = tex_result.0.width;
            let height = tex_result.0.height;
            gl::tex_image_2d(gl::helper::CUBEMAP_FACES[face-1], 0, gl::RGBA, width, height, gl::UNSIGNED_BYTE, &tex_result.0.data);

            gl::tex_parameter_iv(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::LINEAR);
            gl::tex_parameter_iv(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, gl::LINEAR);
            gl::tex_parameter_iv(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE);
            gl::tex_parameter_iv(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE);            
        }
        gl::bind_texture(gl::TEXTURE_CUBE_MAP, 0);

        self.unprocessed_cubemap_textures.remove(&loading_cubemap_id);

        cubemap_id
    }

    pub fn load_texture_internal(&mut self, file_name: &str, params: TextureParams, extra_info: ExtraInfo) -> TextureId {
        self.texture_token_gen += 1;
        let texture_queue_id = self.texture_token_gen;

        let file_name_str = String::from(file_name);

        self.loading_texture_cnt += 1;

        let sender = self.loaded_texture_snd.clone();
        self.thread_pool.execute(move || {
            // make sure to not panic on thread
            let texture = load_rgba_2d_texture(&file_name_str, params.reverse_texture_data).expect(&format!("Failed to load texture: {}", file_name_str));
            sender.send((texture, texture_queue_id, params, extra_info)).expect("Failed to send");
        });

        TextureId::Loading(texture_queue_id)
    }

    fn load_texture_into_graphics_lib(&mut self, texture: Texture2DRGBA, params: TextureParams) -> u32 {
        let tex_id = gl::gen_texture();
        self.tex_list.push(tex_id);
        gl::active_texture(gl::TEXTURE0); // even though 0 is default i think, just to be explicit let's activate texture unit 0
        gl::bind_texture(gl::TEXTURE_2D, tex_id);

        gl::tex_parameter_iv(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT);
        gl::tex_parameter_iv(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT);        

        gl::tex_image_2d(gl::TEXTURE_2D, 0, gl::RGBA, texture.width, texture.height, gl::UNSIGNED_BYTE, &texture.data);
        if params.use_mipmap {
             // turn on mipmapping, has to be called after loading the texture data 
            gl::generate_mipmap(gl::TEXTURE_2D);
            gl::tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR);
            // set texture detail level (more negative means nicer) things at a high angle like grass/flowers may seem blurry if this is positive or 0
            gl::tex_parameterf(gl::TEXTURE_2D, gl::TEXTURE_LOD_BIAS, params.mipmap_lod);
            if params.use_anisotropic_filtering {
                let max_anisotropic = gl::get_floatv(gl::MAX_TEXTURE_MAX_ANISOTROPY_EXT);
                let min_amount = f32_min(TextureParams::DEFAULT_ANISOTROPIC_AMOUNT, max_anisotropic);
                gl::tex_parameterf(gl::TEXTURE_2D, gl::TEXTURE_MAX_ANISOTROPY_EXT, min_amount);
            }

        } else {        
            gl::tex_parameter_iv(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR);
            gl::tex_parameter_iv(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR);
        }

        gl::bind_texture(gl::TEXTURE_2D, 0);        
        tex_id
    }

     pub fn load_gui_texture(&mut self, file_name: &str, params: TextureParams) -> TextureId {
        self.load_texture_internal(file_name, params, ExtraInfo::default())
     }

    pub fn load_texture(&mut self, file_name: &str, params: TextureParams) -> ModelTexture {        
        ModelTexture {
            tex_id: self.load_texture_internal(file_name, params, ExtraInfo::default()),
            ..Default::default()
        }
    }

    pub fn load_particle_texture(&mut self, file_name: &str, params: TextureParams) -> ParticleTexture {        
        ParticleTexture {
            tex_id: self.load_texture_internal(file_name, params, ExtraInfo::default()),
            ..Default::default()
        }
    }

    pub fn load_terrain_texture(&mut self, file_name: &str, params: TextureParams) -> TerrainTexture {        
        TerrainTexture {
            tex_id: self.load_texture_internal(file_name, params, ExtraInfo::default()),
        }
    }

    pub fn create_empty_float_vbo(&mut self, float_count: usize) -> u32 {
        let vbo_id = gl::gen_buffer();
        self.vbo_list.push(vbo_id);
        gl::bind_buffer(gl::ARRAY_BUFFER, vbo_id);
        gl::buffer_data_unitialized::<f32>(gl::ARRAY_BUFFER, float_count, gl::STREAM_DRAW);
        gl::bind_buffer(gl::ARRAY_BUFFER, 0);
        vbo_id
    }

    pub fn create_empty_float_vbo_for_attrib(&mut self, attribute_num: u32, item_count: usize, coord_size: u32) -> u32 {
        let vbo_id = gl::gen_buffer();
        self.vbo_list.push(vbo_id);
        gl::bind_buffer(gl::ARRAY_BUFFER, vbo_id);
        gl::buffer_data_unitialized::<f32>(gl::ARRAY_BUFFER, item_count * (coord_size as usize), gl::STREAM_DRAW);
        gl::vertex_attrib_pointer(attribute_num, coord_size, gl::FLOAT);
        gl::bind_buffer(gl::ARRAY_BUFFER, 0);
        vbo_id
    }

    pub fn add_instanced_attrib(&mut self, vao: u32, vbo: u32, attrib: u32, components_per_attribute: u32, instanced_data_length: usize, offset: usize) {
        gl::bind_buffer(gl::ARRAY_BUFFER, vbo);
        gl::bind_vertex_array(vao);
        gl::vertex_attrib_pointer_interleaved::<f32>(attrib, components_per_attribute, gl::FLOAT, instanced_data_length, offset);
        gl::vertex_attrib_divisor(attrib, 1);
        gl::bind_vertex_array(0);
        gl::bind_buffer(gl::ARRAY_BUFFER, 0);
    }

    pub fn create_vao(&mut self) -> u32 {
        let vao_id = gl::gen_vertex_array();
        self.vao_list.push(vao_id);
        gl::bind_vertex_array(vao_id);                
        vao_id
    }
    
    fn unbind_vao(&self) {
        // binding to 0 unbinds
        gl::bind_vertex_array(0);
    }
    
    fn store_data_in_attribute_list<T: AsGlType>(&mut self, attribute_num: u32, coord_size: u32, data: &[T]) {
        let vbo_id = gl::gen_buffer();
        self.vbo_list.push(vbo_id);
        gl::bind_buffer(gl::ARRAY_BUFFER, vbo_id);
        gl::buffer_data(gl::ARRAY_BUFFER, data, gl::STATIC_DRAW);
        if T::as_gl_type() == gl::INT {
            gl::vertex_attrib_i_pointer(attribute_num, coord_size, T::as_gl_type());
        } else {
            gl::vertex_attrib_pointer(attribute_num, coord_size, T::as_gl_type());
        }
        gl::bind_buffer(gl::ARRAY_BUFFER, 0);
    }

    fn bind_indices_buffer(&mut self, indices: &[u32]) {
        let vbo_id = gl::gen_buffer();
        self.vbo_list.push(vbo_id);
        gl::bind_buffer(gl::ELEMENT_ARRAY_BUFFER, vbo_id);
        gl::buffer_data(gl::ELEMENT_ARRAY_BUFFER, indices, gl::STATIC_DRAW);
        // no unbind since we will bind data buffer next -> that means it HAS to be called after        
    }
}

trait AsGlType {
    fn as_gl_type() -> gl::types::GLenum;
}

impl AsGlType for f32 {
    fn as_gl_type() -> gl::types::GLenum {
        gl::FLOAT
    }
}

impl AsGlType for i32 {
    fn as_gl_type() -> gl::types::GLenum {
        gl::INT
    }
}

impl Drop for ModelLoader {
    fn drop(&mut self) {
        gl::delete_vertex_arrays(&self.vao_list[..]);
        gl::delete_buffers(&self.vbo_list[..]);
        gl::delete_textures(&self.tex_list);
    }
}

#[derive(Default, Clone, PartialEq, Eq, Hash)]
pub struct RawModel {
    pub vao_id: u32,
    pub vertex_count: usize,
}

impl RawModel {
    pub const POS_ATTRIB: u32 = 0;
    pub const TEX_COORD_ATTRIB: u32 = 1;
    pub const NORMAL_ATTRIB: u32 = 2;
    pub const TANGENT_ATTRIB: u32 = 3;
    pub const JOINT_IDX_ATTRIB: u32 = 4;
    pub const JOINT_WEIGHT_ATTRIB: u32 = 5;

    pub fn new(vao_id: u32, vertex_count: usize) -> RawModel {
        RawModel {
            vao_id,
            vertex_count,
        }
    }
}

#[derive(Clone)]
pub struct TerrainTexture {
    pub tex_id: TextureId,
}

#[derive(Clone)]
pub struct TerrainTexturePack {
    pub background_texture: TerrainTexture,
    pub r_texture: TerrainTexture,
    pub g_texture: TerrainTexture,
    pub b_texture: TerrainTexture,
}

#[derive(Clone)]
pub struct ModelTexture {
    pub tex_id: TextureId,
    pub shine_damper: f32,
    pub reflectivity: f32,
    pub has_transparency: bool,
    pub uses_fake_lighting: bool,
    // if this is 1 then the texture is not an atlas
    // also rows == columns since textures are power of two squares and so are textures
    pub number_of_rows_in_atlas: usize,
}

impl Default for ModelTexture {
    fn default() -> ModelTexture {
        ModelTexture {
            tex_id: TextureId::Empty,
            shine_damper: 1.0,
            reflectivity: 0.0,
            has_transparency: false,
            uses_fake_lighting: false,
            number_of_rows_in_atlas: 1,
        }
    }
}

#[derive(Clone)]
pub struct TexturedModel {
    pub raw_model: RawModel,
    pub texture: ModelTexture,
    pub normal_map_tex_id: Option<TextureId>,
    pub extra_info_tex_id: Option<TextureId>,
}

impl PartialEq for TexturedModel {
    fn eq(&self, other: &TexturedModel) -> bool {
        self.texture.tex_id == other.texture.tex_id && self.raw_model.vao_id == other.raw_model.vao_id
    }
}

impl Eq for TexturedModel {}

impl Hash for TexturedModel {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.texture.tex_id.hash(state);
        self.raw_model.vao_id.hash(state);
    }
}

#[derive(Clone)]
pub struct TerrainModel {
    pub raw_model: RawModel,
    pub height_map: Rc<Vec<Vec<f32>>>,
}

#[derive(Clone)]
pub struct QuadModel {
    pub raw_model: RawModel,
}

#[derive(Clone)]
pub struct SkyboxModel {
    pub raw_model: RawModel,
    pub day_texture_id: TextureId,
    pub night_texture_id: TextureId,
    pub cycles_day_night: bool,
}

#[derive(Clone)]
pub struct WaterModel {
    pub raw_model: RawModel,
    pub dudv_tex_id: TextureId,
    pub normal_map_tex_id: TextureId,
}

#[derive(Default, Clone, PartialEq, Eq, Hash)]
pub struct ParticleModel {
    pub raw_model: RawModel,
    pub stream_draw_vbo: u32,
}

#[derive(Clone)]
pub struct DynamicVertexIndexedModel {
    pub raw_model: RawModel,
    pub stream_draw_vbo: u32,
}

impl ParticleModel {    
    pub const MODELVIEW_COLUMN1: u32 = 1;    
    pub const MODELVIEW_COLUMN2: u32 = 2;    
    pub const MODELVIEW_COLUMN3: u32 = 3;    
    pub const MODELVIEW_COLUMN4: u32 = 4;    
    pub const TEX_OFFSET: u32 = 5;    
    pub const BLEND: u32 = 6;
    
    // 21 = (4 + 4 + 4 + 4) + 4 + 1 which is how many floats the shader needs
    pub const INSTANCED_DATA_LENGTH: usize = 21;
    pub const MAX_INSTANCES: usize = 10_000;
}

#[derive(Default, Clone, PartialEq, Eq, Hash)]
pub struct ParticleTexture {
    pub tex_id: TextureId,
    pub number_of_rows_in_atlas: usize,
    pub additive: bool,
}

#[derive(Default, Clone, PartialEq, Eq, Hash)]
pub struct ParticleTexturedModel {
    pub model: ParticleModel,
    pub texture: ParticleTexture,
}