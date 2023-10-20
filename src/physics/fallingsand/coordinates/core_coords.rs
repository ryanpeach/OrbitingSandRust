use crate::physics::fallingsand::coordinates::chunk_coords::ChunkCoords;
use crate::physics::fallingsand::util::mesh::Square;
use crate::physics::fallingsand::util::vectors::ChunkIjkVector;
use ggez::glam::Vec2;
use ggez::graphics::Rect;

use std::f32::consts::PI;

/// The core is always the first layer
/// It defines the radius of all future layers cell_radius
/// All future layers have 2x their previous layers num_radial_lines
/// Making this num_radial_lines the only variable layer to layer
#[derive(Debug, Clone, Copy)]
pub struct CoreChunkCoords {
    radius: f32,
    num_radial_lines: usize,
}

/// The default constructor
/// 6 is a good number for num_radial_lines as it is divisible by 2 and 3
/// 1 is a normal default for radius, but if you wanted a "bigger" planet without
/// changing the "resolution" of its simulation, you could increase this
impl Default for CoreChunkCoords {
    fn default() -> Self {
        Self {
            radius: 1.0,
            num_radial_lines: 6,
        }
    }
}

impl CoreChunkCoords {
    pub fn new(radius: f32, num_radial_lines: usize) -> Self {
        Self {
            radius,
            num_radial_lines,
        }
    }

    /// The goal is to go from the center to the outer edge and then one "unit" around the circle
    /// each vertex triplet for the position.
    fn get_positions(&self) -> Vec<Square> {
        let mut vertices = Vec::new();

        // Outer vertices
        for i in 0..self.num_radial_lines {
            let angle1 = i as f32 * 2.0 * PI / self.num_radial_lines as f32;
            let angle2 = (i + 1) as f32 * 2.0 * PI / self.num_radial_lines as f32;
            let pos0 = Vec2::new(0.0, 0.0);
            let pos1 = Vec2::new(self.radius * angle1.cos(), self.radius * angle1.sin());
            let pos2 = Vec2::new(self.radius * angle2.cos(), self.radius * angle2.sin());
            vertices.push(Square {
                tl: pos0,
                tr: pos0,
                bl: pos2,
                br: pos1,
            });
        }

        vertices
    }

    /// Just the outer sphere
    fn get_outline(&self) -> Vec<Vec2> {
        let mut vertices = Vec::new();

        // Outer vertices
        for i in 0..=self.num_radial_lines {
            let angle1 = i as f32 * 2.0 * PI / self.num_radial_lines as f32;
            let pos = Vec2::new(self.radius * angle1.cos(), self.radius * angle1.sin());
            vertices.push(pos);
        }

        vertices
    }
}

impl ChunkCoords for CoreChunkCoords {
    fn get_outline(&self) -> Vec<Vec2> {
        self.get_outline()
    }
    /// Res does not matter at all for the core chunk
    fn get_positions(&self) -> Vec<Square> {
        self.get_positions()
    }
    fn get_cell_radius(&self) -> f32 {
        self.radius
    }
    fn get_start_radius(&self) -> f32 {
        0.0
    }
    fn get_layer_num(&self) -> usize {
        0
    }
    fn get_chunk_idx(&self) -> ChunkIjkVector {
        ChunkIjkVector::ZERO
    }
    fn get_end_radius(&self) -> f32 {
        self.radius
    }
    fn get_num_radial_lines(&self) -> usize {
        self.num_radial_lines
    }
    fn get_num_concentric_circles(&self) -> usize {
        1
    }
    fn get_start_radial_theta(&self) -> f32 {
        0.0
    }
    fn get_end_radial_theta(&self) -> f32 {
        2.0 * PI * self.radius
    }
    fn get_start_concentric_circle_absolute(&self) -> usize {
        0
    }
    fn get_start_concentric_circle_layer_relative(&self) -> usize {
        0
    }
    fn get_end_concentric_circle_absolute(&self) -> usize {
        1
    }
    fn get_end_concentric_circle_layer_relative(&self) -> usize {
        1
    }
    fn get_end_radial_line(&self) -> usize {
        self.num_radial_lines
    }
    fn get_start_radial_line(&self) -> usize {
        0
    }
    fn get_bounding_box(&self) -> Rect {
        Rect::new(
            -self.radius,
            -self.radius,
            self.radius * 2.0,
            self.radius * 2.0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_vertices() {
        let chunk = CoreChunkCoords::default();
        let positions = chunk.get_positions();
        let uvs = chunk.get_uvs();
        let indices = chunk.get_indices();
        // Triangle 1
        assert_eq!(positions[0], Vec2::new(0.0, 0.0));
        assert_eq!(uvs[0], Vec2::new(0.0, 0.0));
        assert_eq!(indices[0], 0);
        assert_eq!(positions[1], Vec2::new(1.0, 0.0));
        assert_eq!(uvs[1], Vec2::new(1.0 / 6.0, 1.0));
        assert_eq!(indices[1], 1);
        assert_eq!(
            positions[2],
            Vec2::new((1.0 * 2.0 * PI / 6.0).cos(), (1.0 * 2.0 * PI / 6.0).sin(),)
        );
        assert_eq!(uvs[2], Vec2::new(1.0 / 6.0, 0.0));
        assert_eq!(indices[2], 2);

        // Triangle 2
        assert_eq!(positions[3], Vec2::new(0.0, 0.0));
        assert_eq!(uvs[3], Vec2::new(1.0 / 6.0, 0.0));
        assert_eq!(indices[3], 3);
        assert_eq!(
            positions[4],
            Vec2::new((1.0 * 2.0 * PI / 6.0).cos(), (1.0 * 2.0 * PI / 6.0).sin(),)
        );
        assert_eq!(uvs[4], Vec2::new(2.0 / 6.0, 1.0));
        assert_eq!(indices[4], 4);
        assert_eq!(
            positions[5],
            Vec2::new((2.0 * 2.0 * PI / 6.0).cos(), (2.0 * 2.0 * PI / 6.0).sin(),)
        );
        assert_eq!(uvs[5], Vec2::new(2.0 / 6.0, 0.0));
        assert_eq!(indices[5], 5);

        // Triangle 3
        assert_eq!(positions[6], Vec2::new(0.0, 0.0));
        assert_eq!(uvs[6], Vec2::new(2.0 / 6.0, 0.0));
        assert_eq!(indices[6], 6);
        assert_eq!(
            positions[7],
            Vec2::new((2.0 * 2.0 * PI / 6.0).cos(), (2.0 * 2.0 * PI / 6.0).sin(),)
        );
        assert_eq!(uvs[7], Vec2::new(3.0 / 6.0, 1.0));
        assert_eq!(indices[7], 7);
        assert_eq!(
            positions[8],
            Vec2::new((3.0 * 2.0 * PI / 6.0).cos(), (3.0 * 2.0 * PI / 6.0).sin(),)
        );
        assert_eq!(uvs[8], Vec2::new(3.0 / 6.0, 0.0));
        assert_eq!(indices[8], 8);

        // Triangle 4
        assert_eq!(positions[9], Vec2::new(0.0, 0.0));
        assert_eq!(uvs[9], Vec2::new(3.0 / 6.0, 0.0));
        assert_eq!(indices[9], 9);
        assert_eq!(
            positions[10],
            Vec2::new((3.0 * 2.0 * PI / 6.0).cos(), (3.0 * 2.0 * PI / 6.0).sin(),)
        );
        assert_eq!(uvs[10], Vec2::new(4.0 / 6.0, 1.0));
        assert_eq!(indices[10], 10);
        assert_eq!(
            positions[11],
            Vec2::new((4.0 * 2.0 * PI / 6.0).cos(), (4.0 * 2.0 * PI / 6.0).sin(),)
        );
        assert_eq!(uvs[11], Vec2::new(4.0 / 6.0, 0.0));
        assert_eq!(indices[11], 11);

        // Triangle 5
        assert_eq!(positions[12], Vec2::new(0.0, 0.0));
        assert_eq!(uvs[12], Vec2::new(4.0 / 6.0, 0.0));
        assert_eq!(indices[12], 12);
        assert_eq!(
            positions[13],
            Vec2::new((4.0 * 2.0 * PI / 6.0).cos(), (4.0 * 2.0 * PI / 6.0).sin(),)
        );
        assert_eq!(uvs[13], Vec2::new(5.0 / 6.0, 1.0));
        assert_eq!(indices[13], 13);
        assert_eq!(
            positions[14],
            Vec2::new((5.0 * 2.0 * PI / 6.0).cos(), (5.0 * 2.0 * PI / 6.0).sin(),)
        );
        assert_eq!(uvs[14], Vec2::new(5.0 / 6.0, 0.0));
        assert_eq!(indices[14], 14);

        // Triangle 6
        assert_eq!(positions[15], Vec2::new(0.0, 0.0));
        assert_eq!(uvs[15], Vec2::new(5.0 / 6.0, 0.0));
        assert_eq!(indices[15], 15);
        assert_eq!(
            positions[16],
            Vec2::new((5.0 * 2.0 * PI / 6.0).cos(), (5.0 * 2.0 * PI / 6.0).sin(),)
        );
        assert_eq!(uvs[16], Vec2::new(6.0 / 6.0, 1.0));
        assert_eq!(indices[16], 16);
        assert_eq!(
            positions[17],
            Vec2::new((6.0 * 2.0 * PI / 6.0).cos(), (6.0 * 2.0 * PI / 6.0).sin(),)
        );
        assert_eq!(uvs[17], Vec2::new(6.0 / 6.0, 0.0));
        assert_eq!(indices[17], 17);
    }
}
