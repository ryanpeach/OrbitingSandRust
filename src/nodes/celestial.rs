use ggez::glam::Vec2;
use ggez::graphics::Rect;

use crate::physics::fallingsand::chunks::radial_mesh::RadialMesh;
use crate::physics::fallingsand::chunks::util::{MeshDrawMode, OwnedMeshData, RawImage};

use super::camera::Camera;

/// Acts as a cache for a radial mesh's meshes and textures
pub struct Celestial {
    radial_mesh: RadialMesh,
    draw_mode: MeshDrawMode,
    all_meshes: Vec<OwnedMeshData>,
    all_textures: Vec<RawImage>,
    bounding_boxes: Vec<Rect>,
    combined_mesh: OwnedMeshData,
    combined_texture: RawImage,
}

impl Celestial {
    pub fn new(radial_mesh: RadialMesh, draw_mode: MeshDrawMode) -> Self {
        // In testing we found that the resolution doesn't matter, so make it infinite
        // a misnomer is the fact that in this case, big "res" is fewer mesh cells
        let mut out = Self {
            radial_mesh,
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
        let res = 31;
        self.all_meshes = self.radial_mesh.get_mesh_data(res, self.draw_mode);
        self.all_textures = self.radial_mesh.get_textures(res);
        self.bounding_boxes = self.radial_mesh.get_chunk_bounding_boxes();
        self.combined_mesh = OwnedMeshData::combine(self.get_all_meshes());
        self.combined_texture = RawImage::combine(self.get_all_textures());
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
    pub fn frustum_cull(&self, camera: &Camera, screen_size: Vec2) -> Vec<usize> {
        let cam_bb = &camera.get_bounding_box(screen_size);
        let mut out = Vec::with_capacity(self.get_num_chunks());
        for (i, bb) in self.get_all_bounding_boxes().iter().enumerate() {
            if bb.overlaps(cam_bb) {
                out.push(i);
            }
        }
        out
    }
    pub fn update_chunk_texture(&mut self, chunk_idx: usize, image: RawImage) {
        self.all_textures[chunk_idx] = image
    }
}
