use ggegui::{Gui, GuiContext};
use ggez::graphics::{Canvas, DrawParam};
use mint::Point2;

/// A convienience trait for gui objects to make certain functionality common and consistent
pub trait GuiTrait {
    // Getters and setters
    fn get_screen_coords(&self) -> Point2<f32>;
    fn set_screen_coords(&mut self, screen_coords: Point2<f32>);
    fn get_gui(&self) -> &Gui;
    fn get_gui_mut(&mut self) -> &mut Gui;

    /// This is where you should call egui::Window::new.
    /// You only need to define this, you don't ever need to call it.
    /// Set all the necessary parameters using custom setters
    fn window(&mut self, gui_ctx: &mut GuiContext);

    /// Call this in the draw function of the scene
    fn draw(&mut self, ctx: &mut ggez::Context, canvas: &mut Canvas) {
        let mut gui_ctx = self.get_gui_mut().ctx();
        self.window(&mut gui_ctx);
        self.get_gui_mut().update(ctx);
        canvas.draw(
            self.get_gui(),
            DrawParam::default().dest(self.get_screen_coords()),
        );
    }
}
