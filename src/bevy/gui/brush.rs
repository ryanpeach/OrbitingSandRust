//! The brush is a circle that can be resized and moved around the screen.
//! It can be used to apply elements to a celestial.

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use crate::bevy::entities::celestials::celestial::Data;
use crate::bevy::entities::components::Radius;
use crate::common::util::clock::Clock;
use crate::common::util::transforms::get_mouse_model_position;
use crate::common::util::vectors::ModelCoord;
use bevy::app::{App, Plugin, Update};
use bevy::asset::Assets;
use bevy::color::LinearRgba;
use bevy::core::FrameCount;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::{Commands, Res};
use bevy::hierarchy::{BuildChildren, Parent};
use bevy::input::common_conditions::{input_just_pressed, input_pressed};
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseButton;
use bevy::input::ButtonInput;
use bevy::log::{debug, trace, trace_once};
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{Circle, IntoSystemConfigs, ResMut, Window, Without};

use bevy::render::camera::Camera;
use bevy::render::mesh::Mesh;
use bevy::render::view::Visibility;
use bevy::sprite::{ColorMaterial, MaterialMesh2dBundle};
use bevy::time::Time;
use bevy::window::PrimaryWindow;
use bevy::{
    ecs::{component::Component, query::With, system::Query},
    transform::components::Transform,
};
use macros::{call_log, call_log_once};
// use bevy_mod_sysfail::sysfail;

use super::camera::{MainCamera, OverlayLayer3};
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
                Self::resize_up_brush_system.run_if(input_just_pressed(KeyCode::Equal)),
                Self::resize_down_brush_system.run_if(input_just_pressed(KeyCode::Minus)),
                Self::apply_brush_system.run_if(input_pressed(MouseButton::Left)),
                Self::move_brush_system,
                Self::reparent_brush_system,
                Self::brush_visibility_system,
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
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) -> Entity {
        // Create the brush
        let mesh = Mesh::from(Circle::new(1.0));
        let material = ColorMaterial {
            color: LinearRgba {
                red: 1.0,
                green: 1.0,
                blue: 1.0,
                alpha: 0.2,
            }
            .into(),
            ..Default::default()
        };
        let brush = commands
            .spawn((
                Radius(0.5),
                BrushComponent,
                MaterialMesh2dBundle {
                    mesh: meshes.add(mesh).into(),
                    material: materials.add(material),
                    transform: Transform::from_translation(Vec3::new(0., 0., 3.0)),
                    visibility: Visibility::Hidden,
                    ..Default::default()
                },
                OverlayLayer3,
            ))
            .id();

        brush
    }
}

/// Update functions
impl BrushPlugin {
    /// Move the brush with the mouse
    #[call_log_once]
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
    /// Resize the brush with + and -
    #[call_log]
    pub fn resize_up_brush_system(
        mut brushes: Query<(&mut Radius, &mut Transform), With<BrushComponent>>,
    ) {
        if let Ok((mut brush_radius, mut brush_transform)) = brushes.get_single_mut() {
            brush_radius.0 *= 2.0;
            debug!("Brush radius changed to {:?}", brush_radius.0);
            *brush_transform = brush_transform.with_scale((Vec2::ONE * brush_radius.0).extend(0.0));
        }
    }
    /// Resize the brush with + and -
    #[call_log]
    pub fn resize_down_brush_system(
        mut brushes: Query<(&mut Radius, &mut Transform), With<BrushComponent>>,
    ) {
        if let Ok((mut brush_radius, mut brush_transform)) = brushes.get_single_mut() {
            brush_radius.0 /= 2.0;
            if brush_radius.0 < 0.5 {
                brush_radius.0 = 0.5;
            }
            debug!("Brush radius changed to {:?}", brush_radius.0);
            *brush_transform = brush_transform.with_scale((Vec2::ONE * brush_radius.0).extend(0.0));
        }
    }

    /// If the cameras parent changes, change this parent as well
    #[call_log_once]
    pub fn reparent_brush_system(
        mut commands: Commands,
        cameras: Query<&Parent, (With<MainCamera>, Without<BrushComponent>)>,
        brushes: Query<(Option<&Parent>, Entity), (With<BrushComponent>, Without<MainCamera>)>,
    ) {
        if let Ok(camera_parent) = cameras.get_single() {
            if let Ok((brush_parent, brush)) = brushes.get_single() {
                if let Some(brush_parent) = brush_parent {
                    if camera_parent.get() != brush_parent.get() {
                        debug!("Camera parent changed, changing brush parent to match.");
                        commands.entity(brush).set_parent(camera_parent.get());
                    }
                } else {
                    debug!("Brush has no parent, changing brush parent to match camera parent.");
                    commands.entity(brush).set_parent(camera_parent.get());
                }
            }
        }
    }

    /// If the brush is a child of a celestial, make it visible
    #[call_log_once]
    pub fn brush_visibility_system(
        mut brushes: Query<(&Parent, &mut Visibility), (With<BrushComponent>, Without<Data>)>,
        celestials: Query<Entity, (Without<BrushComponent>, With<Data>)>,
    ) {
        if let Ok((brush_parent, mut brush_visibility)) = brushes.get_single_mut() {
            if celestials.get(brush_parent.get()).is_ok() {
                if *brush_visibility != Visibility::Visible {
                    debug!("Brush changed to visible");
                    *brush_visibility = Visibility::Visible;
                }
            } else if *brush_visibility != Visibility::Hidden {
                debug!("Brush changed to hidden");
                *brush_visibility = Visibility::Hidden;
            }
        }
    }

    /// Based on the brush radius and the celestial cell size, return a list of
    /// points in relative xy coordinates that the brush will affect.
    /// TODO: sysfail
    #[call_log_once]
    pub fn apply_brush_system(
        mut brush: Query<
            (&Parent, &Transform, &Radius, &Visibility),
            (With<BrushComponent>, Without<MainCamera>),
        >,
        mut celestial: Query<&mut Data>,
        element_picker: Res<ElementSelection>,
        current_time: Res<Time>,
        frame_count: Res<FrameCount>,
    ) {
        // Get the celestial the brush follows
        if let Ok((brush_parent, brush_transform, radius, brush_visibility)) =
            brush.get_single_mut()
        {
            // We enable or disable the brush by making it visible or not
            if brush_visibility != Visibility::Visible {
                return;
            }
            if let Ok(mut celestial) = celestial.get_mut(brush_parent.get()) {
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
                let mut x =
                    begin_at.0.x + celestial.element_grid_dir.coordinate_dir().cell_width().0 / 2.0;
                while x < end_at.0.x {
                    let mut y = begin_at.0.y
                        + celestial.element_grid_dir.coordinate_dir().cell_width().0 / 2.0;
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
                let current_time =
                    Clock::new(current_time.as_generic(), frame_count.as_ref().to_owned());
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
    }
}
