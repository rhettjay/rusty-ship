use macroquad::prelude::*;
use ::rand::Rng;

pub const PIRATE_ROTATION: f32 = 3.14159265359;

pub struct Pirate {
    pub x: f32,
    pub y: f32,
    pub speed_x: f32,
    pub speed_y: f32,
    pub color: Color,
    pub is_dead: bool,
    pub is_special: bool,
    pub is_shoot: bool,
    pub is_challenger: bool,
    pub width: f32,
    pub height: f32,
}

impl Pirate {
    pub fn new(x: f32, y: f32, speed_x: f32) -> Self {
        let mut rng = ::rand::thread_rng();
        Self {
            x,
            y,
            speed_x,
            speed_y: rng.gen_range(1.0..3.0),
            color: WHITE,
            is_dead: false,
            is_special: false,
            is_shoot: false,
            is_challenger: false,
            width: 32.0,
            height: 32.0,
        }
    }

    pub fn update(&mut self) {
        self.x += self.speed_x;
        self.y += self.speed_y;
        
        let mut rng = ::rand::thread_rng();
        let shoot_chance = rng.gen_range(0..15);
        self.is_shoot = shoot_chance > 10;
        
        let special_chance = rng.gen_range(0..45);
        self.is_special = special_chance > 10;
    }

    pub fn draw(&self, texture: &Texture2D) {
        draw_texture_ex(
            texture,
            self.x,
            self.y,
            self.color,
            DrawTextureParams {
                dest_size: Some(Vec2::new(self.width, self.height)),
                rotation: PIRATE_ROTATION,
                ..Default::default()
            }
        );
    }
    
    pub fn get_rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.width, self.height)
    }
}