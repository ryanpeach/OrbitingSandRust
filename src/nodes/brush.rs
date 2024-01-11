use bevy_ecs::{bundle::Bundle, component::Component};
use ggez::glam::Vec2;

use crate::{
    gui::windows::element_picker::ElementPicker,
    nodes::celestial::Celestial,
    physics::util::{
        clock::Clock,
        vectors::{RelXyPoint, WorldCoord},
    },
};

use super::camera::cam::Camera;

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct BrushData {
    radius: f32,
}

#[derive(Bundle, Debug, Clone, Copy)]
pub struct Brush {
    brush: BrushData,
    world_coord: WorldCoord,
}

impl Default for Brush {
    fn default() -> Self {
        Self {
            brush: BrushData { radius: 0.5 },
            world_coord: WorldCoord(Vec2 { x: 0.0, y: 0.0 }),
        }
    }
}

impl Brush {
    pub fn set_radius(&mut self, radius: f32) {
        self.brush.radius = radius;
    }

    pub fn get_radius(&self) -> f32 {
        self.brush.radius
    }

    pub fn set_position(&mut self, world_coord: WorldCoord) {
        self.world_coord = world_coord;
    }

    pub fn get_world_coord(&self) -> WorldCoord {
        self.world_coord
    }

    pub fn mult_radius(&mut self, multiplier: f32) {
        self.brush.radius *= multiplier;
        if self.brush.radius < 0.5 {
            self.brush.radius = 0.5;
        }
    }

    pub fn draw(
        &self,
        ctx: &mut ggez::Context,
        canvas: &mut ggez::graphics::Canvas,
        camera: Camera,
    ) {
        let circle = ggez::graphics::Mesh::new_circle(
            ctx,
            ggez::graphics::DrawMode::stroke(0.5),
            self.world_coord.0,
            self.brush.radius,
            0.1,
            ggez::graphics::Color::WHITE,
        )
        .unwrap();
        canvas.draw(&circle, camera);
    }
}

/// Brush Radius Effect
impl Brush {
    /// Based on the brush radius and the celestial cell size, return a list of
    /// points in relative xy coordinates that the brush will affect.
    fn brush_positions(&self, celestial: &Celestial) -> Vec<RelXyPoint> {
        let center =
            RelXyPoint(self.get_world_coord().0) - RelXyPoint(celestial.get_world_coord().0);
        let begin_at = center - RelXyPoint::new(self.brush.radius, self.brush.radius);
        let end_at = center + RelXyPoint::new(self.brush.radius, self.brush.radius);
        let mut positions = Vec::new();
        let mut x = begin_at.0.x
            + celestial
                .data
                .get_element_dir()
                .get_coordinate_dir()
                .get_cell_width()
                / 2.0;
        while x < end_at.0.x {
            let mut y = begin_at.0.y
                + celestial
                    .data
                    .get_element_dir()
                    .get_coordinate_dir()
                    .get_cell_width()
                    / 2.0;
            while y < end_at.0.y {
                let pos = RelXyPoint::new(x, y);
                if pos.0.distance(center.0) < self.brush.radius {
                    positions.push(pos);
                }
                y += celestial
                    .data
                    .get_element_dir()
                    .get_coordinate_dir()
                    .get_cell_width();
            }
            x += celestial
                .data
                .get_element_dir()
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
    ) {
        let positions = self.brush_positions(celestial);
        for pos in positions {
            let element_dir = celestial.data.get_element_dir_mut();
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
