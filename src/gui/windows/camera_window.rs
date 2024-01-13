use bevy::ecs::system::Resource;
use bevy_egui::egui::{self, Ui};
use glam::Vec2;

use crate::{
    gui::brush::{Brush, BrushRadius},
    nodes::{
        camera::cam::{Camera, CameraZoom, FPS},
        celestials::celestial::Celestial,
    },
    physics::util::vectors::ScreenCoord,
};

use super::window_trait::WindowTrait;

#[derive(Debug, Clone, Copy)]
pub enum PlayPauseMode {
    Play,
    Step,
    MicroStep,
    Pause,
}

pub enum YesNoFullStep {
    Yes,
    No,
    FullStep,
}

#[derive(Resource)]
pub struct CameraWindow {
    draw_coords: ScreenCoord,
    brush_size: BrushRadius,
    outline: bool,
    wireframe: bool,
    queue_save: bool,
    fps: FPS,
    play_pause: PlayPauseMode,
    camera_zoom: CameraZoom,
    path: String,
}

impl CameraWindow {
    pub fn new() -> Self {
        // let pwd = std::env::current_dir().unwrap();
        // let pwdstr = pwd.to_str().unwrap();
        Self {
            draw_coords: ScreenCoord(Vec2 { x: 0.0, y: 0.0 }),
            outline: false,
            wireframe: false,
            queue_save: true,
            brush_size: BrushRadius(1.0),
            fps: FPS(0.0),
            play_pause: PlayPauseMode::Play,
            camera_zoom: CameraZoom(Vec2 { x: 1.0, y: 1.0 }),
            path: "".to_owned(),
        }
    }

    pub fn update(&mut self, fps: &FPS, camera: &Camera, brush: &Brush) {
        self.fps = fps.clone();
        self.camera_zoom = camera.get_zoom();
        self.brush_size = brush.get_radius();
    }

    pub fn get_outline(&self) -> bool {
        self.outline
    }

    pub fn get_wireframe(&self) -> bool {
        self.wireframe
    }

    pub fn get_play_pause(&self) -> PlayPauseMode {
        self.play_pause
    }

    pub fn set_play_pause(&mut self, play_pause: PlayPauseMode) {
        self.play_pause = play_pause;
    }

    pub fn should_i_process(&mut self) -> YesNoFullStep {
        match self.play_pause {
            PlayPauseMode::Play => YesNoFullStep::Yes,
            PlayPauseMode::Pause => YesNoFullStep::No,
            PlayPauseMode::MicroStep => {
                self.play_pause = PlayPauseMode::Pause;
                YesNoFullStep::No
            }
            PlayPauseMode::Step => YesNoFullStep::FullStep,
        }
    }

    // pub fn save_optionally(&mut self, ctx: &mut Context, celestial: &Celestial) {
    //     if self.queue_save {
    //         self.queue_save = false;
    //         match celestial.save(ctx, &self.path) {
    //             Ok(_) => println!("Saved to '{}'", self.path),
    //             Err(e) => println!("Error saving to {}: {}", self.path, e),
    //         }
    //     }
    // }
}

impl WindowTrait for CameraWindow {
    fn get_offset(&self) -> ScreenCoord {
        self.draw_coords
    }

    fn set_offset(&mut self, screen_coords: ScreenCoord) {
        self.draw_coords = screen_coords;
    }

    fn get_alignment(&self) -> egui::Align2 {
        egui::Align2::LEFT_TOP
    }

    fn get_title(&self) -> &str {
        "Camera"
    }

    fn window(&mut self, ui: &mut Ui) {
        ui.label(format!("Brush Size: {}", self.brush_size.0));
        ui.label(format!("Zoom: {:?}", self.camera_zoom));
        ui.label(format!("FPS: {}", self.fps.0));
        // Set a radiomode for "DrawMode"
        ui.separator();
        ui.checkbox(&mut self.outline, "Outline");
        ui.checkbox(&mut self.wireframe, "Wireframe");
        // Play Step MicroStep Pause
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("Play").clicked() {
                println!("Play Button Clicked");
                self.play_pause = PlayPauseMode::Play;
            }
            if ui.button("Step").clicked() {
                println!("Step Button Clicked");
                self.play_pause = PlayPauseMode::Step;
            }
            if ui.button("MicroStep").clicked() {
                println!("MicroStep Button Clicked");
                self.play_pause = PlayPauseMode::MicroStep;
            }
            if ui.button("Pause").clicked() {
                println!("Pause Button Clicked");
                self.play_pause = PlayPauseMode::Pause;
            }
        });
        // Create a save button and a path selector
        ui.separator();
        if ui.button("Save").clicked() {
            self.queue_save = true;
        }
    }
}
