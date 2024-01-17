use bevy::{
    core_pipeline::core_2d::Camera2d,
    ecs::{
        component::Component,
        event::EventReader,
        query::With,
        system::{Query, Resource},
    },
    input::mouse::MouseMotion,
    render::{color::Color, mesh::Mesh},
    transform::components::Transform,
};
use glam::Vec2;

use crate::{
    entities::celestials::celestial::Celestial,
    physics::{
        fallingsand::util::mesh::OwnedMeshData,
        util::{
            clock::Clock,
            vectors::{RelXyPoint, ScreenCoord, Vertex, WorldCoord},
        },
    },
};

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
    pub fn draw_brush_system(
        mut motion_evr: EventReader<MouseMotion>,
        mut query: Query<&mut Transform, With<BrushRadius>>,
    ) {
        for event in motion_evr.read() {
            query.for_each_mut(|mut transform| {
                transform.translation.x += event.delta.x;
                transform.translation.y += event.delta.y;
            });
        }
    }
}

// /// Brush Radius Effect
// impl Brush {
//     /// Based on the brush radius and the celestial cell size, return a list of
//     /// points in relative xy coordinates that the brush will affect.
//     fn brush_positions(&self, celestial: Query<&Celestial, Camera2d, Transform>) -> Vec<RelXyPoint> {
//         let center =
//             RelXyPoint(self.get_world_coord(camera).0) - RelXyPoint(celestial.get_world_coord().0);
//         let begin_at = center - RelXyPoint::new(self.data.radius.0, self.data.radius.0);
//         let end_at = center + RelXyPoint::new(self.data.radius.0, self.data.radius.0);
//         let mut positions = Vec::new();
//         let mut x = begin_at.0.x
//             + celestial
//                 .data
//                 .element_grid_dir
//                 .get_coordinate_dir()
//                 .get_cell_width()
//                 / 2.0;
//         while x < end_at.0.x {
//             let mut y = begin_at.0.y
//                 + celestial
//                     .data
//                     .element_grid_dir
//                     .get_coordinate_dir()
//                     .get_cell_width()
//                     / 2.0;
//             while y < end_at.0.y {
//                 let pos = RelXyPoint::new(x, y);
//                 if pos.0.distance(center.0) < self.data.radius.0 {
//                     positions.push(pos);
//                 }
//                 y += celestial
//                     .data
//                     .element_grid_dir
//                     .get_coordinate_dir()
//                     .get_cell_width();
//             }
//             x += celestial
//                 .data
//                 .element_grid_dir
//                 .get_coordinate_dir()
//                 .get_cell_width();
//         }
//         positions
//     }

//     pub fn apply(
//         &self,
//         celestial: &mut Celestial,
//         element_picker: &ElementPicker,
//         current_time: Clock,
//         camera: &Camera,
//     ) {
//         let positions = self.brush_positions(celestial, camera);
//         for pos in positions {
//             let element_dir = &mut celestial.data.element_grid_dir;
//             let coord_dir = element_dir.get_coordinate_dir();
//             let conversion = coord_dir.rel_pos_to_cell_idx(pos);
//             if let Ok(coords) = conversion {
//                 element_dir.set_element(
//                     coords,
//                     element_picker.get_selection().get_element(),
//                     current_time,
//                 );
//             }
//         }
//     }
// }
