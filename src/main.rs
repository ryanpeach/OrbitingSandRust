#![allow(dead_code)]

use ggegui::{egui, Gui};
use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler};
use ggez::glam::*;
use ggez::graphics::{
    self, Canvas, Color, DrawParam, FilterMode, Mesh, MeshData, Rect, Sampler, Vertex,
};
use ggez::input::keyboard::{KeyCode, KeyInput};
use ggez::{Context, GameResult};

use crate::nodes::camera::Camera;
use crate::physics::fallingsand::chunks::radial_mesh::{RadialMesh, RadialMeshBuilder};

mod nodes;
mod physics;

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
    radial_mesh: RadialMesh,
    all_vertices: Vec<Vec<Vertex>>,
    all_indices: Vec<Vec<u32>>,
    all_outlines: Vec<Vec<Vec2>>,
    bounding_boxes: Vec<Rect>,
    camera: Camera,
    gui: Gui,
}

// Translates the world coordinate system, which
// has Y pointing up and the origin at the center,
// to the screen coordinate system, which has Y
// pointing downward and the origin at the top-left

fn world_to_screen_coords(screen_size: Vec2, point: Vec2) -> Vec2 {
    let x = point.x + screen_size.x / 2.0;
    let y = screen_size.y - (point.y + screen_size.y / 2.0);
    Vec2::new(x, y)
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let radial_mesh = RadialMeshBuilder::new()
            .cell_radius(1.0)
            .num_layers(9)
            .first_num_radial_lines(6)
            .second_num_concentric_circles(2)
            .res(0)
            .build();

        let all_vertices = radial_mesh.get_vertexes();
        let all_indices = radial_mesh.get_indices();
        let all_outlines = radial_mesh.get_outlines();
        let bounding_boxes = radial_mesh.get_chunk_bounding_boxes();
        println!("Nb of Meshes: {}", all_vertices.len());
        println!("Nb of Vertices: {}", all_vertices.iter().flatten().count());

        Ok(MainState {
            radial_mesh,
            all_vertices,
            all_indices,
            all_outlines,
            bounding_boxes,
            camera: Camera::default(),
            gui: Gui::new(ctx),
        })
    }
}

impl EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let gui_ctx = self.gui.ctx();

        // Handle res updates
        let mut res = self.radial_mesh.get_res();
        egui::Window::new("Title").show(&gui_ctx, |ui| {
            ui.label(format!("zoom: {}", self.camera.get_zoom()));
            ui.label(format!("FPS: {}", ctx.time.fps()));

            ui.separator();
            ui.label("Resolution");
            // Create an integer selector
            ui.add(egui::Slider::new(&mut res, 0..=6).text("res"));
        });
        self.gui.update(ctx);

        if res != self.radial_mesh.get_res() {
            self.radial_mesh.set_res(res);
            self.all_vertices = self.radial_mesh.get_vertexes();
            self.all_indices = self.radial_mesh.get_indices();
            self.all_outlines = self.radial_mesh.get_outlines();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::BLACK);
        canvas.set_sampler(Sampler::from(FilterMode::Nearest));

        // Draw planets
        let screen_size = ctx.gfx.drawable_size();
        let pos = world_to_screen_coords(
            Vec2::new(screen_size.0, screen_size.1),
            self.camera.get_world_coords(),
        );
        let zoom = self.camera.get_zoom();
        let draw_params = graphics::DrawParam::new()
            .dest(pos)
            .scale(Vec2::new(zoom, zoom))
            .rotation(self.camera.get_rotation())
            .offset(Vec2::new(0.5, 0.5));

        for i in 0..self.radial_mesh.get_num_chunks() {
            if !self.bounding_boxes[i].overlaps(&self.camera.get_bounding_box(ctx)) {
                continue;
            }
            let texture = self.radial_mesh.get_texture(ctx, i);
            // Draw the mesh
            let mesh_data = MeshData {
                vertices: &self.all_vertices[i],
                indices: &self.all_indices[i][..],
            };
            let mesh = Mesh::from_data(ctx, mesh_data);
            canvas.draw_textured_mesh(mesh, texture, draw_params);
            // if i == 1 {
            // draw_uv_wireframe(ctx, &mut canvas, mesh_data, draw_params);
            // }
            // draw_triangle_wireframe(ctx, &mut canvas, mesh_data, draw_params);

            // Draw the outlines
            // for outline in &self.all_outlines {
            //     let mut mb = MeshBuilder::new();
            //     let line_mesh_data = mb.line(&outline[..], 0.1, Color::RED)?.build();
            //     let line_mesh = Mesh::from_data(ctx, line_mesh_data);
            //     canvas.draw(&line_mesh, draw_params);
            // }
        }

        // Draw gui
        canvas.draw(&self.gui, DrawParam::default().dest(Vec2::ZERO));

        let _ = canvas.finish(ctx);
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        input: KeyInput,
        _repeated: bool,
    ) -> GameResult {
        match input.keycode {
            Some(KeyCode::W) => {
                self.camera.move_up();
            }
            Some(KeyCode::A) => {
                self.camera.move_right();
            }
            Some(KeyCode::S) => {
                self.camera.move_down();
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
