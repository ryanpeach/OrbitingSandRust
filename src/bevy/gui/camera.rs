//! The bevy camera for the game

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use std::ops::Add;

use bevy::log::error;
use bevy::{
    app::{App, Plugin, Update},
    core_pipeline::core_2d::{Camera2d, Camera2dBundle},
    ecs::{
        component::Component,
        event::{Event, EventReader},
        query::{With, Without},
        system::{Commands, Query, Res, ResMut},
    },
    hierarchy::{BuildChildren, Parent},
    input::{keyboard::KeyCode, mouse::MouseWheel, ButtonInput},
    math::{Rect, Vec2, Vec3},
    prelude::Entity,
    time::Time,
    transform::components::Transform,
};
use bevy_eventlistener::callbacks::ListenerInput;
use bevy_mod_picking::events::{Down, Pointer};


use conv::ValueFrom;

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
        camera: (&Parent, Entity),
    ) -> Result<CelestialIdx, EmptyQueryResult> {
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
        let parent = camera.0;
        if let Some(celestial) = celestials.iter().find(|(entity, _)| *entity == **parent) {
            Ok(*celestial.1)
        } else {
            Err(EmptyQueryResult {
                err: None,
                parameter_name: "selected_celestial".to_string(),
            })
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
        app.add_systems(Update, Self::move_camera_system);
        // Not currently working
        // app.add_systems(Update, Self::frustum_culling_2d);
        app.add_systems(Update, Self::select_celestial_focus);
        app.add_systems(Update, Self::cycle_celestial_focus);
        app.add_systems(Update, Self::first_celestial_focus);
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
                    transform: Transform::from_scale(Vec3::new(1.0, 1.0, 1.0) * 100.0),
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
    fn zoom_camera_system(
        time: Res<Time>,
        mut scroll_evr: EventReader<MouseWheel>,
        mut query: Query<(&mut Transform, &mut Camera2d)>,
    ) {
        let mut delta = 0.;
        for ev in scroll_evr.read() {
            delta += ev.y;
        }
        if delta != 0. {
            for (mut transform, _) in &mut query {
                transform.scale *= (1. + delta * time.delta_seconds() * 0.5).max(0.0001);
            }
        }
    }

    /// Move the camera based on keyboard input
    fn move_camera_system(
        time: Res<Time>,
        keyboard_input: Res<ButtonInput<KeyCode>>,
        mut query: Query<(&mut Transform, &mut Camera2d)>,
    ) {
        let mut delta = Vec3::ZERO;
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
        if delta != Vec3::ZERO {
            for (mut transform, _) in &mut query {
                let scale = transform.scale;
                transform.translation += delta * time.delta_seconds() * scale * 100.;
            }
        }
    }
}

/// Check if two rectangles overlap
fn rect_overlaps(this: &Rect, other: &Rect) -> bool {
    this.min.x < other.max.x
        && this.max.x > other.min.x
        && this.min.y < other.max.y
        && this.max.y > other.min.y
}

/// Add a vector to every corner of a rectangle
fn rect_add(this: &Rect, other: &Vec2) -> Rect {
    Rect::new(
        this.min.x + other.x,
        this.min.y + other.y,
        this.max.x + other.x,
        this.max.y + other.y,
    )
}

/// Celestial Focus Systems
impl CameraPlugin {
    /// If you press "\[" or "\]", you can cycle through the celestials
    /// TODO: #[sysfail]
    pub fn cycle_celestial_focus(
        mut commands: Commands,
        celestials: Query<(Entity, &CelestialIdx)>,
        mut camera: Query<(&Parent, Entity, &mut Transform), With<MainCamera>>,
        mut input: ResMut<ButtonInput<KeyCode>>,
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
                if input.just_pressed(KeyCode::BracketLeft) {
                    input.reset(KeyCode::BracketLeft);
                    idx.prev(
                        celestials_vec
                            .clone()
                            .into_iter()
                            .map(|(_, idx)| idx)
                            .collect::<Vec<_>>(),
                    )
                } else if input.just_pressed(KeyCode::BracketRight) {
                    input.reset(KeyCode::BracketRight);
                    idx.next(
                        celestials_vec
                            .clone()
                            .into_iter()
                            .map(|(_, idx)| idx)
                            .collect::<Vec<_>>(),
                    )
                } else {
                    // return Ok(());
                    return;
                }
            };
            if let Some(next_celestial) = celestials_vec
                .into_iter()
                .find(|(_, idx)| idx.0 == next_idx.0)
            {
                focus_celestial(&mut commands, (&camera, &mut transform), &next_celestial.0);
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

    /// Same as the above, but for when the camera doesn't have a parent yet
    /// TODO: #[sysfail]
    pub fn first_celestial_focus(
        mut commands: Commands,
        celestials: Query<(Entity, &CelestialIdx)>,
        mut camera: Query<(Entity, &mut Transform), (With<MainCamera>, Without<Parent>)>,
        mut input: ResMut<ButtonInput<KeyCode>>,
    ) {
        // -> Result<(), Box<dyn std::error::Error>> {
        if input.just_pressed(KeyCode::BracketLeft) || input.just_pressed(KeyCode::BracketRight) {
            input.reset(KeyCode::BracketLeft);
            input.reset(KeyCode::BracketRight);
            if let Ok((camera, mut transform)) = camera.get_single_mut() {
                if let Some(first_celestial) = celestials.into_iter().find(|(_, idx)| idx.0 == 0) {
                    focus_celestial(&mut commands, (&camera, &mut transform), &first_celestial.0);
                } else {
                    // Err(EmptyQueryResult{err: None, parameter_name: "first_celestial".to_string()})?;
                    error!(
                        "{:?}",
                        EmptyQueryResult {
                            err: None,
                            parameter_name: "first_celestial".to_string()
                        }
                    );
                }
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
    /// TODO: #[sysfail]
    pub fn select_celestial_focus(
        mut commands: Commands,
        chunks: Query<(&Parent, Entity), With<ChunkIjkComponent>>,
        mut camera: Query<(Entity, &mut Transform), With<MainCamera>>,
        mut click_events: EventReader<SelectCelestial>,
    ) {
        // -> Result<(), Box<dyn std::error::Error>> {
        let mut camera = camera.single_mut();
        if let Some(event) = click_events.read().next() {
            if let Some(parent) = chunks.iter().find(|(_, chunk_id)| *chunk_id == event.0) {
                focus_celestial(&mut commands, (&camera.0, &mut camera.1), parent.0);
            } else {
                // Err(EmptyQueryResult{err: None, parameter_name: "selected_celestial".to_string()})?;
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
fn focus_celestial(commands: &mut Commands, camera: (&Entity, &mut Transform), parent: &Entity) {
    // Parent the camera to the celestial
    commands.entity(*camera.0).set_parent(*parent);
    // Zero the camera's translation
    // Scale the camera to the celestial's radius
    camera.1.translation = Vec3::new(0.0, 0.0, 0.0);
}
