use mint::Point2;

use crate::nodes::camera::cam::Camera;

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
