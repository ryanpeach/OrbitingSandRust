use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct Clock {
    current_time: Duration,
    last_delta: Duration,
    current_frame: u64,
}

impl Default for Clock {
    fn default() -> Self {
        Self {
            current_time: Duration::new(0, 0),
            last_delta: Duration::new(0, 0),
            current_frame: 0,
        }
    }
}

impl Clock {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn get_current_time(&self) -> Duration {
        self.current_time
    }
    pub fn get_last_delta(&self) -> Duration {
        self.last_delta
    }
    pub fn get_current_frame(&self) -> u64 {
        self.current_frame
    }
    pub fn update(&mut self, delta: Duration) {
        self.current_time += delta;
        self.last_delta = delta;
        self.current_frame += 1;
    }
}
