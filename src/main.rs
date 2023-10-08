#![allow(dead_code)]

use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler};
use ggez::glam::*;
use ggez::graphics::{
    self, BlendMode, Canvas, Color, FilterMode, Mesh, MeshBuilder, MeshData, Sampler, Vertex,
};
use ggez::input::keyboard::{KeyCode, KeyInput};
use ggez::input::mouse::MouseButton;
use ggez::{Context, GameResult};

use crate::physics::fallingsand::chunks::radial_mesh::{RadialMesh, RadialMeshBuilder};

mod physics;

// ================================
// Create a camera implementation
// ================================
struct Camera {
    world_coords: Vec2,
    zoom: f32,
    zoom_speed: f32,
    min_zoom: f32,
    max_zoom: f32,
    rotation: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            world_coords: Vec2::new(0.0, 0.0),
            zoom: 1.0,
            zoom_speed: 1.1,
            min_zoom: 0.0, // Unbounded
            max_zoom: 100.0,
            rotation: 0.0,
        }
    }
}

impl Camera {
    pub fn zoom_in(&mut self) {
        self.zoom *= self.zoom_speed;
        if self.zoom > self.max_zoom {
            self.zoom = self.max_zoom;
        }
    }
    pub fn zoom_out(&mut self) {
        self.zoom /= self.zoom_speed;
        if self.zoom < self.min_zoom {
            self.zoom = self.min_zoom;
        }
    }
    pub fn move_up(&mut self) {
        self.world_coords.y += 2.0;
    }
    pub fn move_down(&mut self) {
        self.world_coords.y -= 2.0;
    }
    pub fn move_left(&mut self) {
        self.world_coords.x -= 2.0;
    }
    pub fn move_right(&mut self) {
        self.world_coords.x += 2.0;
    }
    pub fn rotate_left(&mut self) {
        self.rotation -= 0.1;
    }
    pub fn rotate_right(&mut self) {
        self.rotation += 0.1;
    }
}

// =================
// Helper methods
// =================
fn draw_triangle_wireframe(
    ctx: &mut Context,
    canvas: &mut Canvas,
    mesh_data: MeshData,
    draw_params: graphics::DrawParam,
) {
    let vertices = mesh_data.vertices;
    let indices = mesh_data.indices;

    for i in (0..indices.len()).step_by(3) {
        let i1 = indices[i] as usize;
        let i2 = indices[i + 1] as usize;
        let i3 = indices[i + 2] as usize;

        let p1 = vertices[i1].position;
        let p2 = vertices[i2].position;
        let p3 = vertices[i3].position;

        canvas.draw(
            &Mesh::new_line(ctx, &[p1, p2, p3, p1], 0.1, Color::WHITE).unwrap(),
            draw_params,
        );
    }
}

fn draw_uv_wireframe(
    ctx: &mut Context,
    canvas: &mut Canvas,
    mesh_data: MeshData,
    draw_params: graphics::DrawParam,
) {
    let vertices = mesh_data.vertices;
    let indices = mesh_data.indices;

    for i in (0..indices.len()).step_by(3) {
        let i1 = indices[i] as usize;
        let i2 = indices[i + 1] as usize;
        let i3 = indices[i + 2] as usize;

        let p1 = vertices[i1].uv;
        let p1_multiplied = Vec2::new(p1[0] * 10.0, p1[1] * 10.0);
        let p2 = vertices[i2].uv;
        let p2_multiplied = Vec2::new(p2[0] * 10.0, p2[1] * 10.0);
        let p3 = vertices[i3].uv;
        let p3_multiplied = Vec2::new(p3[0] * 10.0, p3[1] * 10.0);

        canvas.draw(
            &Mesh::new_line(
                ctx,
                &[p1_multiplied, p2_multiplied, p3_multiplied, p1_multiplied],
                0.1,
                Color::WHITE,
            )
            .unwrap(),
            draw_params,
        );
    }
}

// ===================
// Main Game
// ==================

struct MainState {
    res: u16,
    radial_mesh: RadialMesh,
    all_vertices: Vec<Vec<Vertex>>,
    all_indices: Vec<Vec<u32>>,
    all_outlines: Vec<Vec<Vec2>>,
    screen_width: f32,
    screen_height: f32,
    camera: Camera,
}

// Translates the world coordinate system, which
// has Y pointing up and the origin at the center,
// to the screen coordinate system, which has Y
// pointing downward and the origin at the top-left

fn world_to_screen_coords(screen_width: f32, screen_height: f32, point: Vec2) -> Vec2 {
    let x = point.x + screen_width / 2.0;
    let y = screen_height - (point.y + screen_height / 2.0);
    Vec2::new(x, y)
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let radial_mesh = RadialMeshBuilder::new()
            .cell_radius(1.0)
            .num_layers(9)
            .first_num_radial_lines(6)
            .second_num_concentric_circles(2)
            .build();

        let (width, height) = ctx.gfx.drawable_size();
        let draw_resolution = 0;
        let all_vertices = radial_mesh.get_vertexes(draw_resolution);
        let all_indices = radial_mesh.get_indices(draw_resolution);
        let all_outlines = radial_mesh.get_outlines(draw_resolution);
        println!("Nb of Meshes: {}", all_vertices.len());
        println!("Nb of Vertices: {}", all_vertices.iter().flatten().count());

        Ok(MainState {
            res: draw_resolution,
            radial_mesh,
            all_vertices,
            all_indices,
            all_outlines,
            screen_width: width,
            screen_height: height,
            camera: Camera::default(),
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
        canvas.set_sampler(Sampler::from(FilterMode::Nearest));
        let all_textures = self.radial_mesh.get_textures(ctx, self.res);

        let pos = world_to_screen_coords(
            self.screen_width,
            self.screen_height,
            self.camera.world_coords,
        );
        let draw_params = graphics::DrawParam::new()
            .dest(pos)
            .scale(Vec2::new(self.camera.zoom, self.camera.zoom))
            .rotation(self.camera.rotation)
            .offset(Vec2::new(0.5, 0.5));

        for (i, texture) in all_textures.into_iter().enumerate() {
            // Draw the mesh
            let mesh_data = MeshData {
                vertices: &self.all_vertices[i],
                indices: &self.all_indices[i][..],
            };
            let mesh = Mesh::from_data(ctx, mesh_data);
            canvas.draw_textured_mesh(mesh, texture, draw_params);
            // draw_uv_wireframe(ctx, &mut canvas, mesh_data, draw_params);
            // draw_triangle_wireframe(ctx, &mut canvas, mesh_data, draw_params);

            // Draw the outlines
            // for outline in &self.all_outlines {
            //     let mut mb = MeshBuilder::new();
            //     let line_mesh_data = mb.line(&outline[..], 0.1, Color::RED)?.build();
            //     let line_mesh = Mesh::from_data(ctx, line_mesh_data);
            //     canvas.draw(&line_mesh, draw_params);
            // }
        }

        let fps_text = graphics::Text::new(format!("FPS: {}", ctx.time.fps()));
        canvas.draw(
            &fps_text,
            graphics::DrawParam::default().dest(Vec2::new(0.0, 0.0)),
        );

        let _ = canvas.finish(ctx);
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        input: KeyInput,
        _repeated: bool,
    ) -> GameResult {
        match input.keycode {
            Some(KeyCode::W) => {
                self.camera.move_down();
            }
            Some(KeyCode::A) => {
                self.camera.move_right();
            }
            Some(KeyCode::S) => {
                self.camera.move_up();
            }
            Some(KeyCode::D) => {
                self.camera.move_left();
            }
            // Some(KeyCode::Q) => {
            //     self.camera.RotateLeft();
            // }
            // Some(KeyCode::E) => {
            //     self.camera.RotateRight();
            // }
            _ => (), // Do nothing
        }
        Ok(())
    }

    fn mouse_wheel_event(
        &mut self,
        _ctx: &mut Context,
        _x: f32,
        _y: f32,
    ) -> Result<(), ggez::GameError> {
        if _y > 0.0 {
            self.camera.zoom_in();
        } else if _y < 0.0 {
            self.camera.zoom_out();
        }
        Ok(())
    }
}

pub fn main() -> GameResult {
    let wm = WindowMode::default().dimensions(1920.0, 1080.0);
    let cb = ggez::ContextBuilder::new("drawing", "ggez").window_mode(wm);
    let (mut ctx, events_loop) = cb.build()?;
    let state = MainState::new(&mut ctx).unwrap();
    event::run(ctx, events_loop, state)
}
