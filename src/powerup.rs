use macroquad::prelude::*;
use crate::config::{PowerupType, POWERUP_CONFIG};
use ::rand::Rng;

pub struct PowerUp {
    pub powerup_type: PowerupType,
    pub x: f32,
    pub y: f32,
    pub vel_y: f32,
    pub lifetime: f64,
    pub max_lifetime: f64,
    pub is_active: bool,
    pub width: f32,
    pub height: f32,
    pub rotation: f32,
    pub bob_timer: f64,
    pub glow_intensity: f32,
}

impl PowerUp {
    pub fn new(powerup_type: PowerupType, x: f32, y: f32) -> Self {
        let mut rng = ::rand::thread_rng();
        Self {
            powerup_type,
            x,
            y,
            vel_y: rng.gen_range(60.0..120.0),
            lifetime: 0.0,
            max_lifetime: 15.0,
            is_active: true,
            width: 24.0,
            height: 24.0,
            rotation: 0.0,
            bob_timer: rng.gen_range(0.0..std::f64::consts::TAU),
            glow_intensity: 0.0,
        }
    }

    pub fn update(&mut self, dt: f64) {
        if !self.is_active {
            return;
        }

        self.y += self.vel_y * dt as f32;
        self.lifetime += dt;
        self.rotation += 0.5 * dt as f32;
        self.bob_timer += dt;
        
        let bob = (self.bob_timer * 3.0).sin() as f32 * 0.5;
        self.glow_intensity = (self.bob_timer * 4.0).sin() as f32 * 0.3 + 0.7;

        if self.lifetime >= self.max_lifetime || self.y > screen_height() + 50.0 {
            self.is_active = false;
        }
    }

    pub fn draw(&self, texture: Option<&Texture2D>) {
        if !self.is_active {
            return;
        }

        let color = self.powerup_type.color();
        let glow_color = Color::new(color.r, color.g, color.b, self.glow_intensity * 0.5);

        if let Some(tex) = texture {
            draw_texture_ex(
                tex,
                self.x - self.width / 2.0,
                self.y - self.height / 2.0 + (self.bob_timer * 3.0).sin() as f32 * 2.0,
                color,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(self.width, self.height)),
                    rotation: self.rotation,
                    ..Default::default()
                }
            );
        } else {
            self.draw_procedural(color, glow_color);
        }

        if self.lifetime < 3.0 {
            let alpha = (3.0 - self.lifetime) / 3.0;
            draw_circle_lines(
                self.x,
                self.y + (self.bob_timer * 3.0).sin() as f32 * 2.0,
                self.width * 1.5,
                2.0,
                Color::new(color.r, color.g, color.b, alpha as f32 * 0.5),
            );
        }
    }

    fn draw_procedural(&self, color: Color, glow_color: Color) {
        let y_offset = (self.bob_timer * 3.0).sin() as f32 * 2.0;
        let draw_y = self.y + y_offset;

        match self.powerup_type {
            PowerupType::RapidFire => {
                draw_circle(self.x, draw_y, self.width / 2.0, glow_color);
                draw_rectangle(
                    self.x - 6.0, draw_y - 2.0, 12.0, 4.0, color
                );
                draw_rectangle(
                    self.x - 2.0, draw_y - 8.0, 4.0, 16.0, color
                );
            }
            PowerupType::SpreadShot => {
                draw_circle(self.x, draw_y, self.width / 2.0, glow_color);
                draw_triangle(
                    Vec2::new(self.x, draw_y - 8.0),
                    Vec2::new(self.x - 8.0, draw_y + 8.0),
                    Vec2::new(self.x + 8.0, draw_y + 8.0),
                    color,
                );
            }
            PowerupType::Pierce => {
                draw_circle(self.x, draw_y, self.width / 2.0, glow_color);
                draw_rectangle(
                    self.x - 8.0, draw_y - 1.0, 16.0, 2.0, color
                );
                draw_rectangle(
                    self.x - 8.0, draw_y + 1.0, 16.0, 2.0, color
                );
                draw_rectangle(
                    self.x - 1.0, draw_y - 8.0, 2.0, 16.0, WHITE
                );
            }
            PowerupType::Shield => {
                draw_circle(self.x, draw_y, self.width / 2.0, glow_color);
                draw_circle(self.x, draw_y, self.width / 2.0 - 4.0, Color::new(0.8, 0.8, 1.0, 0.5));
                draw_circle(self.x, draw_y, self.width / 2.0 - 8.0, color);
            }
            PowerupType::Bomb => {
                draw_circle(self.x, draw_y, self.width / 2.0, glow_color);
                draw_circle(self.x, draw_y, self.width / 2.0 - 2.0, color);
                draw_rectangle(
                    self.x - 1.0, draw_y - 8.0, 2.0, 16.0, WHITE
                );
                draw_rectangle(
                    self.x - 8.0, draw_y - 1.0, 16.0, 2.0, WHITE
                );
            }
            PowerupType::Life => {
                draw_circle(self.x, draw_y, self.width / 2.0, glow_color);
                let heart_points = [
                    Vec2::new(self.x, draw_y + 4.0),
                    Vec2::new(self.x - 6.0, draw_y - 2.0),
                    Vec2::new(self.x - 3.0, draw_y - 6.0),
                    Vec2::new(self.x, draw_y - 3.0),
                    Vec2::new(self.x + 3.0, draw_y - 6.0),
                    Vec2::new(self.x + 6.0, draw_y - 2.0),
                ];
                for i in 0..heart_points.len() {
                    let j = (i + 1) % heart_points.len();
                    draw_line(
                        heart_points[i].x, heart_points[i].y,
                        heart_points[j].x, heart_points[j].y,
                        3.0, color,
                    );
                }
            }
            PowerupType::Score => {
                draw_circle(self.x, draw_y, self.width / 2.0, glow_color);
                let star_points = 5;
                let outer_r = 10.0;
                let inner_r = 4.0;
                let mut points = Vec::new();
                for i in 0..star_points * 2 {
                    let angle = i as f32 * std::f32::consts::PI / star_points as f32 - std::f32::consts::FRAC_PI_2;
                    let r = if i % 2 == 0 { outer_r } else { inner_r };
                    points.push(Vec2::new(self.x + angle.cos() * r, draw_y + angle.sin() * r));
                }
                for i in 0..points.len() {
                    let j = (i + 1) % points.len();
                    draw_line(points[i].x, points[i].y, points[j].x, points[j].y, 2.0, color);
                }
            }
        }
    }

    pub fn get_rect(&self) -> (f32, f32, f32, f32) {
        (
            self.x - self.width / 2.0,
            self.y - self.height / 2.0,
            self.width,
            self.height,
        )
    }

    pub fn apply_effect(&self, ship: &mut crate::ship::Ship, cannonball_vec: &mut Vec<crate::cannonball::Cannonball>, pirate_vec: &mut Vec<crate::pirate::Pirate>, game_score: &mut i32, lives: &mut i32) -> Option<PowerupEffect> {
        let duration = POWERUP_CONFIG.iter()
            .find(|c| c.powerup_type == self.powerup_type)
            .map(|c| c.duration)
            .unwrap_or(10.0);

        match self.powerup_type {
            PowerupType::RapidFire => {
                Some(PowerupEffect::RapidFire(duration))
            }
            PowerupType::SpreadShot => {
                Some(PowerupEffect::SpreadShot(duration))
            }
            PowerupType::Pierce => {
                Some(PowerupEffect::Pierce(duration))
            }
            PowerupType::Shield => {
                ship.has_shield = true;
                Some(PowerupEffect::Shield)
            }
            PowerupType::Bomb => {
                for pirate in pirate_vec {
                    if !pirate.is_dead {
                        pirate.is_dead = true;
                    }
                }
                cannonball_vec.clear();
                Some(PowerupEffect::Bomb)
            }
            PowerupType::Life => {
                *lives = (*lives + 1).min(9);
                Some(PowerupEffect::Life)
            }
            PowerupType::Score => {
                *game_score += 500;
                Some(PowerupEffect::Score)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum PowerupEffect {
    RapidFire(f32),
    SpreadShot(f32),
    Pierce(f32),
    Shield,
    Bomb,
    Life,
    Score,
}

pub struct PowerUpManager {
    pub powerups: Vec<PowerUp>,
    pub active_effects: Vec<(PowerupEffect, f32)>,
    pub rapid_fire_timer: f32,
    pub spread_shot_timer: f32,
    pub pierce_timer: f32,
    pub has_shield: bool,
}

impl PowerUpManager {
    pub fn new() -> Self {
        Self {
            powerups: Vec::new(),
            active_effects: Vec::new(),
            rapid_fire_timer: 0.0,
            spread_shot_timer: 0.0,
            pierce_timer: 0.0,
            has_shield: false,
        }
    }

    pub fn update(&mut self, dt: f64, ship: &mut crate::ship::Ship, cannonball_vec: &mut Vec<crate::cannonball::Cannonball>, pirate_vec: &mut Vec<crate::pirate::Pirate>, game_score: &mut i32, lives: &mut i32) {
        let dt_f32 = dt as f32;

        self.powerups.retain_mut(|p| {
            p.update(dt);
            p.is_active
        });

        if self.rapid_fire_timer > 0.0 {
            self.rapid_fire_timer -= dt_f32;
        }
        if self.spread_shot_timer > 0.0 {
            self.spread_shot_timer -= dt_f32;
        }
        if self.pierce_timer > 0.0 {
            self.pierce_timer -= dt_f32;
        }

        self.active_effects.retain_mut(|(effect, timer)| {
            *timer -= dt_f32;
            *timer > 0.0
        });
    }

    pub fn spawn_powerup(&mut self, powerup_type: PowerupType, x: f32, y: f32) {
        self.powerups.push(PowerUp::new(powerup_type, x, y));
    }

    pub fn try_spawn_from_enemy(&mut self, enemy: &crate::enemy::Enemy, wave: u32) {
        let mut rng = ::rand::thread_rng();
        let base_chance = enemy.powerup_chance;
        let wave_bonus = (wave as f32 * 0.01).min(0.1);
        let total_chance = (base_chance + wave_bonus).min(0.2);
        
        if rng.gen_bool(total_chance as f64) {
            let config = &POWERUP_CONFIG[rng.gen_range(0..POWERUP_CONFIG.len())];
            self.spawn_powerup(config.powerup_type, enemy.x, enemy.y);
        }
    }

    pub fn check_pickup(&mut self, ship: &mut crate::ship::Ship, cannonball_vec: &mut Vec<crate::cannonball::Cannonball>, pirate_vec: &mut Vec<crate::pirate::Pirate>, game_score: &mut i32, lives: &mut i32) {
        let ship_rect = ship.get_rect();
        
        for i in (0..self.powerups.len()).rev() {
            if self.powerups[i].is_active {
                let p_rect = self.powerups[i].get_rect();
                if rects_overlap(ship_rect, p_rect) {
                    if let Some(effect) = self.powerups[i].apply_effect(ship, cannonball_vec, pirate_vec, game_score, lives) {
                        self.apply_effect(effect);
                    }
                    self.powerups.remove(i);
                }
            }
        }
    }

    fn apply_effect(&mut self, effect: PowerupEffect) {
        match effect {
            PowerupEffect::RapidFire(duration) => {
                self.rapid_fire_timer = duration;
                self.active_effects.push((effect, duration));
            }
            PowerupEffect::SpreadShot(duration) => {
                self.spread_shot_timer = duration;
                self.active_effects.push((effect, duration));
            }
            PowerupEffect::Pierce(duration) => {
                self.pierce_timer = duration;
                self.active_effects.push((effect, duration));
            }
            PowerupEffect::Shield => {
                self.has_shield = true;
                self.active_effects.push((effect, 0.0));
            }
            PowerupEffect::Bomb => {
                self.active_effects.push((effect, 0.0));
            }
            PowerupEffect::Life => {
                self.active_effects.push((effect, 0.0));
            }
            PowerupEffect::Score => {
                self.active_effects.push((effect, 0.0));
            }
        }
    }

    pub fn get_cannonball_cooldown(&self, base_cooldown: f64) -> f64 {
        if self.rapid_fire_timer > 0.0 {
            base_cooldown * 0.3
        } else {
            base_cooldown
        }
    }

    pub fn is_spread_shot_active(&self) -> bool {
        self.spread_shot_timer > 0.0
    }

    pub fn is_pierce_active(&self) -> bool {
        self.pierce_timer > 0.0
    }

    pub fn has_shield(&self) -> bool {
        self.has_shield
    }

    pub fn consume_shield(&mut self) -> bool {
        if self.has_shield {
            self.has_shield = false;
            self.active_effects.retain(|(e, _)| !matches!(e, PowerupEffect::Shield));
            true
        } else {
            false
        }
    }

    pub fn draw(&self, textures: &std::collections::HashMap<String, Texture2D>) {
        for powerup in &self.powerups {
            if powerup.is_active {
                let key = powerup.powerup_type.sprite_name().to_string();
                let tex = textures.get(&key);
                powerup.draw(tex);
            }
        }
    }

    pub fn draw_effect_indicators(&self) {
        let mut y = 145.0;
        
        if self.rapid_fire_timer > 0.0 {
            draw_text(&format!("RAPID FIRE: {:.1}s", self.rapid_fire_timer), 25.0, y, 18.0, YELLOW);
            y += 22.0;
        }
        if self.spread_shot_timer > 0.0 {
            draw_text(&format!("SPREAD SHOT: {:.1}s", self.spread_shot_timer), 25.0, y, 18.0, SKYBLUE);
            y += 22.0;
        }
        if self.pierce_timer > 0.0 {
            draw_text(&format!("PIERCE: {:.1}s", self.pierce_timer), 25.0, y, 18.0, LIME);
            y += 22.0;
        }
        if self.has_shield {
            draw_text("SHIELD ACTIVE", 25.0, y, 18.0, WHITE);
        }
    }
}

fn rects_overlap(r1: (f32, f32, f32, f32), r2: (f32, f32, f32, f32)) -> bool {
    r1.0 < r2.0 + r2.2 &&
    r1.0 + r1.2 > r2.0 &&
    r1.1 < r2.1 + r2.3 &&
    r1.1 + r1.3 > r2.1
}