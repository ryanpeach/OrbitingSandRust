use macroquad::color::WHITE;
use macroquad::models::Vertex;
use macroquad::prelude::{vec3, Vec2, Vec3};
use macroquad::texture::Texture2D;

/// Finds a point halfway between two points
pub fn interpolate_points(p1: &Vec3, p2: &Vec3) -> Vec3 {
    vec3((p1.x + p2.x) * 0.5, (p1.y + p2.y) * 0.5, 0.0)
}

/// A chunk that can be rendered and simulated
pub trait Chunk {
    /* Drawing */
    fn get_positions(&self) -> Vec<Vec3>;
    fn get_indices(&self) -> Vec<u16>;
    fn get_uvs(&self) -> Vec<Vec2>;
    fn get_texture(&self) -> Texture2D;
    fn get_vertices(&self) -> Vec<Vertex> {
        let positions = self.get_positions();
        let uvs = self.get_uvs();
        let mut vertices: Vec<Vertex> = Vec::new();
        for i in 0..positions.len() {
            vertices.push(Vertex {
                position: positions[i],
                uv: uvs[i],
                color: WHITE,
            });
        }
        vertices
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
