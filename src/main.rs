use crate::physics::fallingsand::chunks::radial_mesh::RadialMeshBuilder;
use macroquad::prelude::*;

mod physics;

#[macroquad::main("Sand Mesh")]
async fn main() {
    let radial_mesh = RadialMeshBuilder::new()
        .cell_radius(1.0)
        .num_layers(9)
        .first_num_radial_lines(6)
        .second_num_concentric_circles(2)
        .build();
    let meshes = radial_mesh.get_meshes();

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

        // Draw each mesh
        for mesh in meshes.iter() {
            draw_mesh(mesh);
        }

        // Fin
        set_default_camera();
        draw_text(format!("FPS: {}", get_fps()).as_str(), 0., 16., 16.0, WHITE);
        next_frame().await
    }
}
