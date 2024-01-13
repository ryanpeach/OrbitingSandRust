use bevy::ecs::component::Component;
use ggez::graphics::{Canvas, DrawParam, Mesh};

use crate::{
    nodes::camera::cam::Camera,
    physics::{
        fallingsand::util::{image::RawImage, mesh::OwnedMeshData},
        util::vectors::ScreenCoord,
    },
};

#[derive(Component)]
pub struct ScreenDrawable {
    screen_coord: ScreenCoord,
    mesh: OwnedMeshData,
    texture: Option<RawImage>,
    enabled: bool,
}

impl Default for ScreenDrawable {
    fn default() -> Self {
        Self {
            screen_coord: ScreenCoord::default(),
            mesh: OwnedMeshData::default(),
            texture: None,
            enabled: true,
        }
    }
}

impl ScreenDrawable {
    pub fn new(screen_coord: ScreenCoord, mesh: OwnedMeshData, texture: Option<RawImage>) -> Self {
        Self {
            screen_coord,
            mesh,
            texture,
            enabled: true,
        }
    }
    pub fn get_screen_coord(&self) -> ScreenCoord {
        self.screen_coord
    }
    pub fn set_screen_coord(&mut self, screen_coord: ScreenCoord) {
        self.screen_coord = screen_coord;
    }
    pub fn get_mesh(&self) -> &OwnedMeshData {
        &self.mesh
    }
    pub fn get_mesh_mut(&mut self) -> &mut OwnedMeshData {
        &mut self.mesh
    }
    pub fn set_mesh(&mut self, mesh: OwnedMeshData) {
        self.mesh = mesh;
    }
    pub fn get_texture(&self) -> &Option<RawImage> {
        &self.texture
    }
    pub fn get_texture_mut(&mut self) -> &mut Option<RawImage> {
        &mut self.texture
    }
    pub fn get_enabled(&self) -> bool {
        self.enabled
    }
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    pub fn draw(&mut self, ctx: &mut ggez::Context, canvas: &mut Canvas, camera: &Camera) {
        if !self.get_enabled() {
            return;
        }
        let mesh = Mesh::from_data(ctx, self.get_mesh_mut().to_mesh_data());
        let texture = self.get_texture_mut();
        if let Some(texture) = texture {
            canvas.draw_textured_mesh(mesh, texture.to_image(ctx), camera.as_draw_param());
        } else {
            let draw_param = DrawParam::new().dest(self.get_screen_coord());
            canvas.draw(&mesh, draw_param);
        }
    }
}
