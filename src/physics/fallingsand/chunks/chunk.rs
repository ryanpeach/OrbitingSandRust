use macroquad::models::Mesh;
use macroquad::prelude::{vec3, Vec3};

/// Finds a point halfway between two points
pub fn interpolate_points(p1: Vec3, p2: Vec3) -> Vec3 {
    vec3((p1.x + p2.x) * 0.5, (p1.y + p2.y) * 0.5, 0.0)
}

/// A chunk that can be rendered and simulated
pub trait Chunk {
    /* Drawing */
    fn get_mesh(&self) -> Mesh;

    /* Shape Parameter Getters */
    fn get_cell_radius(&self) -> f32;
    fn get_start_radius(&self) -> f32;
    fn get_end_radius(&self) -> f32;
    fn get_num_radial_lines(&self) -> usize;
    fn get_num_concentric_circles(&self) -> usize;
    fn total_size(&self) -> usize {
        return self.get_num_radial_lines() * self.get_num_concentric_circles();
    }

    /* Identity */
    fn get_layer_num(&self) -> usize;
    fn get_start_concentric_circle(&self) -> usize;
}
