use ggez::graphics::Color;

pub trait Element {
    fn get_color(&self) -> Color;
}
