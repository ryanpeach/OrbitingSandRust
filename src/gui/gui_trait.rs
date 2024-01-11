use crate::physics::util::vectors::ScreenCoord;

pub trait GuiTrait {
    fn get_screen_coord(&self) -> ScreenCoord;
}
