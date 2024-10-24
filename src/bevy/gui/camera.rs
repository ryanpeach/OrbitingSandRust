//! The bevy camera for the game

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use std::ops::Add;

use bevy::input::common_conditions::{input_just_pressed, input_pressed};
use bevy::log::{debug, error, trace_once};
use bevy::prelude::{Condition, IntoSystemConfigs, MouseButton};
use bevy::render::camera::OrthographicProjection;
use bevy::{
    app::{App, Plugin, Update},
    core_pipeline::core_2d::{Camera2d, Camera2dBundle},
    ecs::{
        component::Component,
        event::{Event, EventReader},
        query::With,
        system::{Commands, Query, Res},
    },
    hierarchy::{BuildChildren, Parent},
    input::{keyboard::KeyCode, mouse::MouseWheel, ButtonInput},
    math::Vec3,
    prelude::Entity,
    time::Time,
    transform::components::Transform,
};
use bevy_egui::EguiContexts;
use bevy_eventlistener::callbacks::ListenerInput;
use bevy_mod_picking::events::{Down, Pointer};

use conv::ValueFrom;
use macros::{call_log, call_log_once};

use crate::bevy::entities::celestials::celestial::ChunkIjkComponent;
use crate::bevy::errors::EmptyQueryResult;

/// Used to help identify our main camera
#[derive(Component)]
pub struct MainCamera;

/// A layer in front of the game. Z-index = 1
#[derive(Component, Debug, Default)]
pub struct OverlayLayer1;

/// A layer in front of the game. Z-index = 2
#[derive(Component, Debug, Default)]
pub struct OverlayLayer2;

/// A layer in front of the game. Z-index = 3
#[derive(Component, Debug, Default)]
pub struct OverlayLayer3;

/// A layer behind the game. Z-index = -1
#[derive(Component, Debug, Default)]
pub struct BackgroundLayer1;

/// A component that allows us to enumerate over all the celestials
#[derive(Component, Debug, Clone, Copy)]
pub struct CelestialIdx(pub u32);

impl Add<u32> for CelestialIdx {
    type Output = Self;

    fn add(self, rhs: u32) -> Self::Output {
        CelestialIdx(self.0 + rhs)
    }
}

impl CelestialIdx {
    /// Returns the selected celestials index
    pub fn selected_celestial(
        celestials: &[(Entity, &CelestialIdx)],
        camera: (Option<&Parent>, Entity),
    ) -> Result<CelestialIdx, EmptyQueryResult> {
        // Some debug checking
        if cfg!(debug_assertions) {
            let max_idx = celestials
                .iter()
                .map(|(_, idx)| idx.0)
                .max()
                .unwrap_or_default();
            let min_idx = celestials
                .iter()
                .map(|(_, idx)| idx.0)
                .min()
                .unwrap_or_default();
            if max_idx == min_idx {
                assert_eq!(max_idx, 0);
            }
            // Check all the indices are unique
            let mut indices = celestials.iter().map(|(_, idx)| idx.0).collect::<Vec<_>>();
            indices.sort_unstable();
            indices.dedup();
            assert_eq!(indices.len(), celestials.len());
            // Check that the indices start at 0 and end at len - 1
            let indices = indices.into_iter();
            for (idx, i) in indices.enumerate() {
                assert_eq!(i as usize, idx);
            }
        }

        if let Some(parent) = camera.0 {
            if let Some(celestial) = celestials.iter().find(|(entity, _)| *entity == **parent) {
                Ok(*celestial.1)
            } else {
                Err(EmptyQueryResult {
                    err: None,
                    parameter_name: "selected_celestial".to_string(),
                })
            }
        } else {
            Ok(CelestialIdx(0))
        }
    }

    /// Gets the next index
    #[must_use]
    pub fn next(&self, celestials: Vec<&CelestialIdx>) -> CelestialIdx {
        let mut idx = self.0 + 1;
        if idx as usize >= celestials.len() {
            idx = 0;
        }
        CelestialIdx(idx)
    }

    /// Gets the previous index
    #[must_use]
    pub fn prev(&self, celestials: Vec<&CelestialIdx>) -> CelestialIdx {
        if self.0 == 0 {
            CelestialIdx(
                u32::value_from(celestials.len() - 1).expect("Shouldn't be that many celestials"),
            )
        } else {
            CelestialIdx(self.0 - 1)
        }
    }
}

/// The plugin for the camera system
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    /// Build the camera plugin
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::zoom_camera_system);
        app.add_systems(
            Update,
            Self::move_camera_system.run_if(
                input_pressed(KeyCode::KeyW)
                    .or_else(input_pressed(KeyCode::KeyS))
                    .or_else(input_pressed(KeyCode::KeyA))
                    .or_else(input_pressed(KeyCode::KeyD)),
            ),
        );
        app.add_systems(
            Update,
            Self::select_celestial_focus.run_if(input_just_pressed(MouseButton::Left)),
        );
        app.add_systems(
            Update,
            Self::cycle_celestial_focus_up.run_if(input_just_pressed(KeyCode::BracketLeft)),
        );
        app.add_systems(
            Update,
            Self::cycle_celestial_focus_down.run_if(input_just_pressed(KeyCode::BracketRight)),
        );
    }
}

/// Startup functions
/// These are not systems, rather are written as function to be applied
/// to the `GuiUnifiedPlugin`
impl CameraPlugin {
    /// Setup the main camera
    pub fn setup_main_camera(commands: &mut Commands) -> Entity {
        commands
            .spawn((
                Camera2dBundle {
                    camera_2d: Camera2d,
                    ..Default::default()
                },
                MainCamera,
            ))
            .id()
    }
}

/// Update functions
impl CameraPlugin {
    /// Zoom the camera based on mouse wheel scroll
    #[call_log_once]
    fn zoom_camera_system(
        time: Res<Time>,
        mut scroll_evr: EventReader<MouseWheel>,
        mut query: Query<&mut OrthographicProjection, With<MainCamera>>,
        mut contexts: EguiContexts,
    ) {
        let mut delta = 0.;
        for ev in scroll_evr.read() {
            delta += ev.y;
        }
        if delta != 0. && !contexts.ctx_mut().is_pointer_over_area() {
            trace_once!("Zooming camera");
            for mut proj in &mut query {
                proj.scale *= 1. + delta * time.delta_seconds() * 0.5;
                proj.scale = proj.scale.max(0.05);
            }
        }
    }

    /// Move the camera based on keyboard input
    #[call_log]
    fn move_camera_system(
        time: Res<Time>,
        keyboard_input: Res<ButtonInput<KeyCode>>,
        mut query: Query<(&mut Transform, &mut Camera2d)>,
    ) {
        let mut delta = Vec3::default();
        if keyboard_input.pressed(KeyCode::KeyA) {
            delta.x -= 1.;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            delta.x += 1.;
        }
        if keyboard_input.pressed(KeyCode::KeyW) {
            delta.y += 1.;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            delta.y -= 1.;
        }
        if delta != Vec3::default() {
            for (mut transform, _) in &mut query {
                let scale = transform.scale;
                transform.translation += delta * time.delta_seconds() * scale * 100.;
            }
        }
    }
}

// /// Check if two rectangles overlap
// fn rect_overlaps(this: &Rect, other: &Rect) -> bool {
//     this.min.x < other.max.x
//         && this.max.x > other.min.x
//         && this.min.y < other.max.y
//         && this.max.y > other.min.y
// }
//
// /// Add a vector to every corner of a rectangle
// fn rect_add(this: &Rect, other: &Vec2) -> Rect {
//     Rect::new(
//         this.min.x + other.x,
//         this.min.y + other.y,
//         this.max.x + other.x,
//         this.max.y + other.y,
//     )
// }

/// Celestial Focus Systems
impl CameraPlugin {
    /// If you press "\[" or "\]", you can cycle through the celestials
    /// TODO: sysfail
    #[call_log]
    pub fn cycle_celestial_focus_up(
        mut commands: Commands,
        celestials: Query<(Entity, &CelestialIdx)>,
        mut camera: Query<(Option<&Parent>, Entity, &mut Transform), With<MainCamera>>,
    ) {
        // -> Result<(), Box<dyn std::error::Error>> {
        if let Ok((parent, camera, mut transform)) = camera.get_single_mut() {
            let celestials_vec = celestials.iter().collect::<Vec<_>>();
            let idx = match CelestialIdx::selected_celestial(&celestials_vec, (parent, camera)) {
                Ok(idx) => idx,
                Err(e) => {
                    error!("{:?}", e);
                    return;
                }
            };
            let next_idx = {
                idx.prev(
                    celestials_vec
                        .clone()
                        .into_iter()
                        .map(|(_, idx)| idx)
                        .collect::<Vec<_>>(),
                )
            };
            if let Some(next_celestial) = celestials_vec
                .into_iter()
                .find(|(_, idx)| idx.0 == next_idx.0)
            {
                focus_celestial(
                    &mut commands,
                    (&camera, &mut transform),
                    &next_celestial.0,
                    next_celestial.1,
                );
            } else {
                // Err(EmptyQueryResult{err: None, parameter_name: "next_celestial".to_string()})?;
                error!(
                    "{:?}",
                    EmptyQueryResult {
                        err: None,
                        parameter_name: "next_celestial".to_string()
                    }
                );
            }
        }
        // Ok(())
    }

    /// If you press "\[" or "\]", you can cycle through the celestials
    /// TODO: sysfail
    #[call_log]
    pub fn cycle_celestial_focus_down(
        mut commands: Commands,
        celestials: Query<(Entity, &CelestialIdx)>,
        mut camera: Query<(Option<&Parent>, Entity, &mut Transform), With<MainCamera>>,
    ) {
        // -> Result<(), Box<dyn std::error::Error>> {
        if let Ok((parent, camera, mut transform)) = camera.get_single_mut() {
            let celestials_vec = celestials.iter().collect::<Vec<_>>();
            let idx = match CelestialIdx::selected_celestial(&celestials_vec, (parent, camera)) {
                Ok(idx) => idx,
                Err(e) => {
                    error!("{:?}", e);
                    return;
                }
            };
            let next_idx = {
                idx.next(
                    celestials_vec
                        .clone()
                        .into_iter()
                        .map(|(_, idx)| idx)
                        .collect::<Vec<_>>(),
                )
            };
            if let Some(next_celestial) = celestials_vec
                .into_iter()
                .find(|(_, idx)| idx.0 == next_idx.0)
            {
                focus_celestial(
                    &mut commands,
                    (&camera, &mut transform),
                    &next_celestial.0,
                    next_celestial.1,
                );
            } else {
                // Err(EmptyQueryResult{err: None, parameter_name: "next_celestial".to_string()})?;
                error!(
                    "{:?}",
                    EmptyQueryResult {
                        err: None,
                        parameter_name: "next_celestial".to_string()
                    }
                );
            }
        }
        // Ok(())
    }
}

/// An event that indicates that a celestial has been selected by the user
#[derive(Event, Debug, Clone, Copy)]
pub struct SelectCelestial(Entity);

impl From<ListenerInput<Pointer<Down>>> for SelectCelestial {
    /// Converts a click event into a `SelectCelestial` event by saving the target of the click
    fn from(event: ListenerInput<Pointer<Down>>) -> Self {
        Self(event.target)
    }
}

/// Event Handler Systems
impl CameraPlugin {
    /// If the celestial is clicked on:
    ///   1. Parent the main camera to the celestial
    ///   2. Zero the camera's translation
    ///   3. Scale the camera to the celestial's radius
    ///
    /// TODO: sysfail
    #[call_log]
    pub fn select_celestial_focus(
        mut commands: Commands,
        celestials: Query<&CelestialIdx>,
        chunks: Query<(&Parent, Entity), With<ChunkIjkComponent>>,
        mut camera: Query<(Entity, &mut Transform), With<MainCamera>>,
        mut click_events: EventReader<SelectCelestial>,
    ) {
        // -> Result<(), Box<dyn std::error::Error>> {
        if let Ok((camera, mut transform)) = camera.get_single_mut() {
            if let Some(event) = click_events.read().next() {
                if let Ok((parent_entity, _)) = chunks.get(event.0) {
                    if let Ok(celestial_idx) = celestials.get(parent_entity.get()) {
                        focus_celestial(
                            &mut commands,
                            (&camera, &mut transform),
                            parent_entity,
                            celestial_idx,
                        );
                        return;
                    }
                }

                error!(
                    "{:?}",
                    EmptyQueryResult {
                        err: None,
                        parameter_name: "selected_celestial".to_string()
                    }
                );
            }
        }
        // Ok(())
    }
}

// Helper Functions
/// Parent the camera to the celestial
fn focus_celestial(
    commands: &mut Commands,
    camera: (&Entity, &mut Transform),
    parent: &Entity,
    celestial_idx: &CelestialIdx,
) {
    debug!("Focusing on celestial {:?}", celestial_idx.0);
    // Parent the camera to the celestial
    commands.entity(*camera.0).set_parent(*parent);
    // Zero the camera's translation
    // Scale the camera to the celestial's radius
    camera.1.translation = Vec3::new(0.0, 0.0, 0.0);
}
