use hashbrown::HashMap;

use ggez::glam::Vec2;
use ggez::graphics::{self, Canvas, Mesh, Rect};
use ggez::Context;

use crate::physics::fallingsand::element_directory::ElementGridDir;
use crate::physics::fallingsand::util::enums::MeshDrawMode;
use crate::physics::fallingsand::util::grid::Grid;
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
    all_textures: HashMap<ChunkIjkVector, RawImage>,
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
            all_textures: HashMap::new(),
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
        self.all_textures
            .extend(self.element_grid_dir.get_updated_target_textures());
        // self.all_textures = self.element_grid_dir.get_textures();
    }

    pub fn draw(&self, ctx: &mut Context, canvas: &mut Canvas, camera: &Camera) {
        // Draw planets
        let pos = camera.get_screen_coords();
        let zoom = camera.get_zoom();
        let draw_params = graphics::DrawParam::new()
            .dest(pos)
            .scale(Vec2::new(zoom, zoom))
            .rotation(camera.get_rotation())
            .offset(Vec2::new(0.5, 0.5));

        let (meshdata, rawimg) = self.get_combined_mesh_texture(camera);
        let mesh = Mesh::from_data(ctx, meshdata.to_mesh_data());
        let img = rawimg.to_image(ctx);
        match self.draw_mode {
            MeshDrawMode::TexturedMesh => canvas.draw_textured_mesh(mesh, img, draw_params),
            MeshDrawMode::TriangleWireframe => canvas.draw(&mesh, draw_params),
            MeshDrawMode::UVWireframe => canvas.draw(&mesh, draw_params),
            MeshDrawMode::Outline => canvas.draw(&mesh, draw_params),
        }
    }
    pub fn set_draw_mode(&mut self, draw_mode: MeshDrawMode) {
        self.draw_mode = draw_mode;
        self.ready();
    }
    pub fn get_combined_mesh_texture(&self, _camera: &Camera) -> (&OwnedMeshData, RawImage) {
        // let filter = self.frustum_cull(camera);
        (
            &self.combined_mesh,
            RawImage::combine(self.all_textures.clone(), self.combined_mesh.uv_bounds),
        )
    }
    pub fn get_all_bounding_boxes(&self) -> &Vec<Grid<Rect>> {
        &self.bounding_boxes
    }
    pub fn get_all_meshes(&self) -> &Vec<Grid<OwnedMeshData>> {
        &self.all_meshes
    }
    pub fn get_all_textures(&self) -> &HashMap<ChunkIjkVector, RawImage> {
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
}
