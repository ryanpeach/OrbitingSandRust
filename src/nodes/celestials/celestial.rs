use bevy::asset::{AssetServer, Assets, Handle};
use bevy::core::FrameCount;
use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::With;
use bevy::ecs::system::{Commands, Query, Res, ResMut};

use bevy::log::{debug, trace, warn};
use bevy::render::texture::Image;
use bevy::sprite::ColorMaterial;
use bevy::time::Time;

use hashbrown::HashMap;

use crate::physics::fallingsand::util::enums::MeshDrawMode;

use crate::physics::fallingsand::data::element_directory::ElementGridDir;
use crate::physics::fallingsand::util::image::{self, RawImage};
use crate::physics::fallingsand::util::mesh::OwnedMeshData;
use crate::physics::fallingsand::util::vectors::ChunkIjkVector;
use crate::physics::util::clock::Clock;
use crate::physics::util::vectors::WorldCoord;

/// Acts as a cache for a radial mesh's meshes and textures
#[derive(Component)]
pub struct CelestialData {
    pub element_grid_dir: ElementGridDir,
    pub combined_mesh: OwnedMeshData,
    pub all_textures: HashMap<ChunkIjkVector, RawImage>,
}

impl CelestialData {
    pub fn new(element_grid_dir: ElementGridDir) -> Self {
        // In testing we found that the resolution doesn't matter, so make it infinite
        // a misnomer is the fact that in this case, big "res" is fewer mesh cells
        Self {
            combined_mesh: Self::calc_combined_mesh(&element_grid_dir),
            all_textures: element_grid_dir.get_textures(),
            element_grid_dir,
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
    fn calc_combined_mesh(element_grid_dir: &ElementGridDir) -> OwnedMeshData {
        OwnedMeshData::combine(
            &element_grid_dir
                .get_coordinate_dir()
                .get_mesh_data(MeshDrawMode::TexturedMesh),
        )
    }
    /// Retrieves the combined mesh
    pub fn get_combined_mesh(&self) -> &OwnedMeshData {
        &self.combined_mesh
    }
    /// Only recalculates the texture for the combined mesh, not the mesh itself
    pub fn calc_combined_mesh_texture(&self) -> RawImage {
        RawImage::combine(self.all_textures.clone(), self.combined_mesh.uv_bounds)
    }

    /// Something to call every frame
    /// This calculates only 1/9th of the grid each frame
    /// for maximum performance
    pub fn process(&mut self, current_time: Clock) {
        self.element_grid_dir.process(current_time);
        self.all_textures
            .extend(self.element_grid_dir.get_updated_target_textures());
    }

    /// Something to call every frame
    /// This is the same as process, but it processes the entire grid
    pub fn process_full(&mut self, current_time: Clock) {
        self.element_grid_dir.process_full(current_time);
        self.all_textures
            .extend(self.element_grid_dir.get_textures());
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

/// Bevy Systems
impl CelestialData {
    pub fn process_system(
        mut celestial: Query<&mut CelestialData>,
        time: Res<Time>,
        frame: Res<FrameCount>,
    ) {
        for (mut celestial) in celestial.iter_mut() {
            celestial.process(Clock::new(time.as_generic(), frame.as_ref().to_owned()));
        }
    }

    pub fn redraw_system(
        mut query: Query<(&mut Handle<ColorMaterial>, &CelestialData), With<CelestialData>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
        asset_server: Res<AssetServer>,
    ) {
        for (material_handle, celestial_data) in query.iter_mut() {
            let new_image: Image = celestial_data.calc_combined_mesh_texture().to_bevy_image();
            if let Some(material) = materials.get_mut(&*material_handle) {
                material.texture = Some(asset_server.add(new_image));
            }
        }
    }
}
