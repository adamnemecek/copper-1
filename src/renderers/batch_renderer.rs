use std::collections::HashMap;
use crate::display::Display;
use crate::gl;
use crate::entities::{
    Entity,
    Camera,
    Light,
    Terrain,
};
use crate::math::Matrix4f;
use crate::loader::{
    TexturedModel,    
};
use super::entity_renderer::EntityRenderer;
use super::terrain_renderer::TerrainRenderer;

pub struct BatchRenderer {    
    projection_matrix: Matrix4f,
    entity_renderer: EntityRenderer,
    terrain_renderer: TerrainRenderer,
}

impl BatchRenderer {

    const FOV_HORIZONTAL: f32 = 70.0;
    // here using actual world coords which are RHS coord sys with z axis going into screen (so more negative means further)
    const NEAR: f32 = -0.1;
    const FAR: f32 = -1000.0;

    pub fn new(display: &Display) -> BatchRenderer {
        let projection_matrix = Matrix4f::create_projection_matrix(BatchRenderer::NEAR, BatchRenderer::FAR, BatchRenderer::FOV_HORIZONTAL, display.get_aspect_ration());
        let entity_renderer = EntityRenderer::new(&projection_matrix);
        let terrain_renderer = TerrainRenderer::new(&projection_matrix);
        
        BatchRenderer {
            projection_matrix,
            entity_renderer,
            terrain_renderer,
        }
    }
    
    pub fn render<'a, 'b>(&mut self, light: &Light, camera: &Camera, entities: &Vec<Entity<'a>>, terrains: &Vec<Terrain<'b>>) {

        self.prepare();

        // render entites
        self.entity_renderer.start_render(light, camera);
        let groups_by_tex = BatchRenderer::group_entities_by_tex(entities);
        for (textured_model, entity_vec) in groups_by_tex.iter() {
            self.entity_renderer.prepare_textured_model(textured_model);
            for entity in entity_vec {
                // load transform matrix into shader
                self.entity_renderer.render(entity);
            }
            self.entity_renderer.unprepare_textured_model();
        }        
        self.entity_renderer.stop_render();

        // render terrain
        self.terrain_renderer.start_render(light, camera);
        for terrain in terrains.iter() {
            self.terrain_renderer.prepare_terrain(terrain);
            self.terrain_renderer.render(terrain);
            self.terrain_renderer.unprepare_terrain();
        }
        self.terrain_renderer.stop_render();
    }
    
    fn prepare(&self) {
        gl::enable(gl::CULL_FACE);
        gl::cull_face(gl::BACK);
        gl::enable(gl::DEPTH_TEST);
        gl::clear_color((1.0, 0.0, 0.0, 1.0));
        gl::clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }

    fn group_entities_by_tex<'a, 'b>(entities: &'b Vec<Entity<'a>>) -> HashMap<&'b TexturedModel, Vec<&'b Entity<'a>>> {
        let mut groups_by_tex = HashMap::new();

        for entity in entities.iter() {
            let group = groups_by_tex.entry(entity.model).or_insert(Vec::new());
            group.push(entity);
        }

        groups_by_tex
    }
}