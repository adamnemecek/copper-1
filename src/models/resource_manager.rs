use super::{
    loader::{
        ModelLoader,
        TexturedModel,
        TerrainTexture,  
        TerrainTexturePack,
        TextureParams,
        TerrainModel,
        GuiModel,
        SkyboxModel,
        WaterModel,
        ParticleModel,
        ParticleTexture,
        DynamicVertexIndexedModel,
    },
    terrain_generator::HeightsGenerator,
};
use crate::entities::Terrain;
use crate::obj_converter::{
    load_obj_model,
    load_simple_obj_model
};
use std::collections::HashMap;
use crate::guis::{
    text::FontType,
    text::GuiText,
    text::TextMaterial,
    text::text_mesh_creator::*,
};
use crate::math::{
    Vector2f,
};

#[derive(Default)]
pub struct ResourceManager {
    loader: ModelLoader,
    terrain_generator: HeightsGenerator,
    texture_pack: Option<TerrainTexturePack>,
    blend_texture: Option<TerrainTexture>,
    terrain_model: Option<TerrainModel>,
    gui_model: Option<GuiModel>,
    skybox_model: Option<SkyboxModel>,
    water_model: Option<WaterModel>,
    particle_model: Option<ParticleModel>,
    debug_model: Option<DynamicVertexIndexedModel>,

    models: HashMap<ModelType, TexturedModel>,
    gui_textures: HashMap<&'static str, u32>,
    font_types: HashMap<&'static str, FontType>,
    particle_textures: HashMap<ParticleTextureProps, ParticleTexture>,
}

pub enum ResType {
    WaterModel,
    SkyboxModel,
    GuiModel,
    GuiRes(&'static str),
    TexAndModel {tex: &'static str, model: &'static str, model_props: ModelProps},
    TerrainTexPack {blend: &'static str, back: &'static str, a_chan: &'static str, g_chan: &'static str, b_chan: &'static str},
    TerrainModel {heightmap: &'static str},
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ModelType {
    Grass,
    Fern,
    Player,
    Tree,
    LowPolyTree,
    Flowers,
    Crate,
    Lamp,
    ToonRocks,
    BobbleTree,
    Barrel,
    Boulder,
}

pub type ParticleTextureProps = (&'static str, usize);

pub struct AtlasProps(usize);

pub struct ModelProps {
    pub has_transparency: bool,
    pub uses_fake_lighting: bool,
    pub uses_mipmaps: bool,
    pub shine_damper: f32,
    pub reflectivity: f32,
    pub atlas_props: AtlasProps,
    pub normal_map: Option<&'static str>,
}
impl ModelProps {
    fn get_texture_params(&self) -> TextureParams {        
        if self.uses_mipmaps {
            if self.normal_map.is_some() {
                TextureParams::mipmapped_texture(-2.4)
            } else {
                TextureParams::mipmapped_texture(-0.4)
            }
        } else {
            TextureParams::default()
        }        
    }
}

pub struct Model(ModelType, &'static str, &'static str, &'static ModelProps);

pub struct Models;

impl Models {
    const GUI_PROPS: ModelProps = ModelProps {
        has_transparency: false, 
        uses_fake_lighting: false, 
        uses_mipmaps: false,
        shine_damper: 1.0,
        reflectivity: 0.0, 
        atlas_props: AtlasProps(1),
        normal_map: None,
    };
    const COMMON_PROPS: ModelProps = ModelProps {
        has_transparency: false, 
        uses_fake_lighting: false, 
        uses_mipmaps: true,
        shine_damper: 1.0,
        reflectivity: 0.0,  
        atlas_props: AtlasProps(1),
        normal_map: None,
    };
    const SHINY_PROPS: ModelProps = ModelProps {
        has_transparency: false, 
        uses_fake_lighting: false, 
        uses_mipmaps: true,
        shine_damper: 20.0,
        reflectivity: 0.6,  
        atlas_props: AtlasProps(1),
        normal_map: None,
    };
    const FERN_PROPS: ModelProps = ModelProps { 
        has_transparency: true, 
        uses_fake_lighting: false, 
        uses_mipmaps: true, 
        shine_damper: 1.0,
        reflectivity: 0.0, 
        atlas_props: AtlasProps(2),
        normal_map: None,
    };    
    const GRASS_PROPS: ModelProps = ModelProps { 
        has_transparency: true, 
        uses_fake_lighting: true, 
        uses_mipmaps: true, 
        shine_damper: 1.0,
        reflectivity: 0.0, 
        atlas_props: AtlasProps(1),
        normal_map: None,
    };
    // point light is inside the lamp. to get it to light up the outer faces we make the outer faces have a vector that points up
    const LAMP_PROPS: ModelProps = ModelProps { 
        has_transparency: false, 
        uses_fake_lighting: true, 
        uses_mipmaps: true, 
        shine_damper: 1.0,
        reflectivity: 0.0, 
        atlas_props: AtlasProps(1),
        normal_map: None,
    };
    const BARREL_PROPS: ModelProps = ModelProps { 
        has_transparency: false, 
        uses_fake_lighting: false, 
        uses_mipmaps: true, 
        shine_damper: 10.0,
        reflectivity: 0.5, 
        atlas_props: AtlasProps(1),
        normal_map: Some("res/textures/normal_maps/barrelNormal.png"),
    };
    const BOULDER_PROPS: ModelProps = ModelProps { 
        has_transparency: false, 
        uses_fake_lighting: false, 
        uses_mipmaps: true, 
        shine_damper: 10.0,
        reflectivity: 0.5, 
        atlas_props: AtlasProps(1),
        normal_map: Some("res/textures/normal_maps/boulderNormal.png"),
    }; 
    
    pub const PLAYER: Model = Model(ModelType::Player, "res/models/person.obj", "res/textures/playerTexture.png", &Models::COMMON_PROPS);
    pub const TREE: Model = Model(ModelType::Tree, "res/models/tree.obj", "res/textures/tree.png", &Models::COMMON_PROPS);
    pub const LOW_POLY_TREE: Model = Model(ModelType::LowPolyTree, "res/models/lowPolyTree.obj", "res/textures/lowPolyTree.png", &Models::COMMON_PROPS);
    pub const FERN: Model = Model(ModelType::Fern, "res/models/fern.obj", "res/textures/atlases/fern.png", &Models::FERN_PROPS);
    pub const GRASS: Model = Model(ModelType::Grass, "res/models/grassModel.obj", "res/textures/grassTexture.png", &Models::GRASS_PROPS);
    pub const FLOWERS: Model = Model(ModelType::Flowers, "res/models/grassModel.obj", "res/textures/flower.png", &Models::GRASS_PROPS);
    pub const CRATE: Model = Model(ModelType::Crate, "res/models/box.obj", "res/textures/box.png", &Models::COMMON_PROPS);
    pub const LAMP: Model = Model(ModelType::Lamp, "res/models/lamp.obj", "res/textures/lamp.png", &Models::LAMP_PROPS);
    pub const TOON_ROCKS: Model = Model(ModelType::ToonRocks, "res/models/toonRocks.obj", "res/textures/toonRocks.png", &Models::SHINY_PROPS);
    pub const BOBBLE_TREE: Model = Model(ModelType::BobbleTree, "res/models/bobbleTree.obj", "res/textures/bobbleTree.png", &Models::COMMON_PROPS);
    pub const BARREL: Model = Model(ModelType::Barrel, "res/models/barrel.obj", "res/textures/barrel.png", &Models::BARREL_PROPS);
    pub const BOULDER: Model = Model(ModelType::Boulder, "res/models/boulder.obj", "res/textures/boulder.png", &Models::BOULDER_PROPS);
}


impl ResourceManager {

    pub const HEALTHBAR_TEXTURE: &'static str = "res/textures/health.png";
    pub const GUI_BACKGROUND_TEXTURE: &'static str = "res/textures/gui_background.png";
        
    pub const COPPER_SDF_FONT_TYPE: &'static str = "res/fonts/copperDf";

    pub const PARTICLE_STAR: ParticleTextureProps = ("res/textures/particles/particleStar.png", 1);
    pub const PARTICLE_ATLAS: ParticleTextureProps = ("res/textures/particles/particleAtlas.png", 4);
    pub const SMOKE_ATLAS: ParticleTextureProps = ("res/textures/particles/smoke.png", 8);
    pub const FIRE_ATLAS: ParticleTextureProps = ("res/textures/particles/fire.png", 8);
    
    pub fn init(&mut self, Model(model_type, obj_file, texture_file, model_props): &Model) {
        // thread safe coz only one mutable reference to resource manager can be held
        if self.models.contains_key(model_type) {
            return;
        }
        
        let (raw_model, normal_map) = if let Some(normal_map_texture) = model_props.normal_map {
            let model_data = load_obj_model(obj_file, true).expect(&format!("Unable to load {}", obj_file));
            let normal_map = self.loader.load_texture(normal_map_texture, TextureParams::default());
            let raw_model = self.loader.load_to_vao_with_normal_map(&model_data.vertices, &model_data.texture_coords, &model_data.indices, &model_data.normals, &model_data.tangents);
            (raw_model, Some(normal_map.tex_id))
        } else {
            let model_data = load_simple_obj_model(obj_file).expect(&format!("Unable to load simple {}", obj_file));
            let raw_model = self.loader.load_to_vao(&model_data.vertices, &model_data.texture_coords, &model_data.indices, &model_data.normals);
            (raw_model, None)
        }; 
        
        let mut texture = self.loader.load_texture(texture_file, model_props.get_texture_params());
        texture.has_transparency = model_props.has_transparency;
        texture.uses_fake_lighting = model_props.uses_fake_lighting;
        texture.shine_damper = model_props.shine_damper;
        texture.reflectivity = model_props.reflectivity;
        texture.number_of_rows_in_atlas = model_props.atlas_props.0;
        let model = TexturedModel { raw_model, texture, normal_map_tex_id: normal_map };

        self.models.insert(model_type.clone(), model);
    }

    pub fn model(&self, model_type: ModelType) -> TexturedModel {
        self.models.get(&model_type).expect(&format!("Need to call init_model({:?}) before accessing the model", model_type)).clone()
    }
    
    pub fn init_terrain_textures(&mut self) {        
        if let None = self.texture_pack {
            let background_texture = self.loader.load_terrain_texture("res/textures/terrain/grassy2.png", TextureParams::mipmapped_texture(-0.4));
            let r_texture = self.loader.load_terrain_texture("res/textures/terrain/mud.png", TextureParams::mipmapped_texture(-0.4));
            let g_texture = self.loader.load_terrain_texture("res/textures/terrain/grassFlowers.png", TextureParams::mipmapped_texture(-0.4));
            let b_texture = self.loader.load_terrain_texture("res/textures/terrain/path.png", TextureParams::mipmapped_texture(-0.4));
            self.texture_pack = Some(TerrainTexturePack { background_texture, r_texture, g_texture, b_texture, });
        }
        if let None = self.blend_texture {
            self.blend_texture = Some(self.loader.load_terrain_texture("res/textures/terrain/blendMap.png", TextureParams::mipmapped_texture(-0.4)));
        }
    }

    pub fn terrain_pack(&self) -> TerrainTexturePack {
        self.texture_pack.clone().expect("Need to call init_terrain_textures before accessing the textures")
    }

    pub fn blend_texture(&self) -> TerrainTexture {
        self.blend_texture.clone().expect("Need to call init_terrain_textures before accessing the textures")
    }

    pub fn init_terrain_model(&mut self) {
        if let None = self.terrain_model {
            let model = Terrain::generate_terrain(&mut self.loader, &self.terrain_generator);
            self.terrain_model = Some(model);
        }
    }

    pub fn terrain_model(&self) -> TerrainModel {
        self.terrain_model.clone().expect("Need to call init_terrain_model before accessing the model")
    }

    pub fn init_gui_textures(&mut self) {        
        let props = Models::GUI_PROPS;
        if !self.gui_textures.contains_key(ResourceManager::HEALTHBAR_TEXTURE) {
            let texture_id = self.loader.load_gui_texture(ResourceManager::HEALTHBAR_TEXTURE, props.get_texture_params());
            self.gui_textures.insert(ResourceManager::HEALTHBAR_TEXTURE, texture_id);
        }

        if !self.gui_textures.contains_key(ResourceManager::GUI_BACKGROUND_TEXTURE) {
            let texture_id = self.loader.load_gui_texture(ResourceManager::GUI_BACKGROUND_TEXTURE, props.get_texture_params());
            self.gui_textures.insert(ResourceManager::GUI_BACKGROUND_TEXTURE, texture_id);
        }
    }

    pub fn get_gui_texture(&self, texture_name: &str) -> u32 {
         let tex_id = self.gui_textures.get(texture_name).expect("Must call init_gui_textures first");
         *tex_id
    }

    pub fn init_gui_model(&mut self) {
        if let None = self.gui_model {
            // create quad that covers full screen -> we will scale it to create guis
            let positions = vec!{
                -1.0, 1.0,
                -1.0, -1.0,
                1.0, 1.0,
                1.0, -1.0,
            };
            let raw_model = self.loader.load_simple_model_to_vao(&positions, 2);
            self.gui_model = Some(GuiModel {
                raw_model,
            });
        }
    }

    pub fn gui_model(&self) -> GuiModel {
        self.gui_model.clone().expect("Need to call init_gui_model before accessing gui model")
    }

    pub fn skybox(&self) -> SkyboxModel {
        self.skybox_model.clone().expect("Need to call init_skybox first")
    }

    pub fn init_skybox(&mut self) {
        if let None = self.skybox_model {
            let day_texture_id = self.loader.load_cube_map("res/textures/cube_maps/day_skybox");
            let night_texture_id = self.loader.load_cube_map("res/textures/cube_maps/night_skybox");
            
            const SIZE: f32 = 500.0;
            let positions = vec![
                -SIZE,  SIZE, -SIZE,
                -SIZE, -SIZE, -SIZE,
                SIZE, -SIZE, -SIZE,
                SIZE, -SIZE, -SIZE,
                SIZE,  SIZE, -SIZE,
                -SIZE,  SIZE, -SIZE,

                -SIZE, -SIZE,  SIZE,
                -SIZE, -SIZE, -SIZE,
                -SIZE,  SIZE, -SIZE,
                -SIZE,  SIZE, -SIZE,
                -SIZE,  SIZE,  SIZE,
                -SIZE, -SIZE,  SIZE,

                SIZE, -SIZE, -SIZE,
                SIZE, -SIZE,  SIZE,
                SIZE,  SIZE,  SIZE,
                SIZE,  SIZE,  SIZE,
                SIZE,  SIZE, -SIZE,
                SIZE, -SIZE, -SIZE,

                -SIZE, -SIZE,  SIZE,
                -SIZE,  SIZE,  SIZE,
                SIZE,  SIZE,  SIZE,
                SIZE,  SIZE,  SIZE,
                SIZE, -SIZE,  SIZE,
                -SIZE, -SIZE,  SIZE,

                -SIZE,  SIZE, -SIZE,
                SIZE,  SIZE, -SIZE,
                SIZE,  SIZE,  SIZE,
                SIZE,  SIZE,  SIZE,
                -SIZE,  SIZE,  SIZE,
                -SIZE,  SIZE, -SIZE,

                -SIZE, -SIZE, -SIZE,
                -SIZE, -SIZE,  SIZE,
                SIZE, -SIZE, -SIZE,
                SIZE, -SIZE, -SIZE,
                -SIZE, -SIZE,  SIZE,
                SIZE, -SIZE,  SIZE
            ];
            let raw_model = self.loader.load_simple_model_to_vao(&positions, 3);
            self.skybox_model = Some(SkyboxModel {
                raw_model,
                day_texture_id,
                night_texture_id,
            });
        }
    }

    pub fn init_water(&mut self) {
        if let None = self.water_model {
            let positions = vec![
                -1.0, 0.0, 1.0, 
                1.0, 0.0, 1.0, 
                -1.0, 0.0, -1.0, 
                1.0, 0.0, -1.0, 
            ];
            let raw_model = self.loader.load_simple_model_to_vao(&positions, 3);
            let dudv_tex_id = self.loader.load_terrain_texture("res/textures/water/waterDUDV.png", TextureParams::default()).tex_id;
            let normal_map_tex_id = self.loader.load_terrain_texture("res/textures/water/normalMap.png", TextureParams::default()).tex_id;
            self.water_model = Some(WaterModel {
                raw_model,
                dudv_tex_id,
                normal_map_tex_id,
            });
        }
    }

    pub fn water_model(&self) -> WaterModel {
        self.water_model.clone().expect("Need to call init_water first")
    }


    pub fn init_fonts(&mut self) {
        let fonts = vec![ResourceManager::COPPER_SDF_FONT_TYPE];

        for font in fonts.iter() {
            if !self.font_types.contains_key(font) {

                let fnt_file_name = format!("{}.{}", font, "fnt");
                let fnt_texture_atlas_name = format!("{}.{}", font, "png");

                let texture_id = self.loader.load_gui_texture(&fnt_texture_atlas_name, TextureParams::default());
                let font_type = FontType::new(&fnt_file_name, texture_id);

                self.font_types.insert(font, font_type);
            }
        }
    }

    pub fn get_font(&self, font_name: &str) -> FontType {
        self.font_types.get(font_name).expect("Must init fonts before accessing font types").clone()
    }

    pub fn create_gui_text(&mut self, text: &str, font_name: &str, font_size: usize, position: Vector2f, material: TextMaterial) -> GuiText {
        let font_type = self.get_font(font_name);
        let text_mesh = create_mesh(text, &font_type, font_size);
        let text_model = self.loader.load_quads_mesh_to_vao(&text_mesh.positions, &text_mesh.tex_coords);
        GuiText::new(font_type, text_model, position, material)
    }

    pub fn init_particle_model(&mut self) {
        if let None = self.particle_model {
            let quad_triang_strip = vec![
                -0.5, 0.5,
                -0.5, -0.5,
                0.5, 0.5,
                0.5, -0.5,
            ];
            let raw_model = self.loader.load_simple_model_to_vao(&quad_triang_strip, 2);
            let stream_draw_vbo = self.loader.create_empty_float_vbo(ParticleModel::INSTANCED_DATA_LENGTH * ParticleModel::MAX_INSTANCES);
            self.loader.add_instanced_attrib(raw_model.vao_id, stream_draw_vbo, ParticleModel::MODELVIEW_COLUMN1, 4, ParticleModel::INSTANCED_DATA_LENGTH, 0);
            self.loader.add_instanced_attrib(raw_model.vao_id, stream_draw_vbo, ParticleModel::MODELVIEW_COLUMN2, 4, ParticleModel::INSTANCED_DATA_LENGTH, 4);
            self.loader.add_instanced_attrib(raw_model.vao_id, stream_draw_vbo, ParticleModel::MODELVIEW_COLUMN3, 4, ParticleModel::INSTANCED_DATA_LENGTH, 8);
            self.loader.add_instanced_attrib(raw_model.vao_id, stream_draw_vbo, ParticleModel::MODELVIEW_COLUMN4, 4, ParticleModel::INSTANCED_DATA_LENGTH, 12);
            self.loader.add_instanced_attrib(raw_model.vao_id, stream_draw_vbo, ParticleModel::TEX_OFFSET, 4, ParticleModel::INSTANCED_DATA_LENGTH, 16);
            self.loader.add_instanced_attrib(raw_model.vao_id, stream_draw_vbo, ParticleModel::BLEND, 1, ParticleModel::INSTANCED_DATA_LENGTH, 20);
            self.particle_model = Some(ParticleModel {
                raw_model,
                stream_draw_vbo,
            });
        }
    }

    pub fn particle_model(&self) -> ParticleModel {
        self.particle_model.as_ref().expect("Must init_particle_model before accessing it").clone()
    }

    pub fn init_particle_textures(&mut self) {
        let texture_props = vec![ResourceManager::PARTICLE_ATLAS, ResourceManager::FIRE_ATLAS, ResourceManager::SMOKE_ATLAS];

        for texture_prop in texture_props.iter() {
            if !self.particle_textures.contains_key(texture_prop) {

                let mut particle_texture = self.loader.load_particle_texture(texture_prop.0, TextureParams::default());
                particle_texture.number_of_rows_in_atlas = texture_prop.1;
                
                self.particle_textures.insert(texture_prop.clone(), particle_texture);
            }
        }
    }

    pub fn particle_texture(&self, texture_prop: ParticleTextureProps) -> ParticleTexture {
        self.particle_textures.get(&texture_prop).expect("Must init_particle_textures before fetching").clone()
    }

    pub fn init_debug_cuboid_model(&mut self) {
        if let None = self.debug_model {
            let indices_cuboid = [
                0, 1, 2,
                0, 3, 2,
                0, 1, 5,
                0, 4, 5,
                0, 3, 7,
                0, 4, 7,
                6, 5, 4,
                6, 7, 4,
                6, 2, 3,
                6, 7, 3,
                6, 2, 1,
                6, 5, 1,
            ];
            // to draw a frustum which has a different order of corners
            // let indices = [
            //     2, 1, 6,
            //     2, 7, 6,
            //     2, 1, 0,
            //     2, 3, 0,
            //     2, 7, 5,
            //     2, 3, 5,
            //     4, 0, 3,
            //     4, 5, 3,
            //     4, 6, 7,
            //     4, 5, 7,
            //     4, 6, 1,
            //     4, 0, 1,
            // ];
            let model = self.loader.load_dynamic_model_with_indices_to_vao(8, &indices_cuboid, 3);
            self.debug_model = Some(model);
        }
    }

    pub fn debug_cuboid_model(&self) -> DynamicVertexIndexedModel {
        self.debug_model.clone().expect("Need to call init_debug_cuboid_model before accessing the model")
    }
}