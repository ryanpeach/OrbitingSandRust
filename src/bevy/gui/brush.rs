//! The brush is a circle that can be resized and moved around the screen.
//! It can be used to apply elements to a celestial.

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use crate::bevy::entities::celestials::celestial::Data;
use crate::bevy::entities::components::Radius;
use crate::bevy::errors::MissingParentError;
use crate::common::util::clock::Clock;
use crate::common::util::transforms::{get_mouse_model_position, get_mouse_world_position};
use crate::common::util::vectors::ModelCoord;
use bevy::app::{App, FixedUpdate, Plugin, Update};
use bevy::asset::Assets;
use bevy::color::palettes::css::WHITE;
use bevy::core::{FrameCount, Name};
use bevy::core_pipeline::core_2d::Camera2d;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::{Commands, Res};
use bevy::hierarchy::{BuildChildren, Parent};
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseButton;
use bevy::input::ButtonInput;
use bevy::log::debug;
use bevy::log::error;
use bevy::math::{Vec2, Vec3, VectorSpace};
use bevy::prelude::{ResMut, SpatialBundle, Window, Without};

use bevy::render::camera::{Camera, OrthographicProjection};
use bevy::render::view::{InheritedVisibility, Visibility};
use bevy::time::Time;
use bevy::transform::components::GlobalTransform;
use bevy::window::PrimaryWindow;
use bevy::{
    ecs::{component::Component, event::EventReader, query::With, system::Query},
    gizmos::gizmos::Gizmos,
    transform::components::Transform,
    window::CursorMoved,
};
use bevy_polyline::material::PolylineMaterial;
use bevy_polyline::polyline::{Polyline, PolylineBundle};
use conv::{ConvAsUtil, Saturate};
// use bevy_mod_sysfail::sysfail;

use super::camera::{MainCamera, OverlayLayer4};
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
                Self::brush_visibility_system,
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
    pub fn create_brush(
        commands: &mut Commands,
        camera: Entity,
        mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
        mut polylines: ResMut<Assets<Polyline>>,
    ) -> Entity {
        let circle: Vec<Vec2> = (0..360)
            .map(|i| {
                Vec2::new(
                    (i as f64)
                        .to_radians()
                        .cos()
                        .approx()
                        .expect("This will be between 0 and 1"),
                    (i as f64)
                        .to_radians()
                        .sin()
                        .approx()
                        .expect("This will be between 0 and 1"),
                )
            })
            .collect();
        // Create the brush
        let brush = commands
            .spawn((
                Name::new("Brush"),
                Radius(0.5),
                BrushComponent,
                // Not using [`PolylineBundle`] Because this should not have [`InheritedVisibility`]
                // (since cameras dont have visibility, and this is a child of the camera)
                polylines.add(Polyline {
                    vertices: circle.iter().map(|x| x.extend(0.0)).collect(),
                }),
                polyline_materials.add(PolylineMaterial {
                    width: 10.0,
                    color: WHITE.into(),
                    perspective: false,
                    ..Default::default()
                }),
                Transform::from_translation(Vec2::ZERO.extend(4.0)),
                Visibility::Hidden,
                GlobalTransform::IDENTITY,
                OverlayLayer4,
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

    /// The brush will be visible iff the camera is a child of a celestial
    /// TODO: Would it be worth it to make this event driven?
    pub fn brush_visibility_system(
        mut brushes: Query<
            &mut Visibility,
            (With<BrushComponent>, Without<MainCamera>, Without<Data>),
        >,
        cameras: Query<&Parent, (With<MainCamera>, Without<BrushComponent>, Without<Data>)>,
        celestials: Query<Entity, (With<Data>, Without<MainCamera>, Without<BrushComponent>)>,
    ) {
        if let Ok(camera_parent) = cameras.get_single() {
            if let Ok(_) = celestials.get(camera_parent.get()) {
                *brushes.single_mut() = Visibility::Visible;
            } else {
                *brushes.single_mut() = Visibility::Hidden;
            }
        } else {
            *brushes.single_mut() = Visibility::Visible;
        }
    }

    /// Resize the brush with + and -
    pub fn resize_brush_system(
        keys: Res<ButtonInput<KeyCode>>,
        mut query: Query<(&mut Radius, &mut Transform), With<BrushComponent>>,
    ) {
        for (mut brush_radius, mut transform) in &mut query {
            if keys.just_pressed(KeyCode::Equal) {
                brush_radius.0 *= 2.0;
            }
            if keys.just_pressed(KeyCode::Minus) {
                brush_radius.0 /= 2.0;
            }
            if brush_radius.0 < 0.5 {
                brush_radius.0 = 0.5;
            }
            *transform = transform.with_scale(Vec2::ONE.extend(0.0) * brush_radius.0);
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
