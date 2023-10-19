use std::time::Duration;

use ggez::graphics::Rect;

use crate::physics::fallingsand::element_directory::ElementGridDir;
use crate::physics::fallingsand::util::enums::MeshDrawMode;
use crate::physics::fallingsand::util::grid::Grid;
use crate::physics::fallingsand::util::image::RawImage;
use crate::physics::fallingsand::util::mesh::OwnedMeshData;

use super::camera::Camera;

/// Acts as a cache for a radial mesh's meshes and textures
pub struct Celestial {
    element_grid_dir: ElementGridDir,
    draw_mode: MeshDrawMode,
    all_meshes: Vec<Grid<OwnedMeshData>>,
    all_textures: Vec<Grid<RawImage>>,
    bounding_boxes: Vec<Grid<Rect>>,
    combined_mesh: OwnedMeshData,
    combined_texture: RawImage,
}

impl Celestial {
    pub fn new(element_grid_dir: ElementGridDir, draw_mode: MeshDrawMode) -> Self {
        // In testing we found that the resolution doesn't matter, so make it infinite
        // a misnomer is the fact that in this case, big "res" is fewer mesh cells
        let mut out = Self {
            element_grid_dir,
            draw_mode,
            all_meshes: Vec::new(),
            all_textures: Vec::new(),
            bounding_boxes: Vec::new(),
            combined_mesh: OwnedMeshData::default(),
            combined_texture: RawImage::default(),
        };
        out.update();
        out
    }
    pub fn update(&mut self) {
        let _res = 31;
        self.all_meshes = self
            .element_grid_dir
            .get_coordinate_dir()
            .get_mesh_data(self.draw_mode);
        self.all_textures = self.element_grid_dir.get_textures();
        self.bounding_boxes = self
            .element_grid_dir
            .get_coordinate_dir()
            .get_chunk_bounding_boxes();
        self.combined_mesh = OwnedMeshData::combine(self.get_all_meshes());
        self.combined_texture = RawImage::combine(self.get_all_textures());
    }
    pub fn process(&mut self, current_time: Duration) {
        self.element_grid_dir.process(current_time);
        // self.update_textures();
    }
    pub fn set_draw_mode(&mut self, draw_mode: MeshDrawMode) {
        self.draw_mode = draw_mode;
        self.update();
    }
    pub fn get_combined_mesh(&self) -> &OwnedMeshData {
        &self.combined_mesh
    }
    pub fn get_combined_texture(&self) -> &RawImage {
        &self.combined_texture
    }
    pub fn get_all_bounding_boxes(&self) -> &Vec<Grid<Rect>> {
        &self.bounding_boxes
    }
    pub fn get_all_meshes(&self) -> &Vec<Grid<OwnedMeshData>> {
        &self.all_meshes
    }
    pub fn get_all_textures(&self) -> &Vec<Grid<RawImage>> {
        &self.all_textures
    }
    /// Produces a mask of which chunks are visible, true if visible, false if not
    pub fn frustum_cull(&self, camera: &Camera) -> Vec<Grid<bool>> {
        let cam_bb = &camera.get_bounding_box();
        let mut out = Vec::new();
        for layer in self.get_all_bounding_boxes() {
            let vec_out = layer
                .iter()
                .map(|x| x.overlaps(cam_bb))
                .collect::<Vec<bool>>();
            out.push(Grid::new(layer.get_width(), layer.get_height(), vec_out));
        }
        out
    }
}
