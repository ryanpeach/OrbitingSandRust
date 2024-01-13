use std::{fmt::Debug, time::Duration};

use bevy::{core::FrameCount, time::Time};

/// A clock for physics objects
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
        Self {
            time: time.clone(),
            frame: frame.clone(),
        }
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
    /// Mostly used for testing
    pub fn update(&mut self, delta: Duration) {
        self.time.advance_by(delta);
        self.frame.0 += 1;
    }
}
