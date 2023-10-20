extern crate orbiting_sand;

use ggegui::{egui, Gui};
use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler};
use ggez::glam::Vec2;
use ggez::graphics::{self, DrawParam, FilterMode, Sampler};
use ggez::input::keyboard::{KeyCode, KeyInput};
use ggez::{Context, GameResult};

use orbiting_sand::physics::fallingsand::element_directory::ElementGridDir;
use orbiting_sand::physics::fallingsand::elements::element::Element;
use orbiting_sand::physics::fallingsand::elements::sand::Sand;
use orbiting_sand::physics::fallingsand::elements::vacuum::Vacuum;
use orbiting_sand::physics::fallingsand::util::enums::MeshDrawMode;

use orbiting_sand::nodes::camera::Camera;
use orbiting_sand::nodes::celestial::Celestial;

use orbiting_sand::physics::fallingsand::coordinates::coordinate_directory::CoordinateDirBuilder;
use orbiting_sand::physics::util::clock::Clock;

// =================
// Helper methods
// =================

// ===================
// Main Game
// ==================
struct MainState {
    mesh_draw_mode: MeshDrawMode,
    celestial: Celestial,
    camera: Camera,
    gui: Gui,
    current_time: Clock,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let coordinate_dir = CoordinateDirBuilder::new()
            .cell_radius(1.0)
            .num_layers(9)
            .first_num_radial_lines(6)
            .second_num_concentric_circles(3)
            .build();
        let fill0: &dyn Element = &Vacuum::default();
        let fill1: &dyn Element = &Sand::default();
        let element_grid_dir = ElementGridDir::new_checkerboard(coordinate_dir, fill0, fill1);
        println!("Num elements: {}", element_grid_dir.get_total_num_cells());

        let celestial = Celestial::new(element_grid_dir, MeshDrawMode::TexturedMesh);
        let _screen_size = ctx.gfx.drawable_size();
        let camera = Camera::new(Vec2::new(_screen_size.0, _screen_size.1));
        Ok(MainState {
            celestial,
            camera,
            mesh_draw_mode: MeshDrawMode::TexturedMesh,
            gui: Gui::new(ctx),
            current_time: Clock::new(),
        })
    }
}

impl EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let gui_ctx = self.gui.ctx();

        // Handle res updates
        let mut mesh_draw_mode = self.mesh_draw_mode;
        egui::Window::new("Title").show(&gui_ctx, |ui| {
            ui.label(format!("zoom: {}", self.camera.get_zoom()));
            ui.label(format!("FPS: {}", ctx.time.fps()));
            // Set a radiomode for "DrawMode"
            ui.separator();
            ui.label("MeshDrawMode:");
            ui.radio_value(
                &mut mesh_draw_mode,
                MeshDrawMode::TexturedMesh,
                "TexturedMesh",
            );
            ui.radio_value(
                &mut mesh_draw_mode,
                MeshDrawMode::UVWireframe,
                "UVWireframe",
            );
            ui.radio_value(
                &mut mesh_draw_mode,
                MeshDrawMode::TriangleWireframe,
                "TriangleWireframe",
            );
            ui.radio_value(&mut mesh_draw_mode, MeshDrawMode::Outline, "Outline");
        });
        self.gui.update(ctx);

        if mesh_draw_mode != self.mesh_draw_mode {
            self.celestial.set_draw_mode(mesh_draw_mode);
            self.mesh_draw_mode = mesh_draw_mode;
        }
        let delta_time = ctx.time.delta();
        self.current_time.update(delta_time);
        self.celestial.process(self.current_time);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::BLACK);
        canvas.set_sampler(Sampler::from(FilterMode::Nearest));

        // Draw the celestial
        self.celestial.draw(ctx, &mut canvas, &self.camera);

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
