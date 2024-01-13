use bevy::asset::Handle;
use bevy::core::FrameCount;
use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;
use bevy::ecs::system::{Query, Res, ResMut};
use bevy::render::mesh::shape::Quad;
use bevy::render::mesh::Mesh;
use bevy::render::render_asset::RenderAsset;
use bevy::render::texture::Image;
use bevy::time::Time;
use hashbrown::HashMap;

use crate::physics::fallingsand::util::enums::MeshDrawMode;
use crate::physics::fallingsand::util::grid::Grid;
use crate::physics::fallingsand::util::image::RawImage;
use crate::physics::fallingsand::util::mesh::OwnedMeshData;
use crate::physics::fallingsand::util::vectors::ChunkIjkVector;
use crate::physics::util::clock::Clock;
use crate::physics::util::vectors::WorldCoord;
use crate::physics::{fallingsand::data::element_directory::ElementGridDir, util::vectors::Rect};

use super::super::node_trait::{NodeTrait, WorldDrawable};

/// Acts as a cache for a radial mesh's meshes and textures
#[derive(Component)]
pub struct CelestialData {
    pub element_grid_dir: ElementGridDir,
    pub all_textures: HashMap<ChunkIjkVector, RawImage>,
    pub bounding_boxes: Vec<Grid<Rect>>,
}

impl CelestialData {
    pub fn new(element_grid_dir: ElementGridDir) -> Self {
        // In testing we found that the resolution doesn't matter, so make it infinite
        // a misnomer is the fact that in this case, big "res" is fewer mesh cells
        Self {
            element_grid_dir,
            all_textures: HashMap::new(),
            bounding_boxes: Vec::new(),
        }
    }

    // /// Save the combined mesh and textures to a directory
    // /// As well as all the chunks
    // pub fn save(&self, ctx: &mut Context, dir_path: &str) -> Result<(), ggez::GameError> {
    //     let img = self.get_combined_mesh_texture().1.to_image(ctx);
    //     let combined_path = format!("{}/combined.png", dir_path);
    //     img.encode(ctx, ggez::graphics::ImageEncodingFormat::Png, combined_path)?;
    //     self.data.element_grid_dir.save(ctx, dir_path)
    // }

    pub fn calc_combined_mesh_outline(&self) -> OwnedMeshData {
        OwnedMeshData::combine(
            &self
                .element_grid_dir
                .get_coordinate_dir()
                .get_mesh_data(MeshDrawMode::Outline),
        )
    }
    pub fn calc_combined_mesh_wireframe(&self) -> OwnedMeshData {
        OwnedMeshData::combine(
            &self
                .element_grid_dir
                .get_coordinate_dir()
                .get_mesh_data(MeshDrawMode::TriangleWireframe),
        )
    }
    /// Only recalculates the mesh for the combined mesh, not the texture
    pub fn calc_combined_mesh(&self) -> OwnedMeshData {
        OwnedMeshData::combine(
            &self
                .element_grid_dir
                .get_coordinate_dir()
                .get_mesh_data(MeshDrawMode::TexturedMesh),
        )
    }
    /// Only recalculates the texture for the combined mesh, not the mesh itself
    pub fn calc_combined_mesh_texture(&self, mesh: &OwnedMeshData) -> RawImage {
        RawImage::combine(self.all_textures.clone(), mesh.uv_bounds.clone())
    }

    /// Something to call every frame
    pub fn process(&mut self, current_time: Clock) {
        self.element_grid_dir.process(current_time);
        self.all_textures
            .extend(self.element_grid_dir.get_updated_target_textures());
        // self.all_textures = self.element_grid_dir.get_textures();
    }

    pub fn process_full(&mut self, current_time: Clock) {
        self.element_grid_dir.process_full(current_time);
        self.all_textures
            .extend(self.element_grid_dir.get_updated_target_textures());
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
    pub fn get_element_dir_mut(&mut self) -> &mut ElementGridDir {
        &mut self.element_grid_dir
    }
}

#[derive(Bundle)]
pub struct Celestial {
    pub world_coord: WorldCoord,
    pub data: CelestialData,
}

impl Celestial {
    pub fn new(element_grid_dir: ElementGridDir) -> Self {
        Self {
            world_coord: WorldCoord::default(),
            data: CelestialData::new(element_grid_dir),
        }
    }
}

impl NodeTrait for Celestial {
    fn get_world_coord(&self) -> WorldCoord {
        self.world_coord
    }
    fn set_world_coord(&mut self, world_coord: WorldCoord) {
        self.world_coord = world_coord;
    }
}

/// Bevy Systems
impl Celestial {
    pub fn process_system(
        mut celestial: Query<&mut CelestialData>,
        time: Res<Time>,
        frame: Res<FrameCount>,
    ) {
        for mut celestial in celestial.iter_mut() {
            celestial.process(Clock::new(time.as_generic(), frame.as_ref().to_owned()));
        }
    }
}
