#![allow(dead_code)]

use ggez::event::{self, EventHandler};
use ggez::glam::Vec2;
use ggez::graphics::{self, Mesh, MeshData, Vertex};
use ggez::{Context, GameResult};

use crate::physics::fallingsand::chunks::radial_mesh::{RadialMesh, RadialMeshBuilder};

mod physics;

struct MainState {
    res: u16,
    radial_mesh: RadialMesh,
    all_vertices: Vec<Vec<Vertex>>,
    all_indices: Vec<Vec<u32>>,
}

impl MainState {
    fn new() -> GameResult<MainState> {
        let radial_mesh = RadialMeshBuilder::new()
            .cell_radius(1.0)
            .num_layers(10)
            .first_num_radial_lines(6)
            .second_num_concentric_circles(2)
            .build();

        let res = 6;
        let all_vertices = radial_mesh.get_vertexes(res);
        let all_indices = radial_mesh.get_indices(res);

        Ok(MainState {
            res,
            radial_mesh,
            all_vertices,
            all_indices,
        })
    }
}

impl EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        // Any logic updates go here
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::BLACK);
        let all_textures = self.radial_mesh.get_textures(ctx, self.res);

        for (i, texture) in all_textures.into_iter().enumerate() {
            let mesh = Mesh::from_data(
                ctx,
                MeshData {
                    vertices: &self.all_vertices[i],
                    indices: &self.all_indices[i][..],
                },
            );

            canvas.draw_textured_mesh(mesh, texture, graphics::DrawParam::new());
        }

        let fps_text = graphics::Text::new(format!("FPS: {}", ctx.time.fps()));
        canvas.draw(
            &fps_text,
            graphics::DrawParam::default().dest(Vec2::new(0.0, 0.0)),
        );

        let _ = canvas.finish(ctx);
        Ok(())
    }
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("drawing", "ggez");
    let (ctx, events_loop) = cb.build()?;
    let state = MainState::new().unwrap();
    event::run(ctx, events_loop, state)
}
