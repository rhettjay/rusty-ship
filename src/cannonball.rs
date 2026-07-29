use macroquad::prelude::*;

pub struct Cannonball {
    pub x: f32,
    pub y: f32,
    pub speed: f32,
    pub color: Color,
    pub is_active: bool,
    pub is_dead: bool,
    pub damage: i32,
    pub width: f32,
    pub height: f32,
}

impl Cannonball {
    pub fn new(x: f32, y: f32, speed: f32, color: Color) -> Self {
        Self {
            x,
            y,
            speed,
            color,
            is_active: true,
            is_dead: false,
            damage: 1,
            width: 5.0,
            height: 15.0,
        }
    }
    
    pub fn update(&mut self) {
        self.y -= self.speed;
        if self.y < -50.0 {
            self.is_active = false;
        }
    }

    pub fn draw(&self) {
        if self.is_active {
            draw_rectangle(self.x, self.y - 5.0, self.width, self.height, self.color);
        }
    }

    pub fn get_rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.width, self.height)
    }
}