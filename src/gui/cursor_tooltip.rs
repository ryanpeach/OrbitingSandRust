use ggegui::{egui, Gui};
use ggez::{
    glam::Vec2,
    graphics::{Canvas, DrawParam, Drawable},
    Context,
};

use crate::{
    nodes::{
        camera::Camera,
        celestial::{self, Celestial},
    },
    physics::{fallingsand::element_directory::ElementGridDir, util::vectors::RelXyPoint},
};

pub struct CursorTooltip {
    world_coords: Vec2,
    screen_coords: Vec2,
    gui: Gui,
}

impl CursorTooltip {
    pub fn new(ctx: &Context) -> Self {
        Self {
            screen_coords: Vec2 { x: 0.0, y: 0.0 },
            world_coords: Vec2 { x: 0.0, y: 0.0 },
            gui: Gui::new(ctx),
        }
    }

    pub fn update(&mut self, ctx: &mut Context, camera: &Camera, celestial: &Celestial) {
        let gui_ctx = self.gui.ctx();
        let coordinate_dir = celestial.get_element_dir().get_coordinate_dir();
        let coords = {
            match coordinate_dir.rel_pos_to_cell_idx(RelXyPoint(self.world_coords)) {
                Ok(coords) => coords,
                Err(coords) => coords,
            }
        };
        egui::Window::new("Title").show(&gui_ctx, |ui| {
            ui.label(format!("zoom: {}", camera.get_zoom()));
            ui.label(format!(
                "position: ({}, {}, {})",
                coords.i, coords.j, coords.k
            ));
        });
        self.gui.update(ctx);
    }

    pub fn set_pos(&mut self, screen_coords: Vec2, camera: &Camera) {
        self.screen_coords = screen_coords;
        self.world_coords = camera.screen_to_world_coords(screen_coords);
    }

    pub fn draw(&self, canvas: &mut Canvas) {
        self.gui
            .draw(canvas, DrawParam::default().dest(self.screen_coords));
    }
}
