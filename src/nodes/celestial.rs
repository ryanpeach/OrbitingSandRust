use std::collections::{HashMap, HashSet};

use ggez::glam::Vec2;
use ggez::graphics::{self, Canvas, Image, ImageFormat, Mesh, Rect};
use ggez::Context;

use crate::physics::fallingsand::element_directory::ElementGridDir;
use crate::physics::fallingsand::elements::element::Element;
use crate::physics::fallingsand::elements::sand::Sand;
use crate::physics::fallingsand::elements::vacuum::Vacuum;
use crate::physics::fallingsand::util::enums::MeshDrawMode;
use crate::physics::fallingsand::util::grid::Grid;
use crate::physics::fallingsand::util::mesh::{OwnedMeshData, Square};
use crate::physics::fallingsand::util::vectors::{ChunkIjkVector, JkVector};
use crate::physics::util::clock::Clock;

use super::camera::Camera;

/// Acts as a cache for a radial mesh's meshes and textures
pub struct Celestial {
    element_grid_dir: ElementGridDir,
    draw_mode: MeshDrawMode,
    all_positions: HashMap<ChunkIjkVector, Vec<Square>>,
    all_uvs: HashMap<ChunkIjkVector, Vec<Square>>,
    bounding_boxes: Vec<Grid<Rect>>,
    texture: Image,
}

impl Celestial {
    pub fn new(
        ctx: &mut Context,
        element_grid_dir: ElementGridDir,
        draw_mode: MeshDrawMode,
    ) -> Self {
        // In testing we found that the resolution doesn't matter, so make it infinite
        // a misnomer is the fact that in this case, big "res" is fewer mesh cells
        let mut out = Self {
            element_grid_dir,
            draw_mode,
            all_positions: HashMap::new(),
            all_uvs: HashMap::new(),
            bounding_boxes: Vec::new(),
            texture: Celestial::create_texture(ctx),
        };
        out.ready();
        out
    }

    pub fn create_texture(ctx: &mut Context) -> Image {
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
        let pixels = element_lst
            .into_iter()
            .flat_map(|x| {
                let rgba = x.1.to_rgba();
                vec![rgba.0, rgba.1, rgba.2, rgba.3]
            })
            .collect::<Vec<u8>>();
        Image::from_pixels(
            ctx,
            &pixels[..],
            ImageFormat::Rgba8Unorm,
            pixels.len() as u32,
            1,
        )
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
        match self.draw_mode {
            MeshDrawMode::TexturedMesh => {
                canvas.draw_textured_mesh(mesh, self.texture, draw_params)
            }
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
    pub fn get_all_bounding_boxes(&self) -> &Vec<Grid<Rect>> {
        &self.bounding_boxes
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
