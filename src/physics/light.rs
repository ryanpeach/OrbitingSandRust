use bevy::app::{App, Plugin};

#[warn(missing_docs)]
#[warn(clippy::missing_docs_in_private_items)]
pub mod types;

/// The plugin for the light physics.
pub struct LightPlugin;

/// Implement the bevy plugin trait for the light plugin.
impl Plugin for LightPlugin {
    /// Build the light plugin.
    fn build(&self, _app: &mut App) {}
}
