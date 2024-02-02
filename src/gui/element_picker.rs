use bevy::{
    app::{App, Plugin, Update},
    ecs::system::{ResMut, Resource},
};
use bevy_egui::{
    egui::{self},
    EguiContexts,
};

use crate::physics::fallingsand::elements::element::ElementType;

/// This is a gui window that lets you select an element to place
pub struct ElementPickerPlugin;

impl Plugin for ElementPickerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ElementSelection::default());
        app.add_systems(Update, ElementSelection::element_picker_system);
    }
}

/// A window used to select an element to place
#[derive(Resource, Default)]
pub struct ElementSelection(pub ElementType);

impl ElementSelection {
    pub fn element_picker_system(
        mut contexts: EguiContexts,
        mut element_selection: ResMut<ElementSelection>,
    ) {
        egui::Window::new("Element Picker").show(contexts.ctx_mut(), |ui| {
            ui.label(format!("Current Selection: {:?}", element_selection.0));
            ui.separator();
            ui.label("Debug Elements");
            ui.radio_value(
                &mut element_selection.0,
                ElementType::DownFlier,
                "DownFlier",
            );
            ui.radio_value(
                &mut element_selection.0,
                ElementType::LeftFlier,
                "LeftFlier",
            );
            ui.radio_value(
                &mut element_selection.0,
                ElementType::RightFlier,
                "RightFlier",
            );
            ui.separator();
            ui.label("Elements");
            ui.radio_value(&mut element_selection.0, ElementType::Vacuum, "Vacuum");
            ui.radio_value(&mut element_selection.0, ElementType::Sand, "Sand");
            ui.radio_value(&mut element_selection.0, ElementType::Stone, "Stone");
            ui.radio_value(&mut element_selection.0, ElementType::Water, "Water");
        });
    }
}
