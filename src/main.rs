use macroquad::prelude::*;

mod physics;

#[macroquad::main("Sand Mesh")]
async fn main() {
    // let core = CoreChunk::default();
    // let core_mesh = core.get_mesh();
    // let first_layer = PartialLayerChunk::new(1.0, 0, 12,12, 2, 0, 1);
    // let first_layer = PartialLayerChunk::new(1.0, 1, 11, 12, 1, 1, 2);
    // let first_layer_mesh = first_layer.get_mesh();
    loop {
        // Set the scene
        clear_background(BLACK);
        set_camera(&Camera3D {
            position: vec3(0.0, 0.0, 10.0),
            up: vec3(0.0, 1.0, 0.0),
            target: vec3(0.0, 0.0, 0.0),
            ..Default::default()
        });

        // Draw each mesh
        // draw_mesh(&core_mesh);
        // draw_mesh(&first_layer_mesh);

        // Fin
        next_frame().await
    }
}
