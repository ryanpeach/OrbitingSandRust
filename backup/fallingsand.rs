extern crate orbiting_sand;

use std::{env, path};

use bevy::ecs::schedule::Schedule;
use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, FilterMode, GraphicsContext, Sampler};
use ggez::input::keyboard::{KeyCode, KeyInput};
use ggez::{Context, GameResult};

use orbiting_sand::gui::brush::Brush;
use orbiting_sand::gui::windows::camera_window::{self, CameraWindow, YesNoFullStep};
use orbiting_sand::gui::windows::cursor_tooltip::{self, CursorTooltip};
use orbiting_sand::gui::windows::element_picker::ElementPicker;

use orbiting_sand::gui::windows::window_trait::WindowTrait;
use orbiting_sand::nodes::camera::cam::{Camera, ScreenSize};

use bevy::ecs::world::World;
use orbiting_sand::nodes::celestials::celestial::{Celestial, CelestialData};
use orbiting_sand::nodes::celestials::earthlike::EarthLikeBuilder;
use orbiting_sand::nodes::node_trait::WorldDrawable;
use orbiting_sand::physics::util::clock::GlobalClock;

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
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);
        canvas.set_sampler(Sampler::from(FilterMode::Nearest));

        // Get the camera
        let camera = self.world.get_resource::<Camera>().unwrap();

        // Draw the celestials
        let all_celestials = self.world.query::<WorldDrawable>();
        for celestial in all_celestials.iter(&self.world) {
            celestial.draw(ctx, &mut canvas, &camera);
        }

        // Draw the gui
        let camera_window = self.world.get_resource::<CameraWindow>().unwrap();
        camera_window.draw(ctx, &mut canvas);
        let cursor_tooltip = self.world.get_resource::<CursorTooltip>().unwrap();
        cursor_tooltip.draw(ctx, &mut canvas);
        let element_picker = self.world.get_resource::<ElementPicker>().unwrap();
        element_picker.draw(ctx, &mut canvas);
        let brush = self.world.get_resource::<Brush>().unwrap();
        brush.draw(ctx, &mut canvas, &camera);

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
