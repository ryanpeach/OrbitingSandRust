use bevy::prelude::SystemSet;

/// The set of all systems that create movement
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovementSet;

/// The set of all systems that draw things like gizmos.
/// Should run after [`MovementSet`]
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DrawSet;

/// The set of all systems that are `FixedUpdate` compute
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComputeSet;
