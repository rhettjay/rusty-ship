use macroquad::prelude::*;

pub struct Cannonball {
    pub x: f32,
    pub y: f32,
    pub speed: f32,
    pub color: Color,
    pub is_ready: bool,
}

impl Cannonball {
    pub fn new(x: f32, y: f32, speed: f32, color: Color) -> Self {
        Self {
            x,
            y,
            speed,
            color,
            is_ready: true,
        }
    }
    
    pub fn fire(&mut self) {
        self.is_ready = false;
    }

    pub fn ready(&mut self) {
        self.is_ready = true;
    }

    pub fn update(&mut self) {
        self.y -= self.speed;
    }

    pub fn draw(&self) {
        draw_rectangle(self.x, self.y - 5.0, 5.0, 15.0, self.color);
    }
}