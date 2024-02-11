use bevy::{
    app::{App, Plugin, Update},
    core_pipeline::core_2d::Camera2d,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    ecs::{
        entity::Entity,
        query::With,
        schedule::IntoSystemConfigs,
        system::{Local, Query, Res, ResMut, Resource},
    },
    render::view::{screenshot::ScreenshotManager, Visibility},
    transform::components::Transform,
    window::PrimaryWindow,
};
use bevy_egui::{
    egui::{self},
    EguiContexts,
};

use crate::{
    entities::celestials::celestial::{CelestialOutline, CelestialWireframe},
    physics::fallingsand::util::mesh::{GizmoDrawableLoop, GizmoDrawableTriangles},
};

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

#[derive(Resource, Debug, Clone, Copy)]
pub struct CameraWindowCheckboxes {
    pub outline: bool,
    pub wireframe: bool,
    pub material: bool,
    pub heat: bool,
}

impl Default for CameraWindowCheckboxes {
    fn default() -> Self {
        Self {
            outline: false,
            wireframe: false,
            material: true,
            heat: false,
        }
    }
}

/// The Camera Window Plugin
pub struct CameraWindowPlugin;

impl Plugin for CameraWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::camera_window_system);
        app.init_resource::<CameraWindowCheckboxes>();
    }
}

impl CameraWindowPlugin {
    fn camera_window_system(
        mut contexts: EguiContexts,
        diagnostics: Res<DiagnosticsStore>,
        main_window: Query<Entity, With<PrimaryWindow>>,
        mut screenshot_manager: ResMut<ScreenshotManager>,
        mut screenshot_counter: Local<u32>,
        mut checkboxes: ResMut<CameraWindowCheckboxes>,
        camera_transform: Query<(&Transform, &Camera2d)>,
    ) {
        let fps = diagnostics
            .get(FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
            .unwrap_or(0.0);
        let scale = camera_transform.iter().next().unwrap().0.scale.z;
        egui::Window::new("Camera Window").show(contexts.ctx_mut(), |ui| {
            // ui.label(format!("Brush Size: {}", self.brush_size.0));
            ui.label(format!("Zoom: {:?}", scale));
            ui.label(format!("FPS: {}", fps));
            // Set a radiomode for "DrawMode"
            ui.separator();
            ui.checkbox(&mut checkboxes.outline, "Outline");
            ui.checkbox(&mut checkboxes.wireframe, "Wireframe");
            ui.checkbox(&mut checkboxes.material, "Material");
            ui.checkbox(&mut checkboxes.heat, "Heat");
            // TODO: Play Step MicroStep Pause
            // ui.separator();
            // ui.horizontal(|ui| {
            //     if ui.button("Play").clicked() {
            //         println!("Play Button Clicked");
            //         self.play_pause = PlayPauseMode::Play;
            //     }
            //     if ui.button("Step").clicked() {
            //         println!("Step Button Clicked");
            //         self.play_pause = PlayPauseMode::Step;
            //     }
            //     if ui.button("MicroStep").clicked() {
            //         println!("MicroStep Button Clicked");
            //         self.play_pause = PlayPauseMode::MicroStep;
            //     }
            //     if ui.button("Pause").clicked() {
            //         println!("Pause Button Clicked");
            //         self.play_pause = PlayPauseMode::Pause;
            //     }
            // });
            // Create a save button and a path selector
            ui.separator();
            if ui.button("Save").clicked() {
                // self.queue_save = true;
            }
            if ui.button("Screenshot").clicked() {
                // Create the ./save directory if it doesn't exist
                std::fs::create_dir_all("./save/screenshots").unwrap();
                let path = format!("./save/screenshots/screenshot-{}.png", *screenshot_counter);
                *screenshot_counter += 1;
                screenshot_manager
                    .save_screenshot_to_disk(main_window.single(), path)
                    .unwrap();
            }
        });
    }
}
