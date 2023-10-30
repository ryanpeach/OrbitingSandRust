use ggegui::{egui, Gui, GuiContext};
use ggez::Context;
use mint::Point2;

use crate::physics::fallingsand::elements::element::ElementType;

use super::gui_trait::GuiTrait;

/// A window used to select an element to place
struct ElementPicker {
    screen_coords: Point2<f32>,
    current_selection: ElementType,
    gui: Gui,
}

impl ElementPicker {
    pub fn new(ctx: &mut Context) -> Self {
        Self {
            screen_coords: Point2 { x: 0.0, y: 0.0 },
            current_selection: ElementType::Vacuum,
            gui: Gui::new(ctx),
        }
    }

    pub fn set_selection(&mut self, selection: ElementType) {
        self.current_selection = selection;
    }
}

impl GuiTrait for ElementPicker {
    fn get_screen_coords(&self) -> Point2<f32> {
        self.screen_coords
    }

    fn set_screen_coords(&mut self, screen_coords: Point2<f32>) {
        self.screen_coords = screen_coords;
    }

    fn get_gui(&self) -> &Gui {
        &self.gui
    }

    fn get_gui_mut(&mut self) -> &mut Gui {
        &mut self.gui
    }

    fn window(&mut self, gui_ctx: &mut GuiContext) {
        egui::Window::new("Element Picker").show(gui_ctx, |ui| {
            ui.label(format!("Current Selection: {:?}", self.current_selection));
            ui.separator();
            ui.radio_value(
                &mut self.current_selection,
                ElementType::DownFlier,
                "DownFlier",
            );
            ui.separator();
            ui.radio_value(&mut self.current_selection, ElementType::Vacuum, "Vacuum");
            ui.radio_value(&mut self.current_selection, ElementType::Sand, "Sand");
        });
    }
}
