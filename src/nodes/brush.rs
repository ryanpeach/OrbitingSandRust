use mint::Point2;

use crate::{
    gui::windows::element_picker::ElementPicker,
    nodes::{camera::cam::Camera, celestial::Celestial, node_trait::NodeTrait},
    physics::util::{clock::Clock, vectors::RelXyPoint},
};

pub struct Brush {
    radius: f32,
    world_coord: Point2<f32>,
}

impl Default for Brush {
    fn default() -> Self {
        Self::new()
    }
}

impl Brush {
    pub fn new() -> Self {
        Self {
            radius: 0.5,
            world_coord: Point2 { x: 0.0, y: 0.0 },
        }
    }

    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
    }

    pub fn set_position(&mut self, world_coord: Point2<f32>) {
        self.world_coord = world_coord;
    }

    pub fn mult_radius(&mut self, multiplier: f32) {
        self.radius *= multiplier;
        if self.radius < 0.5 {
            self.radius = 0.5;
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
            self.world_coord,
            self.radius,
            0.1,
            ggez::graphics::Color::WHITE,
        )
        .unwrap();
        canvas.draw(&circle, camera);
    }
}

impl NodeTrait for Brush {
    fn get_world_coord(&self) -> Point2<f32> {
        self.world_coord
    }
}

/// Brush Radius Effect
impl Brush {
    /// Based on the brush radius and the celestial cell size, return a list of
    /// points in relative xy coordinates that the brush will affect.
    fn brush_positions(&self, celestial: &Celestial) -> Vec<RelXyPoint> {
        let center = RelXyPoint(self.get_world_coord().into())
            - RelXyPoint(celestial.get_world_coord().into());
        let begin_at = center - RelXyPoint::new(self.radius, self.radius);
        let end_at = center + RelXyPoint::new(self.radius, self.radius);
        let mut positions = Vec::new();
        let mut x = begin_at.0.x
            + celestial
                .get_element_dir()
                .get_coordinate_dir()
                .get_cell_width()
                / 2.0;
        while x < end_at.0.x {
            let mut y = begin_at.0.y
                + celestial
                    .get_element_dir()
                    .get_coordinate_dir()
                    .get_cell_width()
                    / 2.0;
            while y < end_at.0.y {
                let pos = RelXyPoint::new(x, y);
                if pos.0.distance(center.0) < self.radius {
                    positions.push(pos);
                }
                y += celestial
                    .get_element_dir()
                    .get_coordinate_dir()
                    .get_cell_width();
            }
            x += celestial
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
            let element_dir = celestial.get_element_dir_mut();
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
