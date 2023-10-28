use ggegui::{egui, Gui};
use ggez::{
    graphics::{Canvas, DrawParam, Drawable},
    mint::Point2,
    Context,
};

use crate::{
    nodes::{camera::cam::Camera, celestial::Celestial},
    physics::util::vectors::RelXyPoint,
};

pub struct CursorTooltip {
    world_coords: Point2<f32>,
    screen_coords: Point2<f32>,
    gui: Gui,
}

impl CursorTooltip {
    pub fn new(ctx: &Context) -> Self {
        Self {
            screen_coords: Point2 { x: 0.0, y: 0.0 },
            world_coords: Point2 { x: 0.0, y: 0.0 },
            gui: Gui::new(ctx),
        }
    }

    pub fn update(&mut self, ctx: &mut Context, camera: &Camera, celestial: &Celestial) {
        let gui_ctx = self.gui.ctx();
        let coordinate_dir = celestial.get_element_dir().get_coordinate_dir();
        let coords = {
            match coordinate_dir.rel_pos_to_cell_idx(RelXyPoint(self.world_coords.into())) {
                Ok(coords) => coords,
                Err(coords) => coords,
            }
        };
        let chunk_coords = coordinate_dir.cell_idx_to_chunk_idx(coords);
        egui::Window::new("Title").show(&gui_ctx, |ui| {
            ui.label(format!("zoom: {:?}", camera.get_zoom()));
            ui.label(format!(
                "IjkCoord: ({}, {}, {})",
                coords.i, coords.j, coords.k
            ));
            ui.label(format!(
                "ChunkIjkCoord: ({}, {}, {})",
                chunk_coords.0.i, chunk_coords.0.j, chunk_coords.0.k
            ));
            ui.label(format!(
                "JkCoord: ({}, {})",
                chunk_coords.1.j, chunk_coords.1.k
            ));
            ui.label(format!(
                "Type: {:?}",
                celestial.get_element_dir().get_element(coords).get_type()
            ))
        });
        self.gui.update(ctx);
    }

    pub fn set_pos(&mut self, screen_coords: Point2<f32>, camera: &Camera) {
        self.screen_coords = screen_coords;
        self.world_coords = camera.screen_to_world_coords(screen_coords);
    }

    pub fn draw(&self, canvas: &mut Canvas) {
        self.gui
            .draw(canvas, DrawParam::default().dest(self.screen_coords));
    }
}
