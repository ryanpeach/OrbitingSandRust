use bevy::{ecs::component::Component, math::Vec2, render::color::Color};

use crate::physics::{fallingsand::util::mesh::OwnedMeshData, util::vectors::Vertex};

/// Radius for circular entities
#[derive(Component, Debug, Clone, Copy)]
pub struct Radius(pub f32);

impl Radius {
    /// Calculate the mesh for the circle described by the radius
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
