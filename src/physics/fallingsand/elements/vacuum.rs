use ggez::graphics::Color;

use super::element::Element;

/// Literally nothing
#[derive(Default)]
pub struct Vacuum {}

impl Element for Vacuum {
    fn get_color(&self) -> Color {
        Color::new(1.0, 0.0, 0.0, 1.0)
    }
}
