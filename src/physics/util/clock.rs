//! A clock that can be passed to objects that need to know the last time they were updated.

use std::{fmt::Debug, time::Duration};

use bevy::{core::FrameCount, time::Time};

/// A clock that can be passed to objects that need to know the last time they were updated.
/// Combines the frame count and the time structs from the engine.
/// WARNING: We are reusing the frame count and Time structs from the engine.
///          however, since we are using getters this should be flexible if we change engines.
#[derive(Default, Clone, Copy)]
pub struct Clock {
    time: Time,
    frame: FrameCount,
}

impl Debug for Clock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Clock")
            .field("time", &self.time)
            .field("frame", &self.frame.0)
            .finish()
    }
}

impl Clock {
    pub fn new(time: Time, frame: FrameCount) -> Self {
        Self { time, frame }
    }
    pub fn get_current_time(&self) -> Duration {
        self.time.elapsed()
    }
    pub fn get_last_delta(&self) -> Duration {
        self.time.delta()
    }
    pub fn get_current_frame(&self) -> u32 {
        self.frame.0
    }
    /// Advances the clock by the given delta and one frame.
    /// Mostly used for testing
    pub fn update(&mut self, delta: Duration) {
        self.time.advance_by(delta);
        self.frame.0 += 1;
    }
}
