//! The brush is a circle that can be resized and moved around the screen.
//! It can be used to apply elements to a celestial.

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use crate::bevy::components::mesh::GizmoDrawableLoop;
use crate::bevy::entities::celestials::celestial::Data;
use crate::bevy::entities::components::Radius;
use crate::bevy::errors::MissingParentError;
use crate::common::util::clock::Clock;
use crate::common::util::transforms::{get_mouse_model_position, get_mouse_world_position};
use crate::common::util::vectors::ModelCoord;
use bevy::app::{App, FixedUpdate, Plugin, Update};
use bevy::color::palettes::css::WHITE;
use bevy::core::FrameCount;
use bevy::core_pipeline::core_2d::Camera2d;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::{Commands, Res};
use bevy::hierarchy::{BuildChildren, Parent};
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseButton;
use bevy::input::ButtonInput;
use bevy::log::debug;
use bevy::log::error;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{SpatialBundle, Window, Without};

use bevy::render::camera::{Camera, OrthographicProjection};
use bevy::time::Time;
use bevy::transform::components::GlobalTransform;
use bevy::window::PrimaryWindow;
use bevy::{
    ecs::{component::Component, event::EventReader, query::With, system::Query},
    gizmos::gizmos::Gizmos,
    transform::components::Transform,
    window::CursorMoved,
};
// use bevy_mod_sysfail::sysfail;

use super::camera::MainCamera;
use super::element_picker::ElementSelection;

/// Identifies the brush
#[derive(Component)]
pub struct BrushComponent;

/// The brush is a circle that can be resized and moved around the screen.
pub struct BrushPlugin;

impl Plugin for BrushPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                Self::move_brush_system,
                Self::draw_brush_system,
                Self::resize_brush_system,
                Self::apply_brush_system,
            ),
        );
    }
}

/// Startup functions
/// These are not systems, rather are written as function to be applied
/// to the `GuiUnifiedPlugin`
impl BrushPlugin {
    /// Create the brush
    pub fn create_brush(commands: &mut Commands, camera: Entity) -> Entity {
        // Create the brush
        let brush = commands
            .spawn((
                Radius(0.5),
                BrushComponent,
                SpatialBundle {
                    transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
                    ..Default::default()
                },
            ))
            .id();

        // Parent the brush to the camera
        commands.entity(camera).push_children(&[brush]);

        brush
    }
}

/// Update functions
impl BrushPlugin {
    /// Move the brush with the mouse
    pub fn move_brush_system(
        mut query: Query<&mut Transform, (With<BrushComponent>, Without<MainCamera>)>,
        windows: Query<&Window, With<PrimaryWindow>>,
        cameras: Query<(&Camera, &Transform), (With<MainCamera>, Without<BrushComponent>)>,
    ) {
        let window = windows.single();
        let camera = cameras.single();
        if let Some(mouse_transform) = get_mouse_model_position(window, camera) {
            query.iter_mut().for_each(|mut brush_transform| {
                brush_transform.translation.x = mouse_transform.0.x;
                brush_transform.translation.y = mouse_transform.0.y;
            });
        }
    }

    /// Draw the brush circle
    pub fn draw_brush_system(
        query: Query<(&GlobalTransform, &Radius), With<BrushComponent>>,
        mut gizmos: Gizmos,
    ) {
        for (transform, brush_radius) in query.iter() {
            let mesh = brush_radius.mesh();
            GizmoDrawableLoop::new(mesh, WHITE.into()).draw_bevy_gizmo_loop(
                &mut gizmos,
                &Transform::from_translation(transform.translation()),
            );
        }
    }

    /// Resize the brush with + and -
    pub fn resize_brush_system(
        keys: Res<ButtonInput<KeyCode>>,
        mut query: Query<&mut Radius, With<BrushComponent>>,
    ) {
        for mut brush_radius in &mut query {
            if keys.just_pressed(KeyCode::Equal) {
                brush_radius.0 *= 2.0;
            }
            if keys.just_pressed(KeyCode::Minus) {
                brush_radius.0 /= 2.0;
            }
            if brush_radius.0 < 0.5 {
                brush_radius.0 = 0.5;
            }
        }
    }

    /// Based on the brush radius and the celestial cell size, return a list of
    /// points in relative xy coordinates that the brush will affect.
    /// TODO: sysfail
    pub fn apply_brush_system(
        mouse: Res<ButtonInput<MouseButton>>,
        mut brush: Query<
            (&Parent, &Transform, &Radius),
            (With<BrushComponent>, Without<MainCamera>),
        >,
        mut camera: Query<&Parent, (With<MainCamera>, Without<BrushComponent>)>,
        mut celestial: Query<&mut Data>,
        element_picker: Res<ElementSelection>,
        current_time: Res<Time>,
        frame_count: Res<FrameCount>,
    ) {
        if !mouse.pressed(MouseButton::Left) {
            // return Ok(());
            return;
        }

        // Get the camera the brush follows, and the celestial the camera follows
        let (brush_parent, brush_transform, radius) = brush.single_mut();
        let camera_parent = match camera.get_mut(brush_parent.get()) {
            Ok(x) => x,
            Err(e) => {
                error!(
                    "{:?}",
                    MissingParentError {
                        err: Some(Box::new(e)),
                        entity_type: "Brush".to_string(),
                        necessary_parent_type: "Camera".to_string()
                    }
                );
                return;
            }
        };
        let mut celestial = match celestial.get_mut(camera_parent.get()) {
            Ok(x) => x,
            Err(e) => {
                error!(
                    "{:?}",
                    MissingParentError {
                        err: Some(Box::new(e)),
                        entity_type: "Camera".to_string(),
                        necessary_parent_type: "Celestial".to_string(),
                    }
                );
                return;
            }
        };

        // Get the bounds of the brush in terms of the celestials cells
        let brush_translation = brush_transform.translation;
        let begin_at = ModelCoord::new(
            brush_translation.x - radius.0,
            brush_translation.y - radius.0,
        );
        let end_at = ModelCoord::new(
            brush_translation.x + radius.0,
            brush_translation.y + radius.0,
        );
        let mut positions = Vec::new();
        let mut x = begin_at.0.x + celestial.element_grid_dir.coordinate_dir().cell_width().0 / 2.0;
        while x < end_at.0.x {
            let mut y =
                begin_at.0.y + celestial.element_grid_dir.coordinate_dir().cell_width().0 / 2.0;
            while y < end_at.0.y {
                let pos = ModelCoord::new(x, y);
                if pos.0.distance(brush_translation.truncate()) < radius.0 {
                    positions.push(pos);
                }
                y += celestial.element_grid_dir.coordinate_dir().cell_width().0;
            }
            x += celestial.element_grid_dir.coordinate_dir().cell_width().0;
        }

        // Now apply the brush to the celestial
        let current_time = Clock::new(current_time.as_generic(), frame_count.as_ref().to_owned());
        for pos in positions {
            let element_dir = &mut celestial.element_grid_dir;
            let coord_dir = element_dir.coordinate_dir();
            let conversion = coord_dir.rel_pos_to_cell_idx(pos);
            if let Ok(coords) = conversion {
                element_dir.set_element(coords, element_picker.0.element(), current_time);
            }
        }
    }
}
