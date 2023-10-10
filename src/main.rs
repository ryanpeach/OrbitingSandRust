#![allow(dead_code)]

use ggegui::{egui, Gui};
use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler};
use ggez::glam::*;
use ggez::graphics::{self, DrawParam, FilterMode, Image, Mesh, Sampler};
use ggez::input::keyboard::{KeyCode, KeyInput};
use ggez::{Context, GameResult};
use physics::fallingsand::chunks::radial_mesh::RadialMesh;
use physics::fallingsand::chunks::util::{DrawMode, OwnedMeshData, RawImage};

use crate::nodes::camera::Camera;
use crate::nodes::celestial::Celestial;

use crate::physics::fallingsand::chunks::radial_mesh::RadialMeshBuilder;

mod nodes;
mod physics;

// =================
// Helper methods
// =================

// ===================
// Main Game
// ==================
struct MainState {
    draw_mode: DrawMode,
    celestial: Celestial,
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
            .build();

        let celestial = Celestial::new(radial_mesh, DrawMode::TexturedMesh);
        let camera = Camera::default();
        let screen_size = ctx.gfx.drawable_size();
        Ok(MainState {
            celestial,
            camera,
            draw_mode: DrawMode::TexturedMesh,
            gui: Gui::new(ctx),
        })
    }
}

impl EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let gui_ctx = self.gui.ctx();

        // Handle res updates
        let mut draw_mode = self.draw_mode;
        egui::Window::new("Title").show(&gui_ctx, |ui| {
            ui.label(format!("zoom: {}", self.camera.get_zoom()));
            ui.label(format!("FPS: {}", ctx.time.fps()));
            // Set a radiomode for "DrawMode"
            ui.separator();
            ui.label("DrawMode:");
            ui.radio_value(&mut draw_mode, DrawMode::TexturedMesh, "TexturedMesh");
            ui.radio_value(&mut draw_mode, DrawMode::UVWireframe, "UVWireframe");
            ui.radio_value(
                &mut draw_mode,
                DrawMode::TriangleWireframe,
                "TriangleWireframe",
            );
            ui.radio_value(&mut draw_mode, DrawMode::Outline, "Outline");
        });
        self.gui.update(ctx);

        if draw_mode != self.draw_mode {
            self.celestial.set_draw_mode(draw_mode);
            self.draw_mode = draw_mode;
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

        let mesh = Mesh::from_data(ctx, self.celestial.get_combined_mesh().to_mesh_data());
        let img = self.celestial.get_combined_texture().to_image(ctx);
        match self.draw_mode {
            DrawMode::TexturedMesh => canvas.draw_textured_mesh(mesh, img, draw_params),
            DrawMode::TriangleWireframe => canvas.draw(&mesh, draw_params),
            DrawMode::UVWireframe => canvas.draw(&mesh, draw_params),
            DrawMode::Outline => canvas.draw(&mesh, draw_params),
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
