use hashbrown::HashMap;

use ggez::graphics::{Canvas, Mesh, Rect};
use ggez::Context;

use crate::physics::fallingsand::element_directory::ElementGridDir;
use crate::physics::fallingsand::util::enums::MeshDrawMode;
use crate::physics::fallingsand::util::grid::Grid;
use crate::physics::fallingsand::util::image::RawImage;
use crate::physics::fallingsand::util::mesh::OwnedMeshData;
use crate::physics::fallingsand::util::vectors::ChunkIjkVector;
use crate::physics::util::clock::Clock;

use super::camera::cam::Camera;

/// Acts as a cache for a radial mesh's meshes and textures
pub struct Celestial {
    element_grid_dir: ElementGridDir,
    all_textures: HashMap<ChunkIjkVector, RawImage>,
    bounding_boxes: Vec<Grid<Rect>>,
    combined_mesh: OwnedMeshData,
    combined_outline_mesh: OwnedMeshData,
    combined_wireframe_mesh: OwnedMeshData,
}

impl Celestial {
    pub fn new(element_grid_dir: ElementGridDir) -> Self {
        // In testing we found that the resolution doesn't matter, so make it infinite
        // a misnomer is the fact that in this case, big "res" is fewer mesh cells
        let mut out = Self {
            element_grid_dir,
            all_textures: HashMap::new(),
            bounding_boxes: Vec::new(),
            combined_mesh: OwnedMeshData::default(),
            combined_outline_mesh: OwnedMeshData::default(),
            combined_wireframe_mesh: OwnedMeshData::default(),
        };
        out.ready();
        out
    }

    /// Save the combined mesh and textures to a directory
    /// As well as all the chunks
    pub fn save(&self, ctx: &mut Context, dir_path: &str) -> Result<(), ggez::GameError> {
        let img = self.get_combined_mesh_texture().1.to_image(ctx);
        let combined_path = format!("{}/combined.png", dir_path);
        img.encode(ctx, ggez::graphics::ImageEncodingFormat::Png, combined_path)?;
        self.element_grid_dir.save(ctx, dir_path)
    }

    /// Something to call only on MAJOR changes, not every frame
    fn ready(&mut self) {
        let _res = 31;
        let all_meshes = self
            .element_grid_dir
            .get_coordinate_dir()
            .get_mesh_data(MeshDrawMode::TexturedMesh);
        let all_outlines = self
            .element_grid_dir
            .get_coordinate_dir()
            .get_mesh_data(MeshDrawMode::Outline);
        let all_wireframes = self
            .element_grid_dir
            .get_coordinate_dir()
            .get_mesh_data(MeshDrawMode::TriangleWireframe);
        self.all_textures = self.element_grid_dir.get_textures();
        self.bounding_boxes = self
            .element_grid_dir
            .get_coordinate_dir()
            .get_chunk_bounding_boxes();
        self.combined_mesh = OwnedMeshData::combine(&all_meshes);
        self.combined_outline_mesh = OwnedMeshData::combine(&all_outlines);
        self.combined_wireframe_mesh = OwnedMeshData::combine(&all_wireframes);
    }

    /// Something to call every frame
    pub fn process(&mut self, current_time: Clock) {
        self.element_grid_dir.process(current_time);
        self.all_textures
            .extend(self.element_grid_dir.get_updated_target_textures());
        // self.all_textures = self.element_grid_dir.get_textures();
    }

    /// Draw the textures
    pub fn draw(&self, ctx: &mut Context, canvas: &mut Canvas, camera: Camera) {
        // Draw planets
        let (meshdata, rawimg) = self.get_combined_mesh_texture();
        let mesh = Mesh::from_data(ctx, meshdata.to_mesh_data());
        let img = rawimg.to_image(ctx);
        canvas.draw_textured_mesh(mesh, img, camera);
    }

    pub fn draw_outline(&self, ctx: &mut Context, canvas: &mut Canvas, camera: Camera) {
        // Draw planets
        let mesh = Mesh::from_data(ctx, self.combined_outline_mesh.to_mesh_data());
        canvas.draw(&mesh, camera);
    }

    pub fn draw_wireframe(&self, ctx: &mut Context, canvas: &mut Canvas, camera: Camera) {
        // Draw planets
        let mesh = Mesh::from_data(ctx, self.combined_wireframe_mesh.to_mesh_data());
        canvas.draw(&mesh, camera);
    }

    pub fn get_combined_mesh_texture(&self) -> (&OwnedMeshData, RawImage) {
        // let filter = self.frustum_cull(camera);
        (
            &self.combined_mesh,
            RawImage::combine(self.all_textures.clone(), self.combined_mesh.uv_bounds),
        )
    }
    pub fn get_all_bounding_boxes(&self) -> &Vec<Grid<Rect>> {
        &self.bounding_boxes
    }
    pub fn get_all_textures(&self) -> &HashMap<ChunkIjkVector, RawImage> {
        &self.all_textures
    }
    pub fn get_element_dir(&self) -> &ElementGridDir {
        &self.element_grid_dir
    }
}

impl Celestial {
    /// Produces a mask of which chunks are visible, true if visible, false if not
    fn frustum_cull(&self, camera: &Camera) -> Vec<Grid<bool>> {
        let cam_bb = &camera.world_coord_bounding_box();
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
