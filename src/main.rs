use macroquad::prelude::*;

mod physics;

use crate::physics::fallingsand::chunks::chunk::Chunk;
use crate::physics::fallingsand::chunks::core::CoreChunk;

#[macroquad::main("Sand Mesh")]
async fn main() {
    let chunk = CoreChunk::default();
    let mesh = chunk.get_mesh();
    loop {
        clear_background(BLACK);
        set_camera(&Camera3D {
            position: vec3(0.0, 0.0, 10.0),
            up: vec3(0.0, 1.0, 0.0),
            target: vec3(0.0, 0.0, 0.0),
            ..Default::default()
        });
        draw_mesh(&mesh);
        next_frame().await
    }
}
