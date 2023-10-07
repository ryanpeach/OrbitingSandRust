use ggez::glam::Vec2;
use ggez::graphics::Image;
use ggez::graphics::Vertex;
use ggez::Context;

/// Finds a point halfway between two points
pub fn interpolate_points(p1: &Vec2, p2: &Vec2) -> Vec2 {
    Vec2::new((p1.x + p2.x) * 0.5, (p1.x + p2.x) * 0.5)
}

/// A chunk that can be rendered and simulated
pub trait Chunk {
    /* Drawing */
    fn get_outline(&self, res: u16) -> Vec<Vec2>;
    fn get_positions(&self, res: u16) -> Vec<Vec2>;
    fn get_indices(&self, res: u16) -> Vec<u32>;
    fn get_uvs(&self, res: u16) -> Vec<Vec2>;
    fn get_texture(&self, ctx: &mut Context, res: u16) -> Image;
    fn get_vertices(&self, res: u16) -> Vec<Vertex> {
        let positions = self.get_positions(res);
        let uvs = self.get_uvs(res);
        let vertexes: Vec<Vertex> = positions
            .iter()
            .zip(uvs.iter())
            .map(|(p, uv)| Vertex {
                position: [p.x, p.y],
                uv: [uv.x, uv.y],
                color: [1.0, 1.0, 1.0, 1.0],
            })
            .collect();
        vertexes
    }

    /* Shape Parameter Getters */
    fn get_cell_radius(&self) -> f32;
    fn get_start_radius(&self) -> f32;
    fn get_end_radius(&self) -> f32;
    fn get_start_radial_theta(&self) -> f32;
    fn get_end_radial_theta(&self) -> f32;
    fn get_num_radial_lines(&self) -> usize;
    fn get_num_concentric_circles(&self) -> usize;
    fn total_size(&self) -> usize {
        self.get_num_radial_lines() * self.get_num_concentric_circles()
    }

    /* Identity */
    fn get_start_concentric_circle_layer_relative(&self) -> usize;
    fn get_start_concentric_circle_absolute(&self) -> usize;
    fn get_end_concentric_circle_absolute(&self) -> usize;
    fn get_end_concentric_circle_relative(&self) -> usize;
    fn get_end_radial_line(&self) -> usize;
    fn get_start_radial_line(&self) -> usize;
}
