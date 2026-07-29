use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PowerupEffectType {
    RapidFire,
    SpreadShot,
    Pierce,
    Shield,
}

pub struct Ship {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub speed: f32,
    pub color: Color,
    pub gameover: bool,
    pub has_shield: bool,
    pub rapid_fire_timer: f32,
    pub spread_shot_timer: f32,
    pub pierce_timer: f32,
}

impl Ship {
    pub fn left(&mut self) {
        self.x -= self.speed;
        if self.x < 0.0 { self.x = 0.0; }
    }

    pub fn right(&mut self) {
        self.x += self.speed;
        let max_x = screen_width() - self.w;
        if self.x > max_x { self.x = max_x; }
    }

    pub fn draw(&self, texture: &Texture2D) {
        draw_texture(texture, self.x, self.y, self.color);
        if self.has_shield {
            draw_circle_lines(self.x + self.w / 2.0, self.y + self.h / 2.0, self.w.max(self.h) * 0.7, 2.0, Color::new(0.5, 0.8, 1.0, 0.8));
        }
    }

    pub fn get_rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.w, self.h)
    }

    pub fn has_shield(&self) -> bool {
        self.has_shield
    }

    pub fn consume_shield(&mut self) -> bool {
        if self.has_shield {
            self.has_shield = false;
            true
        } else {
            false
        }
    }

    pub fn apply_powerup(&mut self, effect: PowerupEffectType, duration: f32) {
        match effect {
            PowerupEffectType::RapidFire => self.rapid_fire_timer = duration.max(self.rapid_fire_timer),
            PowerupEffectType::SpreadShot => self.spread_shot_timer = duration.max(self.spread_shot_timer),
            PowerupEffectType::Pierce => self.pierce_timer = duration.max(self.pierce_timer),
            PowerupEffectType::Shield => self.has_shield = true,
        }
    }

    pub fn update_powerups(&mut self, dt: f32) {
        if self.rapid_fire_timer > 0.0 { self.rapid_fire_timer -= dt; }
        if self.spread_shot_timer > 0.0 { self.spread_shot_timer -= dt; }
        if self.pierce_timer > 0.0 { self.pierce_timer -= dt; }
    }

    pub fn is_rapid_fire_active(&self) -> bool { self.rapid_fire_timer > 0.0 }
    pub fn is_spread_shot_active(&self) -> bool { self.spread_shot_timer > 0.0 }
    pub fn is_pierce_active(&self) -> bool { self.pierce_timer > 0.0 }
}