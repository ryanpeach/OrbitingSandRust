//! The brush is a circle that can be resized and moved around the screen.
//! It can be used to apply elements to a celestial.

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use crate::entities::celestials::celestial::CelestialData;
use crate::entities::utils::Radius;
use crate::physics::fallingsand::util::mesh::GizmoDrawableLoop;
use crate::physics::util::clock::Clock;
use crate::physics::util::vectors::{mouse_coord_to_world_coord, RelXyPoint};
use bevy::app::{App, Plugin, Update};
use bevy::core::FrameCount;
use bevy::core_pipeline::core_2d::Camera2d;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::Without;
use bevy::ecs::system::{Commands, Res};
use bevy::hierarchy::{BuildChildren, Parent};
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseButton;
use bevy::input::Input;
use bevy::log::debug;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::Window;
use bevy::render::color::Color;

use bevy::time::Time;
use bevy::{
    ecs::{component::Component, event::EventReader, query::With, system::Query},
    gizmos::gizmos::Gizmos,
    transform::components::Transform,
    window::CursorMoved,
};

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
            Update,
            (
                Self::move_brush_system,
                Self::draw_brush_system,
                Self::resize_brush_system,
                // Self::apply_brush_system,
            ),
        );
    }
}

/// Startup functions
/// These are not systems, rather are written as function to be applied
/// to the GuiUnifiedPlugin
impl BrushPlugin {
    /// Create the brush
    pub fn create_brush(commands: &mut Commands, camera: Entity) -> Entity {
        // Create the brush
        let brush = commands
            .spawn((
                Radius(0.5),
                BrushComponent,
                Transform::from_translation(Vec3::new(0., 0., 0.)),
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
        windows: Query<&mut Window>,
        mut cursor_moved_events: EventReader<CursorMoved>,
        mut query: Query<&mut Transform, With<BrushComponent>>,
    ) {
        for event in cursor_moved_events.read() {
            let mouse_transform = mouse_coord_to_world_coord(&windows, event);

            query.for_each_mut(|mut brush_transform| {
                brush_transform.translation.x = mouse_transform.translation.x;
                brush_transform.translation.y = mouse_transform.translation.y; // Invert y-axis to match Bevy's coordinate system
            });
        }
    }

    /// Draw the brush circle
    pub fn draw_brush_system(
        query: Query<(&Transform, &Radius), With<BrushComponent>>,
        mut gizmos: Gizmos,
    ) {
        for (transform, brush_radius) in query.iter() {
            let mesh = brush_radius.calc_mesh();
            GizmoDrawableLoop::new(mesh, Color::WHITE).draw_bevy_gizmo_loop(&mut gizmos, transform);
        }
    }

    /// Resize the brush with + and -
    pub fn resize_brush_system(
        keys: Res<Input<KeyCode>>,
        mut query: Query<&mut Radius, With<BrushComponent>>,
    ) {
        for mut brush_radius in query.iter_mut() {
            if keys.just_pressed(KeyCode::Equals) {
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
    pub fn apply_brush_system(
        mouse: Res<Input<MouseButton>>,
        mut brush: Query<(&Parent, &mut Transform, &Radius), With<BrushComponent>>,
        mut camera: Query<
            (&Parent, &mut Transform, &mut Camera2d, &MainCamera),
            Without<BrushComponent>,
        >,
        mut celestial: Query<&mut CelestialData>,
        element_picker: Res<ElementSelection>,
        current_time: Res<Time>,
        frame_count: Res<FrameCount>,
    ) {
        if !mouse.pressed(MouseButton::Left) {
            return;
        }
        debug!("Applying brush");
        let (brush_parent, brush_transform, radius) = brush.single_mut();
        let (camera_parent, camera_transform, _, _) = camera.get_mut(brush_parent.get()).unwrap();
        let mut celestial = celestial.get_mut(camera_parent.get()).unwrap();
        let begin_at = RelXyPoint::new(
            radius.0 + brush_transform.translation.x + camera_transform.translation.x,
            radius.0 + brush_transform.translation.y + camera_transform.translation.y,
        );
        let end_at = RelXyPoint::new(
            radius.0 + brush_transform.translation.x + camera_transform.translation.x,
            radius.0 + brush_transform.translation.y + camera_transform.translation.y,
        );
        let mut positions = Vec::new();
        let mut x = begin_at.0.x
            + celestial
                .element_grid_dir
                .get_coordinate_dir()
                .get_cell_width()
                .0
                / 2.0;
        while x < end_at.0.x {
            let mut y = begin_at.0.y
                + celestial
                    .element_grid_dir
                    .get_coordinate_dir()
                    .get_cell_width()
                    .0
                    / 2.0;
            while y < end_at.0.y {
                let pos = RelXyPoint::new(x, y);
                if pos.0.distance(Vec2::new(0., 0.)) < radius.0 {
                    positions.push(pos);
                }
                y += celestial
                    .element_grid_dir
                    .get_coordinate_dir()
                    .get_cell_width()
                    .0;
            }
            x += celestial
                .element_grid_dir
                .get_coordinate_dir()
                .get_cell_width()
                .0;
        }

        // Now apply the brush to the celestial
        let current_time = Clock::new(current_time.as_generic(), frame_count.as_ref().to_owned());
        for pos in positions {
            let element_dir = &mut celestial.element_grid_dir;
            let coord_dir = element_dir.get_coordinate_dir();
            let conversion = coord_dir.rel_pos_to_cell_idx(pos);
            if let Ok(coords) = conversion {
                element_dir.set_element(coords, element_picker.0.get_element(), current_time);
            }
        }
    }
}
