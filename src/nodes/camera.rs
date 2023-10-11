use ggez::{glam::Vec2, graphics::Rect};

pub struct Camera {
    world_coords: Vec2,
    zoom: f32,
    zoom_speed: f32,
    min_zoom: f32,
    max_zoom: f32,
    rotation: f32,
    screen_size: Vec2,
}

impl Camera {
    pub fn new(screen_size: Vec2) -> Self {
        Self {
            world_coords: Vec2::new(0.0, 0.0),
            zoom: 1.0,
            zoom_speed: 1.1,
            min_zoom: 0.0, // Unbounded
            max_zoom: 100.0,
            // max_zoom: 7.0,
            rotation: 0.0,
            screen_size,
        }
    }

    pub fn get_bounding_box(&self) -> Rect {
        let screen_width = self.screen_size.x;
        let screen_height = self.screen_size.y;
        let world_width = screen_width / self.zoom;
        let world_height = screen_height / self.zoom;
        let world_x = self.world_coords.x;
        let world_y = self.world_coords.y;
        Rect::new(
            world_x - world_width / 2.0,
            world_y - world_height / 2.0,
            world_width,
            world_height,
        )
    }

    // ===========================
    // Getters
    // ===========================
    pub fn get_zoom(&self) -> f32 {
        self.zoom
    }
    pub fn get_world_coords(&self) -> Vec2 {
        self.world_coords
    }
    pub fn get_rotation(&self) -> f32 {
        self.rotation
    }
    pub fn get_screen_size(&self) -> Vec2 {
        self.screen_size
    }

    // ====================
    // Movement
    // ====================
    pub fn zoom_in(&mut self) {
        self.zoom *= self.zoom_speed;
        if self.zoom > self.max_zoom {
            self.zoom = self.max_zoom;
        }
    }
    pub fn zoom_out(&mut self) {
        self.zoom /= self.zoom_speed;
        if self.zoom < self.min_zoom {
            self.zoom = self.min_zoom;
        }
    }
    pub fn move_up(&mut self) {
        self.world_coords.y -= 2.0;
    }
    pub fn move_down(&mut self) {
        self.world_coords.y += 2.0;
        if self.world_coords.y > 0.0 {
            self.world_coords.y = 0.0;
        }
    }
    pub fn move_left(&mut self) {
        self.world_coords.x -= 2.0;
    }
    pub fn move_right(&mut self) {
        self.world_coords.x += 2.0;
    }
    pub fn rotate_left(&mut self) {
        self.rotation -= 0.1;
    }
    pub fn rotate_right(&mut self) {
        self.rotation += 0.1;
    }
}
