//! This module contains all the GUI related code.
//! Things that are drawn to via screen coordinates rather than world coordinates.

use bevy::app::{PluginGroup, PluginGroupBuilder};

pub mod brush;
pub mod camera_window;
pub mod element_picker;

pub struct GuiPluginGroup;

impl PluginGroup for GuiPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(camera_window::CameraWindowPlugin)
            .add(brush::BrushPlugin)
            .add(element_picker::ElementPickerPlugin)
    }
}
