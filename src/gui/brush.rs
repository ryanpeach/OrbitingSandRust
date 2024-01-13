use bevy_ecs::{component::Component, system::Resource};
use ggez::{glam::Vec2, graphics::Vertex};

use crate::{
    gui::windows::element_picker::ElementPicker,
    nodes::{camera::cam::Camera, celestial::Celestial, node_trait::NodeTrait},
    physics::{
        fallingsand::util::mesh::OwnedMeshData,
        util::{
            clock::Clock,
            vectors::{RelXyPoint, ScreenCoord, WorldCoord},
        },
    },
};

use super::screen_trait::ScreenDrawable;

#[derive(Component, Debug, Clone, Copy)]
pub struct BrushRadius(pub f32);

#[derive(Component, Debug, Clone, Copy)]
pub struct BrushData {
    radius: BrushRadius,
    nb_vertices: usize,
}

impl Default for BrushData {
    fn default() -> Self {
        Self {
            radius: BrushRadius(1.0),
            nb_vertices: 100,
        }
    }
}

#[derive(Resource, Default)]
pub struct Brush {
    data: BrushData,
    drawable: ScreenDrawable,
}

impl Brush {
    pub fn new() -> Self {
        let data = BrushData::default();
        Self {
            data: data,
            drawable: Self::calc_mesh(data),
        }
    }

    pub fn set_radius(&mut self, radius: BrushRadius) {
        self.data.radius = radius;
        if self.data.radius.0 < 0.5 {
            self.data.radius = BrushRadius(0.5);
        }
        self.drawable = Self::calc_mesh(self.data);
    }

    pub fn get_radius(&self) -> BrushRadius {
        self.data.radius
    }

    pub fn set_position(&mut self, screen_coords: ScreenCoord) {
        self.drawable.set_screen_coord(screen_coords);
    }

    pub fn mult_radius(&mut self, multiplier: f32) {
        self.set_radius(BrushRadius(multiplier * self.get_radius().0));
    }

    pub fn get_world_coord(&self, camera: &Camera) -> WorldCoord {
        camera.screen_to_world_coords(self.drawable.get_screen_coord())
    }

    pub fn calc_mesh(data: BrushData) -> ScreenDrawable {
        let mut vertices: Vec<Vertex> = Vec::with_capacity(data.nb_vertices);
        let mut indices: Vec<u32> = Vec::with_capacity(data.nb_vertices);
        for i in 0..data.nb_vertices {
            let angle = 2.0 * std::f32::consts::PI * (i as f32) / (data.nb_vertices as f32);
            let x = data.radius.0 * angle.cos();
            let y = data.radius.0 * angle.sin();
            vertices.push(Vertex {
                position: [x, y],
                uv: [0.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            });
            indices.push(i as u32);
        }
        let mesh = OwnedMeshData::new(vertices, indices);
        ScreenDrawable::new(ScreenCoord(Vec2::new(0.0, 0.0)), mesh, None)
    }
}

/// Brush Radius Effect
impl Brush {
    /// Based on the brush radius and the celestial cell size, return a list of
    /// points in relative xy coordinates that the brush will affect.
    fn brush_positions(&self, celestial: &Celestial, camera: &Camera) -> Vec<RelXyPoint> {
        let center =
            RelXyPoint(self.get_world_coord(&camera).0) - RelXyPoint(celestial.get_world_coord().0);
        let begin_at = center - RelXyPoint::new(self.data.radius.0, self.data.radius.0);
        let end_at = center + RelXyPoint::new(self.data.radius.0, self.data.radius.0);
        let mut positions = Vec::new();
        let mut x = begin_at.0.x
            + celestial
                .data
                .element_grid_dir
                .get_coordinate_dir()
                .get_cell_width()
                / 2.0;
        while x < end_at.0.x {
            let mut y = begin_at.0.y
                + celestial
                    .data
                    .element_grid_dir
                    .get_coordinate_dir()
                    .get_cell_width()
                    / 2.0;
            while y < end_at.0.y {
                let pos = RelXyPoint::new(x, y);
                if pos.0.distance(center.0) < self.data.radius.0 {
                    positions.push(pos);
                }
                y += celestial
                    .data
                    .element_grid_dir
                    .get_coordinate_dir()
                    .get_cell_width();
            }
            x += celestial
                .data
                .element_grid_dir
                .get_coordinate_dir()
                .get_cell_width();
        }
        positions
    }

    pub fn apply(
        &self,
        celestial: &mut Celestial,
        element_picker: &ElementPicker,
        current_time: Clock,
        camera: &Camera,
    ) {
        let positions = self.brush_positions(celestial, &camera);
        for pos in positions {
            let element_dir = &mut celestial.data.element_grid_dir;
            let coord_dir = element_dir.get_coordinate_dir();
            let conversion = coord_dir.rel_pos_to_cell_idx(pos);
            if let Ok(coords) = conversion {
                element_dir.set_element(
                    coords,
                    element_picker.get_selection().get_element(),
                    current_time,
                );
            }
        }
    }
}
