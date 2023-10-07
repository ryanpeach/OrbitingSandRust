use crate::physics::fallingsand::chunks::radial_mesh::RadialMeshBuilder;
use macroquad::models::{Mesh, Vertex};
use macroquad::prelude::*;
use std::mem::replace;

mod physics;

#[macroquad::main("Sand Mesh")]
async fn main() {
    let radial_mesh = RadialMeshBuilder::new()
        .cell_radius(1.0)
        .num_layers(7)
        .first_num_radial_lines(6)
        .second_num_concentric_circles(2)
        .build();

    // Pre-compute all vertices and indices
    let mut all_indices: Vec<Vec<u16>> = radial_mesh.get_indices();
    let mut all_vertices: Vec<Vec<Vertex>> = radial_mesh.get_vertices();

    loop {
        // Set the scene
        clear_background(BLACK);
        set_camera(&Camera3D {
            position: vec3(0.0, 0.0, 10.0),
            up: vec3(0.0, 1.0, 0.0),
            target: vec3(0.0, 0.0, 0.0),
            projection: Projection::Orthographics,
            fovy: 360.0 * 2.0,
            ..Default::default()
        });

        // Generate new textures and draw them
        let all_textures = radial_mesh.get_textures();
        for (i, texture) in all_textures.into_iter().enumerate() {
            let vertices = replace(&mut all_vertices[i], Vec::new());
            let indices = replace(&mut all_indices[i], Vec::new());
            let mesh = Mesh {
                vertices,
                indices,
                texture: Some(texture),
            };
            draw_mesh(&mesh);
            let _ = replace(&mut all_vertices[i], mesh.vertices);
            let _ = replace(&mut all_indices[i], mesh.indices);
        }

        // Fin
        set_default_camera();
        draw_text(format!("FPS: {}", get_fps()).as_str(), 0., 16., 16.0, WHITE);
        next_frame().await
    }
}
