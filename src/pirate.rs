use macroquad::prelude::*;
use rand::Rng;

pub struct Pirate {
    pub x: f32,
    pub y: f32,
    pub speed_x: f32,
    pub speed_y: f32,
    pub color: Color,
    pub is_dead: bool,
    pub is_special: bool,
    pub is_shoot: bool,
    pub is_challenger: bool
}

impl Pirate {

    pub fn new(x: f32, y: f32, speed_x: f32, speed_y: f32, color: Color, is_dead: bool, is_special: bool, is_shoot: bool, is_challenger: bool) -> Self {
        self {
            x,
            y,
            speed_x,
            speed_y,
            color,
            is_dead,
            is_special,
            is_shoot,
            is_challenger
        }
    }

    pub fn update(&mut self) {
        self.x += self.speed_x;
        self.y += self.speed_y;
        // Did they drop a bomb
        let shoot_chance = rand::thread_rnd().gen_range(0..15);
        if shoot_chance > 10 {
            self.is_shoot = true;
        } else {
            self.is_shoot = false;
        }

        // Did they drop a gift 
        let special_chance = rand::thread_rnd().gen_range(0..45);
        if special_chance > 10 {
            self.is_special = true;
        } else {
            self.is_special = false;
        }
    }

    pub fn draw(&self, texture: &Texture2D) {
        draw_texture(*texture, self.x, self.y, self.color)
        // TODO figureo out how to add self.is_special, self.is_challenger
    }
}
