use ggez::graphics::Rect;

use crate::physics::fallingsand::chunks::radial_mesh::RadialMesh;
use crate::physics::fallingsand::chunks::util::{DrawMode, OwnedMeshData, RawImage};

/// Acts as a cache for a radial mesh's meshes and textures
pub struct Celestial {
    all_meshes: Vec<OwnedMeshData>,
    all_textures: Vec<RawImage>,
    bounding_boxes: Vec<Rect>,
}

impl Celestial {
    pub fn new(radial_mesh: &RadialMesh, draw_mode: DrawMode, res: u16) -> Self {
        let all_meshes = radial_mesh.get_mesh_data(res, draw_mode);
        let all_textures = radial_mesh.get_textures(res);
        let bounding_boxes = radial_mesh.get_chunk_bounding_boxes();
        Self {
            all_meshes,
            all_textures,
            bounding_boxes,
        }
    }
    pub fn get_num_chunks(&self) -> usize {
        self.all_meshes.len()
    }
    pub fn get_all_bounding_boxes(&self) -> &Vec<Rect> {
        &self.bounding_boxes
    }
    pub fn get_all_meshes(&self) -> &Vec<OwnedMeshData> {
        &self.all_meshes
    }
    pub fn get_all_textures(&self) -> &Vec<RawImage> {
        &self.all_textures
    }
    pub fn update_chunk_texture(&mut self, chunk_idx: usize, image: RawImage) {
        self.all_textures[chunk_idx] = image
    }
}
