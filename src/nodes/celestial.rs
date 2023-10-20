use ggez::graphics::Rect;

use crate::physics::fallingsand::element_directory::ElementGridDir;
use crate::physics::fallingsand::util::enums::MeshDrawMode;
use crate::physics::fallingsand::util::grid::{filter_vecgrid, Grid};
use crate::physics::fallingsand::util::image::RawImage;
use crate::physics::fallingsand::util::mesh::OwnedMeshData;
use crate::physics::fallingsand::util::vectors::{ChunkIjkVector, JkVector};
use crate::physics::util::clock::Clock;

use super::camera::Camera;

/// Acts as a cache for a radial mesh's meshes and textures
pub struct Celestial {
    element_grid_dir: ElementGridDir,
    draw_mode: MeshDrawMode,
    all_meshes: Vec<Grid<OwnedMeshData>>,
    all_textures: Vec<Grid<RawImage>>,
    bounding_boxes: Vec<Grid<Rect>>,
    combined_mesh: OwnedMeshData,
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
        };
        out.ready();
        out
    }

    /// Something to call only on MAJOR changes, not every frame
    fn ready(&mut self) {
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
    }

    /// Something to call every frame
    pub fn process(&mut self, current_time: Clock) {
        self.element_grid_dir.process(current_time);
        // self.update_textures();
    }

    pub fn set_draw_mode(&mut self, draw_mode: MeshDrawMode) {
        self.draw_mode = draw_mode;
        self.ready();
    }
    pub fn get_combined_mesh_texture(&self, camera: &Camera) -> (OwnedMeshData, RawImage) {
        let filter = self.frustum_cull(camera);
        let meshdata = filter_vecgrid(&self.all_meshes, &filter);
        (
            OwnedMeshData::combine(&meshdata),
            RawImage::combine(&self.element_grid_dir.get_textures_filtered(&filter)),
        )
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
}

impl Celestial {
    /// Produces a mask of which chunks are visible, true if visible, false if not
    fn frustum_cull(&self, camera: &Camera) -> Vec<Grid<bool>> {
        let cam_bb = &camera.get_bounding_box();
        let mut out =
            Vec::with_capacity(self.element_grid_dir.get_coordinate_dir().get_num_layers());
        for layer in self.get_all_bounding_boxes() {
            let vec_out = layer
                .iter()
                .map(|x| x.overlaps(cam_bb))
                .collect::<Vec<bool>>();
            out.push(Grid::new(layer.get_width(), layer.get_height(), vec_out));
        }
        out
    }

    /// Produce a mask of which chunks need to be processed
    fn filter_inactive(&self, current_time: Clock) -> Vec<Grid<bool>> {
        let coords = self.element_grid_dir.get_coordinate_dir();
        let mut out = Vec::with_capacity(coords.get_num_layers());
        for i in 0..coords.get_num_layers() {
            let size_j = coords.get_layer_num_concentric_chunks(i);
            let size_k = coords.get_layer_num_radial_chunks(i);
            let mut grid_out = Grid::new(size_k, size_j, vec![false; size_j * size_k]);
            for j in 0..size_j {
                for k in 0..size_k {
                    let chunk = self
                        .element_grid_dir
                        .get_chunk_by_chunk_ijk(ChunkIjkVector { i, j, k });
                    if chunk.get_last_set().get_current_frame()
                        > current_time.get_current_frame() - 1
                    {
                        grid_out.set(JkVector { j, k }, true);
                    }
                }
            }
            out.push(grid_out);
        }
        out
    }
}
