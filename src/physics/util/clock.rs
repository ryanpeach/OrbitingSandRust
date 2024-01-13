use std::time::Duration;

use bevy::ecs::{bundle::Bundle, component::Component, system::Resource};

/// The current frame
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct Frame {
    pub frame: u64,
}

/// The current time in the game, taking into account pausing and ff, from bootup
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct InGameTime {
    pub time: Duration,
}

/// The delta between the last frame and the current frame
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct DeltaInGameTime {
    pub delta: Duration,
}

/// The time and delta for physics objects
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct PhysicsTime {
    pub current: InGameTime,
    pub delta: DeltaInGameTime,
}

/// A clock for physics objects
#[derive(Bundle, Default, Debug, Clone, Copy)]
pub struct Clock {
    time: PhysicsTime,
    frame: Frame,
}

/// The clock for the world itself
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct GlobalClock {
    pub clock: Clock,
}

impl Clock {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn get_current_time(&self) -> Duration {
        self.time.current.time
    }
    pub fn get_last_delta(&self) -> Duration {
        self.time.delta.delta
    }
    pub fn get_current_frame(&self) -> u64 {
        self.frame.frame
    }
    pub fn update(&mut self, delta: Duration) {
        self.time.current.time += delta;
        self.time.delta.delta = delta;
        self.frame.frame += 1;
    }
}
