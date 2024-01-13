use bevy_ecs::system::Resource;
use ggegui::{
    egui::{self, Ui},
    Gui,
};
use ggez::{glam::Vec2, Context};


use crate::physics::{fallingsand::elements::element::ElementType, util::vectors::ScreenCoord};

use super::gui_trait::WindowTrait;

/// A window used to select an element to place
#[derive(Resource)]
pub struct ElementPicker {
    screen_coords: ScreenCoord,
    current_selection: ElementType,
    gui: Gui,
}

impl ElementPicker {
    pub fn new(ctx: &mut Context) -> Self {
        Self {
            screen_coords: ScreenCoord(Vec2 { x: 0.0, y: 0.0 }),
            current_selection: ElementType::Vacuum,
            gui: Gui::new(ctx),
        }
    }

    pub fn set_selection(&mut self, selection: ElementType) {
        self.current_selection = selection;
    }
    pub fn get_selection(&self) -> ElementType {
        self.current_selection
    }
}

impl WindowTrait for ElementPicker {
    fn get_offset(&self) -> ScreenCoord {
        self.screen_coords
    }

    fn set_offset(&mut self, screen_coords: ScreenCoord) {
        self.screen_coords = screen_coords;
    }

    fn get_gui(&self) -> &Gui {
        &self.gui
    }

    fn get_gui_mut(&mut self) -> &mut Gui {
        &mut self.gui
    }

    fn get_title(&self) -> &str {
        "Element Picker"
    }

    fn get_alignment(&self) -> egui::Align2 {
        egui::Align2::RIGHT_TOP
    }

    fn window(&mut self, ui: &mut Ui) {
        ui.label(format!("Current Selection: {:?}", self.current_selection));
        ui.separator();
        ui.label("Debug Elements");
        ui.radio_value(
            &mut self.current_selection,
            ElementType::DownFlier,
            "DownFlier",
        );
        ui.radio_value(
            &mut self.current_selection,
            ElementType::LeftFlier,
            "LeftFlier",
        );
        ui.radio_value(
            &mut self.current_selection,
            ElementType::RightFlier,
            "RightFlier",
        );
        ui.separator();
        ui.label("Elements");
        ui.radio_value(&mut self.current_selection, ElementType::Vacuum, "Vacuum");
        ui.radio_value(&mut self.current_selection, ElementType::Sand, "Sand");
        ui.radio_value(&mut self.current_selection, ElementType::Stone, "Stone");
        ui.radio_value(&mut self.current_selection, ElementType::Water, "Water");
    }
}
