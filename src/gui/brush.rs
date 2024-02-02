use crate::entities::celestials::celestial::CelestialData;
use crate::physics::fallingsand::util::mesh::OwnedMeshData;
use crate::physics::util::clock::Clock;
use crate::physics::util::vectors::{mouse_coord_to_world_coord, RelXyPoint, Vertex};
use bevy::app::{App, Plugin, Update};
use bevy::core::FrameCount;
use bevy::core_pipeline::core_2d::Camera2d;
use bevy::ecs::query::Without;
use bevy::ecs::system::Res;
use bevy::hierarchy::Parent;
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseButton;
use bevy::input::Input;
use bevy::log::debug;
use bevy::math::Vec2;
use bevy::prelude::Window;
use bevy::time::Time;
use bevy::{
    ecs::{component::Component, event::EventReader, query::With, system::Query},
    gizmos::gizmos::Gizmos,
    render::color::Color,
    transform::components::Transform,
    window::CursorMoved,
};

use super::element_picker::ElementSelection;

/// The brush is a circle that can be resized and moved around the screen.
pub struct BrushPlugin;

impl Plugin for BrushPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                BrushRadius::move_brush_system,
                BrushRadius::draw_brush_system,
                BrushRadius::resize_brush_system,
                BrushRadius::apply_brush_system,
            ),
        );
    }
}

#[derive(Default, Component, Debug, Clone, Copy)]
pub struct BrushRadius(pub f32);

impl BrushRadius {
    pub fn calc_mesh(self) -> OwnedMeshData {
        const NB_VERTICES: usize = 100;
        let mut vertices: Vec<Vertex> = Vec::with_capacity(NB_VERTICES);
        let mut indices: Vec<u32> = Vec::with_capacity(NB_VERTICES);
        for i in 0..NB_VERTICES {
            let angle = 2.0 * std::f32::consts::PI * (i as f32) / (NB_VERTICES as f32);
            let x = self.0 * angle.cos();
            let y = self.0 * angle.sin();
            vertices.push(Vertex {
                position: Vec2::new(x, y),
                uv: Vec2::new(0.0, 0.0),
                color: Color::rgba(0.0, 0.0, 0.0, 1.0),
            });
            indices.push(i as u32);
        }
        OwnedMeshData::new(vertices, indices)
    }
}

/// Bevy Systems
impl BrushRadius {
    pub fn move_brush_system(
        windows: Query<&mut Window>,
        mut cursor_moved_events: EventReader<CursorMoved>,
        mut query: Query<&mut Transform, With<BrushRadius>>,
    ) {
        for event in cursor_moved_events.read() {
            let mouse_transform = mouse_coord_to_world_coord(&windows, event);

            query.for_each_mut(|mut brush_transform| {
                brush_transform.translation.x = mouse_transform.translation.x;
                brush_transform.translation.y = mouse_transform.translation.y; // Invert y-axis to match Bevy's coordinate system
            });
        }
    }

    pub fn draw_brush_system(query: Query<(&Transform, &BrushRadius)>, mut gizmos: Gizmos) {
        for (transform, brush_radius) in query.iter() {
            let mesh = brush_radius.calc_mesh();
            mesh.draw_bevy_gizmo_outline(&mut gizmos, transform);
        }
    }

    pub fn resize_brush_system(keys: Res<Input<KeyCode>>, mut query: Query<&mut BrushRadius>) {
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
}

/// Brush Radius Effect
impl BrushRadius {
    /// Based on the brush radius and the celestial cell size, return a list of
    /// points in relative xy coordinates that the brush will affect.
    pub fn apply_brush_system(
        mouse: Res<Input<MouseButton>>,
        mut brush: Query<(&Parent, &mut Transform, &BrushRadius)>,
        mut camera: Query<(&Parent, &mut Transform, &mut Camera2d), Without<BrushRadius>>,
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
        let (camera_parent, camera_transform, _) = camera.get_mut(brush_parent.get()).unwrap();
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
                / 2.0;
        while x < end_at.0.x {
            let mut y = begin_at.0.y
                + celestial
                    .element_grid_dir
                    .get_coordinate_dir()
                    .get_cell_width()
                    / 2.0;
            while y < end_at.0.y {
                let pos = RelXyPoint::new(x, y);
                if pos.0.distance(Vec2::new(0., 0.)) < radius.0 {
                    positions.push(pos);
                }
                y += celestial
                    .element_grid_dir
                    .get_coordinate_dir()
                    .get_cell_width();
            }
            x += celestial
                .element_grid_dir
                .get_coordinate_dir()
                .get_cell_width();
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
