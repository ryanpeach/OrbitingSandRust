use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use hashbrown::HashMap;

use ggez::graphics::{Canvas, Mesh, Rect};
use ggez::Context;

use crate::physics::fallingsand::data::element_directory::ElementGridDir;
use crate::physics::fallingsand::util::enums::MeshDrawMode;
use crate::physics::fallingsand::util::grid::Grid;
use crate::physics::fallingsand::util::image::RawImage;
use crate::physics::fallingsand::util::mesh::OwnedMeshData;
use crate::physics::fallingsand::util::vectors::ChunkIjkVector;
use crate::physics::util::clock::Clock;
use crate::physics::util::vectors::WorldCoord;

use super::camera::cam::Camera;
use super::node_trait::{NodeTrait, WorldDrawable};

/// Acts as a cache for a radial mesh's meshes and textures
#[derive(Component)]
pub struct CelestialData {
    pub element_grid_dir: ElementGridDir,
    pub all_textures: HashMap<ChunkIjkVector, RawImage>,
    pub bounding_boxes: Vec<Grid<Rect>>,
}

#[derive(Bundle)]
pub struct Celestial {
    pub data: CelestialData,
    pub combined_mesh: WorldDrawable,
    pub combined_outline_mesh: WorldDrawable,
    pub combined_wireframe_mesh: WorldDrawable,
}

impl Celestial {
    pub fn new(element_grid_dir: ElementGridDir) -> Self {
        // In testing we found that the resolution doesn't matter, so make it infinite
        // a misnomer is the fact that in this case, big "res" is fewer mesh cells
        let mut out = Self {
            data: CelestialData {
                element_grid_dir,
                all_textures: HashMap::new(),
                bounding_boxes: Vec::new(),
            },
            combined_mesh: WorldDrawable::default(),
            combined_outline_mesh: WorldDrawable::default(),
            combined_wireframe_mesh: WorldDrawable::default(),
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
        self.data.element_grid_dir.save(ctx, dir_path)
    }

    /// Something to call only on MAJOR changes, not every frame
    fn ready(&mut self) {
        self.calc_combined_mesh();
        self.calc_combined_mesh_texture();
        self.calc_combined_mesh_outline();
        self.calc_combined_mesh_wireframe();
    }
    pub fn calc_combined_mesh_outline(&mut self) {
        self.combined_mesh.mesh = OwnedMeshData::combine(
            &self
                .data
                .element_grid_dir
                .get_coordinate_dir()
                .get_mesh_data(MeshDrawMode::Outline),
        );
    }
    pub fn calc_combined_mesh_wireframe(&mut self) {
        self.combined_mesh.mesh = OwnedMeshData::combine(
            &self
                .data
                .element_grid_dir
                .get_coordinate_dir()
                .get_mesh_data(MeshDrawMode::TriangleWireframe),
        );
    }
    /// Only recalculates the mesh for the combined mesh, not the texture
    pub fn calc_combined_mesh(&mut self) {
        self.combined_mesh.mesh = OwnedMeshData::combine(
            &self
                .data
                .element_grid_dir
                .get_coordinate_dir()
                .get_mesh_data(MeshDrawMode::TexturedMesh),
        );
    }
    /// Only recalculates the texture for the combined mesh, not the mesh itself
    pub fn calc_combined_mesh_texture(&self) {
        self.combined_mesh.texture = Some(RawImage::combine(
            self.data.all_textures.clone(),
            self.combined_mesh.mesh.uv_bounds,
        ));
    }

    /// Something to call every frame
    pub fn process(&mut self, current_time: Clock) {
        self.data.element_grid_dir.process(current_time);
        self.data
            .all_textures
            .extend(self.data.element_grid_dir.get_updated_target_textures());
        // self.data.all_textures = self.data.element_grid_dir.get_textures();
    }

    pub fn process_full(&mut self, current_time: Clock) {
        self.data.element_grid_dir.process_full(current_time);
        self.data
            .all_textures
            .extend(self.data.element_grid_dir.get_updated_target_textures());
    }
    pub fn get_combined_mesh_texture(&self) -> (&OwnedMeshData, &RawImage) {
        (
            &self.combined_mesh.mesh,
            self.combined_mesh.texture.as_ref().unwrap(),
        )
    }
    pub fn get_all_bounding_boxes(&self) -> &Vec<Grid<Rect>> {
        &self.data.bounding_boxes
    }
    pub fn get_all_textures(&self) -> &HashMap<ChunkIjkVector, RawImage> {
        &self.data.all_textures
    }
    pub fn get_element_dir(&self) -> &ElementGridDir {
        &self.data.element_grid_dir
    }
    pub fn get_element_dir_mut(&mut self) -> &mut ElementGridDir {
        &mut self.data.element_grid_dir
    }
}

// impl CelestialData {
//     /// Produces a mask of which chunks are visible, true if visible, false if not
//     fn frustum_cull(&self, camera: &Camera) -> Vec<Grid<bool>> {
//         let cam_bb = &camera.world_coord_bounding_box();
//         let mut out =
//             Vec::with_capacity(self.data.element_grid_dir.get_coordinate_dir().get_num_layers());
//         for layer in self.get_all_bounding_boxes() {
//             let vec_out = layer
//                 .iter()
//                 .map(|x| x.overlaps(cam_bb))
//                 .collect::<Vec<bool>>();
//             out.push(Grid::new(layer.get_width(), layer.get_height(), vec_out));
//         }
//         out
//     }
// }

impl NodeTrait for Celestial {
    fn get_world_coord(&self) -> WorldCoord {
        self.combined_mesh.get_world_coord()
    }
    fn set_world_coord(&mut self, world_coord: WorldCoord) {
        self.combined_mesh.set_world_coord(world_coord);
        self.combined_outline_mesh.set_world_coord(world_coord);
        self.combined_wireframe_mesh.set_world_coord(world_coord);
    }
}
