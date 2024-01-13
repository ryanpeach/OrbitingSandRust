use ggegui::{
    egui::{self, Align2, Ui},
    Gui,
};
use ggez::graphics::{Canvas, DrawParam};


use crate::physics::util::vectors::ScreenCoord;

/// A convienience trait for gui objects to make certain functionality common and consistent
pub trait WindowTrait {
    // Getters and setters
    fn get_offset(&self) -> ScreenCoord;
    fn set_offset(&mut self, screen_coords: ScreenCoord);
    fn get_alignment(&self) -> Align2;
    fn get_gui(&self) -> &Gui;
    fn get_gui_mut(&mut self) -> &mut Gui;
    fn get_title(&self) -> &str;

    /// This is where you should call egui::Window::new.
    /// You only need to define this, you don't ever need to call it.
    /// Set all the necessary parameters using custom setters
    fn window(&mut self, ui: &mut Ui);

    /// Call this in the draw function of the scene
    fn draw(&mut self, ctx: &mut ggez::Context, canvas: &mut Canvas) {
        let gui_ctx = self.get_gui_mut().ctx();
        egui::Window::new(self.get_title())
            .anchor(
                self.get_alignment(),
                [self.get_offset().0.x, self.get_offset().0.y],
            )
            .show(&gui_ctx, |ui| {
                self.window(ui);
            });
        self.get_gui_mut().update(ctx);
        canvas.draw(self.get_gui(), DrawParam::default());
    }
}
