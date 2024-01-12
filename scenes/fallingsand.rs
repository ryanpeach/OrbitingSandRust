extern crate orbiting_sand;

use std::{env, path};

use bevy_ecs::schedule::Schedule;
use bevy_ecs::system::Resource;
use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler};
use ggez::glam::Vec2;
use ggez::graphics::{self, FilterMode, GraphicsContext, Sampler};
use ggez::input::keyboard::{KeyCode, KeyInput};
use ggez::{Context, GameResult};

use mint::{Point2, Vector2};
use orbiting_sand::gui::windows::camera_window::{CameraWindow, YesNoFullStep};
use orbiting_sand::gui::windows::cursor_tooltip::CursorTooltip;
use orbiting_sand::gui::windows::element_picker::ElementPicker;
use orbiting_sand::gui::windows::window_trait::WindowTrait;
use orbiting_sand::nodes::brush::Brush;
use orbiting_sand::physics::fallingsand::data::element_directory::ElementGridDir;

use orbiting_sand::nodes::camera::cam::Camera;
use orbiting_sand::nodes::celestial::{Celestial, CelestialData};

use bevy_ecs::world::World;
use orbiting_sand::physics::fallingsand::mesh::coordinate_directory::CoordinateDirBuilder;
use orbiting_sand::physics::util::clock::{Clock, GlobalClock};
use orbiting_sand::physics::util::vectors::WorldCoord;

struct MainState {
    world: World,
    schedule: Schedule,
    mouse_down: bool,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        // Create the world
        let mut world = World::default();

        // Create the celestial
        let coordinate_dir = CoordinateDirBuilder::new()
            .cell_radius(1.0)
            .num_layers(7)
            .first_num_radial_lines(12)
            .second_num_concentric_circles(3)
            .first_num_radial_chunks(3)
            .max_radial_lines_per_chunk(128)
            .max_concentric_circles_per_chunk(128)
            .build();
        let element_grid_dir = ElementGridDir::new_empty(coordinate_dir);
        println!("Num elements: {}", element_grid_dir.get_total_num_cells());
        let celestial_data = CelestialData::new(element_grid_dir);
        world.spawn(Celestial {
            data: celestial_data,
            world_coord: WorldCoord::default(),
        });

        // Create the camera
        let _screen_size = ctx.gfx.drawable_size();
        let camera = Camera::new(Vec2::new(_screen_size.0, _screen_size.1));
        world.insert_resource(camera);

        // Create the camera window
        let camera_window = CameraWindow::new(ctx);
        world.insert_resource(camera_window);

        // Create the cursor tooltip
        let cursor_tooltip = CursorTooltip::new(ctx, &camera);
        world.insert_resource(cursor_tooltip);

        // Create the element picker
        let element_picker = ElementPicker::new(ctx);
        world.insert_resource(element_picker);

        // Create the brush
        let brush = Brush::default();
        world.insert_resource(brush);

        // Create the global clock
        let current_time = GlobalClock::default();
        world.insert_resource(current_time);

        // Create the schedule
        let mut schedule = Schedule::default();

        // Return the world
        Ok(MainState {
            world,
            schedule,
            mouse_down: false,
        })
    }
}

impl EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.schedule.run(&mut self.world);
        // // Update the gui
        // self.camera_window.update(ctx, &self.camera, &self.brush);
        // self.cursor_tooltip.update(&self.camera, &self.celestial);

        // // Save the celestial if requested
        // self.camera_window.save_optionally(ctx, &self.celestial);

        // // Update the clock
        // let delta_time = ctx.time.delta();
        // self.current_time.update(delta_time);

        // // Process the celestial
        // match self.camera_window.should_i_process() {
        //     YesNoFullStep::Yes => self.celestial.data.process(self.current_time),
        //     YesNoFullStep::FullStep => self.celestial.data.process_full(self.current_time),
        //     YesNoFullStep::No => {}
        // }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::BLACK);
        canvas.set_sampler(Sampler::from(FilterMode::Nearest));

        // Draw the celestial
        self.celestial.data.draw(ctx, &mut canvas, self.camera);
        if self.camera_window.get_outline() {
            self.celestial
                .data
                .draw_outline(ctx, &mut canvas, self.camera);
        }
        if self.camera_window.get_wireframe() {
            self.celestial
                .data
                .draw_wireframe(ctx, &mut canvas, self.camera);
        }

        // Draw the gui
        self.camera_window.draw(ctx, &mut canvas);
        self.cursor_tooltip.draw(ctx, &mut canvas);
        self.element_picker.draw(ctx, &mut canvas);
        self.brush.draw(ctx, &mut canvas, self.camera);

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
                self.camera.move_by_screen_coords(Point2 { x: 0., y: 10. });
            }
            Some(KeyCode::A) => {
                self.camera.move_by_screen_coords(Point2 { x: 10., y: 0. });
            }
            Some(KeyCode::S) => {
                self.camera.move_by_screen_coords(Point2 { x: 0., y: -10. });
            }
            Some(KeyCode::D) => {
                self.camera.move_by_screen_coords(Point2 { x: -10., y: 0. });
            }
            Some(KeyCode::Equals) => {
                self.brush.mult_radius(2.0);
            }
            Some(KeyCode::Minus) => {
                self.brush.mult_radius(0.5);
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
        let position = Point2 { x, y };
        self.cursor_tooltip.set_offset(position);
        self.brush
            .set_position(self.camera.screen_to_world_coords(position));
        if self.mouse_down {
            self.brush
                .apply(&mut self.celestial, &self.element_picker, self.current_time)
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
