use ggegui::{egui, Gui};
use ggez::{
    glam::Vec2,
    graphics::{Canvas, DrawParam},
    Context,
};

use crate::nodes::camera::Camera;

pub struct CameraWindow {
    outline: bool,
    wireframe: bool,
    gui: Gui,
}

impl CameraWindow {
    pub fn new(ctx: &Context) -> Self {
        Self {
            outline: false,
            wireframe: false,
            gui: Gui::new(ctx),
        }
    }

    pub fn update(&mut self, ctx: &mut Context, camera: &Camera) {
        let gui_ctx = self.gui.ctx();
        egui::Window::new("Title").show(&gui_ctx, |ui| {
            ui.label(format!("zoom: {}", camera.get_zoom()));
            ui.label(format!("FPS: {}", ctx.time.fps()));
            // Set a radiomode for "DrawMode"
            ui.separator();
            ui.checkbox(&mut self.outline, "Outline");
            ui.checkbox(&mut self.wireframe, "Wireframe");
        });
        self.gui.update(ctx);
    }

    pub fn draw(&self, canvas: &mut Canvas) {
        canvas.draw(&self.gui, DrawParam::default().dest(Vec2::ZERO));
    }

    pub fn get_outline(&self) -> bool {
        self.outline
    }

    pub fn get_wireframe(&self) -> bool {
        self.wireframe
    }
}
