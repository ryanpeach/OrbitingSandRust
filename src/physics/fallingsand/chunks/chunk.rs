use macroquad::models::Mesh;

/// A chunk that can be rendered and simulated
pub trait Chunk {
    fn get_mesh(&self) -> Mesh;
}
