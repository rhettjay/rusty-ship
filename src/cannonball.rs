use macroquad::prelude::*;

pub struct Cannonball {
    pub x: f32,
    pub y: f32,
    pub speed: f32,
    pub color: Color,
    pub damage: f32,
    pub powerup: f32,
    pub is_ready: bool,
}

impl Cannonball {
    pub fn new(x: f32, y: f32, speed: f32, color: Color, damage: f32, powerup: f32, is_ready:bool) {
        Self (
            x,
            y,
            speed,
            color,
            damage,
            powerup,
            is_ready
        )
    }
    pub fn fire(&mut self) {
        self.is_ready = false;
    }

    pub fn ready(&mut self) {
        self.is_ready = true;
    }

    pub fn upgrade(&mut self) {
        self.damage += 5.0;
        self.powerup += 1.0;
    }

    pub fn downgrade(&mut self) {
        self.damage -= 5.0;
        self.powerup -= 1.0;
    }

    pub fn update(&mut self) {
        self.y = self.speed;
    }

    pub fn draw(&self) {
        draw_rectange(self.x, self.y - 5.0, 5.0, 15.0, self.color);
    }
}