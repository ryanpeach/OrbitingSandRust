use ggegui::{
    egui::{self, Ui},
    Gui,
};
use ggez::Context;
use mint::{Point2, Vector2};

use crate::nodes::{camera::cam::Camera, celestial::Celestial};

use super::gui_trait::WindowTrait;

pub struct CameraWindow {
    draw_coords: Point2<f32>,
    outline: bool,
    wireframe: bool,
    queue_save: bool,
    fps: f64,
    camera_zoom: Vector2<f32>,
    path: String,
    gui: Gui,
}

impl CameraWindow {
    pub fn new(ctx: &Context) -> Self {
        // let pwd = std::env::current_dir().unwrap();
        // let pwdstr = pwd.to_str().unwrap();
        Self {
            draw_coords: Point2 { x: 0.0, y: 0.0 },
            outline: false,
            wireframe: false,
            queue_save: true,
            fps: 0.0,
            camera_zoom: Vector2 { x: 1.0, y: 1.0 },
            path: "".to_owned(),
            gui: Gui::new(ctx),
        }
    }

    pub fn update(&mut self, ctx: &mut Context, camera: &Camera) {
        self.fps = ctx.time.fps();
        self.camera_zoom = camera.get_zoom();
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

impl WindowTrait for CameraWindow {
    fn get_offset(&self) -> Point2<f32> {
        self.draw_coords
    }

    fn set_offset(&mut self, screen_coords: Point2<f32>) {
        self.draw_coords = screen_coords;
    }

    fn get_gui(&self) -> &Gui {
        &self.gui
    }

    fn get_gui_mut(&mut self) -> &mut Gui {
        &mut self.gui
    }

    fn get_alignment(&self) -> egui::Align2 {
        egui::Align2::LEFT_TOP
    }

    fn get_title(&self) -> &str {
        "Camera"
    }

    fn window(&mut self, ui: &mut Ui) {
        ui.label(format!("zoom: {:?}", self.camera_zoom));
        ui.label(format!("FPS: {}", self.fps));
        // Set a radiomode for "DrawMode"
        ui.separator();
        ui.checkbox(&mut self.outline, "Outline");
        ui.checkbox(&mut self.wireframe, "Wireframe");
        // Create a save button and a path selector
        ui.separator();
        if ui.button("Save").clicked() {
            self.queue_save = true;
        }
    }
}
