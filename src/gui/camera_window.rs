use ggegui::{egui, Gui};
use ggez::{
    glam::Vec2,
    graphics::{Canvas, DrawParam},
    Context,
};

use crate::nodes::{camera::Camera, celestial::Celestial};

pub struct CameraWindow {
    outline: bool,
    wireframe: bool,
    queue_save: bool,
    path: String,
    gui: Gui,
}

impl CameraWindow {
    pub fn new(ctx: &Context) -> Self {
        // let pwd = std::env::current_dir().unwrap();
        // let pwdstr = pwd.to_str().unwrap();
        Self {
            outline: false,
            wireframe: false,
            queue_save: true,
            path: "".to_owned(),
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
            // Create a save button and a path selector
            ui.separator();
            if ui.button("Save").clicked() {
                self.queue_save = true;
            }
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

    pub fn save_optionally(&mut self, ctx: &mut Context, celestial: &Celestial) {
        if self.queue_save {
            self.queue_save = false;
            match celestial.save(ctx, &self.path) {
                Ok(_) => println!("Saved to '{}'", self.path),
                Err(e) => println!("Error saving to {}: {}", self.path, e),
            }
        }
    }
}
