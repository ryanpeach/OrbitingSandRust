use bevy::ecs::schedule::Schedule;
use bevy::ecs::world::World;
use bevy::math::Vec2;
use ggez::event::winit_event::{Event, KeyboardInput, WindowEvent};
use ggez::graphics::{self, Color, DrawMode};
use ggez::input::keyboard;
use ggez::GameResult;
use ggez::{event, winit, Context};
use orbiting_sand::gui::brush::Brush;
use orbiting_sand::gui::windows::camera_window::CameraWindow;
use orbiting_sand::gui::windows::cursor_tooltip::CursorTooltip;
use orbiting_sand::gui::windows::element_picker::ElementPicker;
use orbiting_sand::nodes::camera::cam::{Camera, ScreenSize};
use orbiting_sand::nodes::celestials::earthlike::EarthLikeBuilder;
use orbiting_sand::physics::util::clock::GlobalClock;
use winit::event_loop::ControlFlow;

struct ContextResource {
    ctx: Context,
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("eventloop", "ggez");
    let (mut ctx, events_loop) = cb.build()?;
    let mut state = MainState::new(ctx)?;

    let mut position: f32 = 1.0;

    // Handle events. Refer to `winit` docs for more information.
    events_loop.run(move |mut event, _window_target, control_flow| {
        let ctx: &mut &Context = &mut state.world.get_resource().unwrap();

        if ctx.quit_requested {
            ctx.continuing = false;
        }
        if !ctx.continuing {
            *control_flow = ControlFlow::Exit;
            return;
        }

        *control_flow = ControlFlow::Poll;

        // This tells `ggez` to update it's internal states, should the event require that.
        // These include cursor position, view updating on resize, etc.
        event::process_event(ctx, &mut event);
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => ctx.request_quit(),
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(keycode),
                            ..
                        },
                    ..
                } => {
                    if let keyboard::KeyCode::Escape = keycode {
                        ctx.request_quit();
                    }
                }
                // `CloseRequested` and `KeyboardInput` events won't appear here.
                x => println!("Other window event fired: {x:?}"),
            },
            Event::MainEventsCleared => {
                // Tell the timer stuff a frame has happened.
                // Without this the FPS timer functions and such won't work.
                ctx.time.tick();

                // Update
                position += 1.0;

                // Draw
                ctx.gfx.begin_frame().unwrap();

                let mut canvas =
                    graphics::Canvas::from_frame(ctx, Color::from([0.1, 0.2, 0.3, 1.0]));

                let circle = graphics::Mesh::new_circle(
                    ctx,
                    DrawMode::fill(),
                    bevy::math::Vec2::new(0.0, 0.0),
                    100.0,
                    2.0,
                    Color::WHITE,
                )
                .unwrap();
                canvas.draw(&circle, bevy::math::Vec2::new(position, 380.0));

                canvas.finish(ctx).unwrap();
                ctx.gfx.end_frame().unwrap();

                // reset the mouse delta for the next frame
                // necessary because it's calculated cumulatively each cycle
                ctx.mouse.reset_delta();

                // Copy the state of the keyboard into the KeyboardContext and
                // the mouse into the MouseContext.
                // Not required for this example but important if you want to
                // use the functions keyboard::is_key_just_pressed/released and
                // mouse::is_button_just_pressed/released.
                ctx.keyboard.save_keyboard_state();
                ctx.mouse.save_mouse_state();

                ggez::timer::yield_now();
            }

            x => println!("Device event fired: {x:?}"),
        }
    });
}
