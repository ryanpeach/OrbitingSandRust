use bevy::{
    ecs::{bundle::Bundle, component::Component},
    pbr::StandardMaterial,
    render::{mesh::Mesh, render_resource::Texture, texture::Image},
    sprite::MaterialMesh2dBundle,
};

use crate::physics::{
    fallingsand::util::{image::RawImage, mesh::OwnedMeshData},
    util::vectors::WorldCoord,
};

pub trait NodeTrait {
    fn get_world_coord(&self) -> WorldCoord;
    fn set_world_coord(&mut self, world_coord: WorldCoord);
}

#[derive(Component)]
pub struct WorldDrawable {
    pub world_coord: WorldCoord,
    pub mesh: OwnedMeshData,
    pub texture: Option<RawImage>,
    pub enabled: bool,
}

impl Default for WorldDrawable {
    fn default() -> Self {
        Self {
            world_coord: WorldCoord::default(),
            mesh: OwnedMeshData::default(),
            texture: None,
            enabled: true,
        }
    }
}

impl NodeTrait for WorldDrawable {
    fn get_world_coord(&self) -> WorldCoord {
        self.world_coord
    }
    fn set_world_coord(&mut self, world_coord: WorldCoord) {
        self.world_coord = world_coord;
    }
}

impl WorldDrawable {
    fn create_bevy_mesh(&self) -> Mesh {
        todo!()
    }

    fn create_bevy_material(&self) -> StandardMaterial {
        todo!()
    }
}
