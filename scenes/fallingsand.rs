extern crate orbiting_sand;

use std::{env, path};

use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler};
use ggez::glam::Vec2;
use ggez::graphics::{self, FilterMode, Sampler};
use ggez::input::keyboard::{KeyCode, KeyInput};
use ggez::{Context, GameResult};

use mint::{Point2, Vector2};
use orbiting_sand::gui::camera_window::CameraWindow;
use orbiting_sand::gui::cursor_tooltip::CursorTooltip;
use orbiting_sand::physics::fallingsand::element_directory::ElementGridDir;
use orbiting_sand::physics::fallingsand::elements::element::Element;
use orbiting_sand::physics::fallingsand::elements::sand::Sand;

use orbiting_sand::nodes::camera::cam::Camera;
use orbiting_sand::nodes::celestial::Celestial;

use orbiting_sand::physics::fallingsand::coordinates::coordinate_directory::CoordinateDirBuilder;
use orbiting_sand::physics::fallingsand::elements::vacuum::Vacuum;
use orbiting_sand::physics::util::clock::Clock;
use orbiting_sand::physics::util::vectors::RelXyPoint;

// =================
// Helper methods
// =================

// ===================
// Main Game
// ==================
struct MainState {
    celestial: Celestial,
    camera: Camera,
    cursor_tooltip: CursorTooltip,
    camera_window: CameraWindow,
    current_time: Clock,
    mouse_down: bool,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        // Create the celestial
        let coordinate_dir = CoordinateDirBuilder::new()
            .cell_radius(1.0)
            .num_layers(10)
            .first_num_radial_lines(6)
            .second_num_concentric_circles(3)
            .build();
        let fill0: &dyn Element = &Vacuum::default();
        let fill1: &dyn Element = &Sand::default();
        let element_grid_dir = ElementGridDir::new_checkerboard(coordinate_dir, fill0, fill1);
        println!("Num elements: {}", element_grid_dir.get_total_num_cells());
        let celestial = Celestial::new(element_grid_dir);

        // Create the camera
        let _screen_size = ctx.gfx.drawable_size();
        let camera = Camera::new(Vec2::new(_screen_size.0, _screen_size.1));
        Ok(MainState {
            celestial,
            camera,
            cursor_tooltip: CursorTooltip::new(ctx),
            camera_window: CameraWindow::new(ctx),
            current_time: Clock::new(),
            mouse_down: false,
        })
    }

    fn set_element(&mut self, pos: Point2<f32>) {
        let coordinate_dir = self.celestial.get_element_dir().get_coordinate_dir();
        let coords = {
            let world_coord = self.camera.screen_to_world_coords(pos);
            match coordinate_dir.rel_pos_to_cell_idx(RelXyPoint(world_coord.into())) {
                Ok(coords) => coords,
                Err(coords) => coords,
            }
        };
        self.celestial.get_element_dir_mut().set_element(
            coords,
            Box::<Sand>::default(),
            self.current_time,
        );
    }
}

impl EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Update the gui
        self.camera_window.update(ctx, &self.camera);
        self.cursor_tooltip
            .update(ctx, &self.camera, &self.celestial);

        // Save the celestial if requested
        self.camera_window.save_optionally(ctx, &self.celestial);

        // Update the clock
        let delta_time = ctx.time.delta();
        self.current_time.update(delta_time);

        // Process the celestial
        self.celestial.process(self.current_time);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::BLACK);
        canvas.set_sampler(Sampler::from(FilterMode::Nearest));

        // Draw the celestial
        self.celestial.draw(ctx, &mut canvas, self.camera);
        if self.camera_window.get_outline() {
            self.celestial.draw_outline(ctx, &mut canvas, self.camera);
        }
        if self.camera_window.get_wireframe() {
            self.celestial.draw_wireframe(ctx, &mut canvas, self.camera);
        }

        // Draw the gui
        self.camera_window.draw(&mut canvas);
        self.cursor_tooltip.draw(&mut canvas);

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
                self.camera.move_by_screen_coords(Point2 { x: 0., y: 1. });
            }
            Some(KeyCode::A) => {
                self.camera.move_by_screen_coords(Point2 { x: 1., y: 0. });
            }
            Some(KeyCode::S) => {
                self.camera.move_by_screen_coords(Point2 { x: 0., y: -1. });
            }
            Some(KeyCode::D) => {
                self.camera.move_by_screen_coords(Point2 { x: -1., y: 0. });
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
            self.camera.zoom(Vector2 { x: 0.9, y: 0.9 });
        } else if _y < 0.0 {
            self.camera.zoom(Vector2 { x: 1.1, y: 1.1 });
        }
        Ok(())
    }

    fn mouse_motion_event(
        &mut self,
        _ctx: &mut Context,
        x: f32,
        y: f32,
        _dx: f32,
        _dy: f32,
    ) -> Result<(), ggez::GameError> {
        self.cursor_tooltip.set_pos(Point2 { x, y }, &self.camera);
        if self.mouse_down {
            self.set_element(Point2 { x, y });
        }
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _button: event::MouseButton,
        _x: f32,
        _y: f32,
    ) -> Result<(), ggez::GameError> {
        self.mouse_down = true;
        Ok(())
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        _button: event::MouseButton,
        _x: f32,
        _y: f32,
    ) -> Result<(), ggez::GameError> {
        self.mouse_down = false;
        Ok(())
    }
}

pub fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("saves");
        path
    } else {
        path::PathBuf::from("./saves")
    };

    let wm = WindowMode::default().dimensions(1920.0, 1080.0);
    let cb = ggez::ContextBuilder::new("drawing", "ggez")
        .add_resource_path(resource_dir)
        .window_mode(wm);
    let (mut ctx, events_loop) = cb.build()?;
    println!("Full filesystem info: {:#?}", ctx.fs);

    println!("Resource stats:");
    ctx.fs.print_all();
    let state = MainState::new(&mut ctx).unwrap();
    event::run(ctx, events_loop, state)
}
