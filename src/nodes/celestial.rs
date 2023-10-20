use hashbrown::{HashMap, HashSet};

use ggez::glam::Vec2;
use ggez::graphics::{self, Canvas, Image, ImageFormat, Mesh, Rect};
use ggez::Context;

use crate::physics::fallingsand::element_directory::ElementGridDir;
use crate::physics::fallingsand::elements::element::Element;
use crate::physics::fallingsand::elements::sand::Sand;
use crate::physics::fallingsand::elements::vacuum::Vacuum;
use crate::physics::fallingsand::util::enums::MeshDrawMode;
use crate::physics::fallingsand::util::mesh::{OwnedMeshData, Square};
use crate::physics::fallingsand::util::vectors::ChunkIjkVector;
use crate::physics::util::clock::Clock;

use super::camera::Camera;

/// Acts as a cache for a radial mesh's meshes and textures
pub struct Celestial {
    element_grid_dir: ElementGridDir,
    draw_mode: MeshDrawMode,
    all_positions: HashMap<ChunkIjkVector, Vec<Square>>,
    all_uvs: HashMap<ChunkIjkVector, Vec<Square>>,
    bounding_boxes: HashMap<ChunkIjkVector, Rect>,
    texture: Vec<u8>,
}

impl Celestial {
    pub fn new(element_grid_dir: ElementGridDir, draw_mode: MeshDrawMode) -> Self {
        // In testing we found that the resolution doesn't matter, so make it infinite
        // a misnomer is the fact that in this case, big "res" is fewer mesh cells
        let mut out = Self {
            element_grid_dir,
            draw_mode,
            all_positions: HashMap::new(),
            all_uvs: HashMap::new(),
            bounding_boxes: HashMap::new(),
            texture: Celestial::create_texture(),
        };
        out.ready();
        out
    }

    pub fn create_texture() -> Vec<u8> {
        let mut element_lst = vec![
            (
                Vacuum::default().get_uv_index(),
                Vacuum::default().get_color(),
            ),
            (Sand::default().get_uv_index(), Sand::default().get_color()),
        ];
        // assert that no two elements have the same uv index
        if cfg!(debug_assertions) {
            let mut uv_indices = HashSet::new();
            for (uv_index, _) in &element_lst {
                assert!(!uv_indices.contains(uv_index));
                uv_indices.insert(uv_index);
            }
        }
        element_lst.sort_by(|a, b| a.0.cmp(&b.0));
        element_lst
            .into_iter()
            .flat_map(|x| {
                let rgba = x.1.to_rgba();
                vec![rgba.0, rgba.1, rgba.2, rgba.3]
            })
            .collect::<Vec<u8>>()
    }

    /// Something to call only on MAJOR changes, not every frame
    fn ready(&mut self) {
        let _res = 31;
        self.all_positions = self.element_grid_dir.get_coordinate_dir().get_positions();
        self.all_uvs = self.element_grid_dir.get_uvs();
        self.bounding_boxes = self
            .element_grid_dir
            .get_coordinate_dir()
            .get_chunk_bounding_boxes();
    }

    /// Something to call every frame
    pub fn process(&mut self, current_time: Clock) {
        self.element_grid_dir.process(current_time);
        self.all_uvs
            .extend(self.element_grid_dir.get_updated_target_uvs());
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

        let meshdata = self.get_mesh();
        let mesh: Mesh = Mesh::from_data(ctx, meshdata.to_mesh_data());
        let texture = Image::from_pixels(
            ctx,
            &self.texture,
            ImageFormat::Rgba8Unorm,
            self.texture.len() as u32 / 4,
            1,
        );
        match self.draw_mode {
            MeshDrawMode::TexturedMesh => canvas.draw_textured_mesh(mesh, texture, draw_params),
            MeshDrawMode::TriangleWireframe => canvas.draw(&mesh, draw_params),
            MeshDrawMode::UVWireframe => canvas.draw(&mesh, draw_params),
            MeshDrawMode::Outline => canvas.draw(&mesh, draw_params),
        }
    }
    pub fn set_draw_mode(&mut self, draw_mode: MeshDrawMode) {
        self.draw_mode = draw_mode;
        self.ready();
    }
    pub fn get_mesh(&self) -> OwnedMeshData {
        let mut all_positions = Vec::with_capacity(self.all_positions.len());
        let mut all_uvs = Vec::with_capacity(self.all_uvs.len());
        for (key, value) in &self.all_positions {
            all_positions.extend(value.clone());
            all_uvs.extend(self.all_uvs.get(key).unwrap().clone());
        }
        OwnedMeshData::new(all_positions, all_uvs)
    }
    pub fn get_all_bounding_boxes(&self) -> &HashMap<ChunkIjkVector, Rect> {
        &self.bounding_boxes
    }
}

impl Celestial {
    /// Produces a mask of which chunks are visible, true if visible, false if not
    fn frustum_cull(&self, camera: &Camera) -> HashSet<ChunkIjkVector> {
        let cam_bb = &camera.get_bounding_box();
        let mut out =
            HashSet::with_capacity(self.element_grid_dir.get_coordinate_dir().get_num_layers());
        for layer in self.get_all_bounding_boxes() {
            if cam_bb.overlaps(layer.1) {
                out.insert(layer.0.clone());
            }
        }
        out
    }
}
