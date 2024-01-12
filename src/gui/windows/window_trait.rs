use bevy_ecs::component::Component;
use ggegui::{
    egui::{self, Align2, Ui, Window},
    Gui,
};
use ggez::graphics::{Canvas, DrawParam};

use crate::physics::util::vectors::ScreenCoord;

/// A convienience trait for gui objects to make certain functionality common and consistent
pub trait WindowTrait: Send + Sync {
    /// This is where you should call egui::Window::new.
    /// You only need to define this, you don't ever need to call it.
    /// Set all the necessary parameters using custom setters
    fn window(&mut self, ui: &mut Ui);
}

#[derive(Component)]
pub struct DrawableGui {
    screen_coord: ScreenCoord,
    gui: Gui,
    alignment: Align2,
    title: String,
    window: Box<dyn WindowTrait>,
}

impl DrawableGui {
    pub fn new(
        screen_coord: ScreenCoord,
        gui: Gui,
        alignment: Align2,
        title: String,
        window: Box<dyn WindowTrait>,
    ) -> Self {
        Self {
            screen_coord,
            gui,
            alignment,
            title,
            window,
        }
    }

    pub fn get_offset(&self) -> ScreenCoord {
        self.screen_coord
    }

    pub fn set_offset(&mut self, screen_coords: ScreenCoord) {
        self.screen_coord = screen_coords;
    }

    pub fn get_gui(&self) -> &Gui {
        &self.gui
    }

    pub fn get_gui_mut(&mut self) -> &mut Gui {
        &mut self.gui
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn get_alignment(&self) -> Align2 {
        self.alignment
    }

    /// Call this in the draw function of the scene
    fn draw(&mut self, ctx: &mut ggez::Context, canvas: &mut Canvas) {
        let gui_ctx = self.get_gui_mut().ctx();
        egui::Window::new(self.get_title())
            .anchor(
                self.get_alignment(),
                [self.get_offset().0.x, self.get_offset().0.y],
            )
            .show(&gui_ctx, |ui| {
                self.window.window(ui);
            });
        self.get_gui_mut().update(ctx);
        canvas.draw(self.get_gui(), DrawParam::default());
    }
}
