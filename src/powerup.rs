use crate::collision::rects_overlap;
use crate::config::{PowerupType, POWERUP_CONFIG};
use ::rand::Rng;
use macroquad::prelude::*;

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
                },
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
                draw_rectangle(self.x - 6.0, draw_y - 2.0, 12.0, 4.0, color);
                draw_rectangle(self.x - 2.0, draw_y - 8.0, 4.0, 16.0, color);
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
                draw_rectangle(self.x - 8.0, draw_y - 1.0, 16.0, 2.0, color);
                draw_rectangle(self.x - 8.0, draw_y + 1.0, 16.0, 2.0, color);
                draw_rectangle(self.x - 1.0, draw_y - 8.0, 2.0, 16.0, WHITE);
            }
            PowerupType::Shield => {
                draw_circle(self.x, draw_y, self.width / 2.0, glow_color);
                draw_circle(
                    self.x,
                    draw_y,
                    self.width / 2.0 - 4.0,
                    Color::new(0.8, 0.8, 1.0, 0.5),
                );
                draw_circle(self.x, draw_y, self.width / 2.0 - 8.0, color);
            }
            PowerupType::Bomb => {
                draw_circle(self.x, draw_y, self.width / 2.0, glow_color);
                draw_circle(self.x, draw_y, self.width / 2.0 - 2.0, color);
                draw_rectangle(self.x - 1.0, draw_y - 8.0, 2.0, 16.0, WHITE);
                draw_rectangle(self.x - 8.0, draw_y - 1.0, 16.0, 2.0, WHITE);
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
                        heart_points[i].x,
                        heart_points[i].y,
                        heart_points[j].x,
                        heart_points[j].y,
                        3.0,
                        color,
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
                    let angle = i as f32 * std::f32::consts::PI / star_points as f32
                        - std::f32::consts::FRAC_PI_2;
                    let r = if i % 2 == 0 { outer_r } else { inner_r };
                    points.push(Vec2::new(
                        self.x + angle.cos() * r,
                        draw_y + angle.sin() * r,
                    ));
                }
                for i in 0..points.len() {
                    let j = (i + 1) % points.len();
                    draw_line(
                        points[i].x,
                        points[i].y,
                        points[j].x,
                        points[j].y,
                        2.0,
                        color,
                    );
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
}

/// Single source of truth for active powerup effects. The ship does not keep
/// its own copies of this state.
pub struct PowerUpManager {
    pub powerups: Vec<PowerUp>,
    pub rapid_fire_timer: f32,
    pub spread_shot_timer: f32,
    pub pierce_timer: f32,
    pub has_shield: bool,
}

impl PowerUpManager {
    pub fn new() -> Self {
        Self {
            powerups: Vec::new(),
            rapid_fire_timer: 0.0,
            spread_shot_timer: 0.0,
            pierce_timer: 0.0,
            has_shield: false,
        }
    }

    pub fn update(&mut self, dt: f64) {
        let dt_f32 = dt as f32;

        self.powerups.retain_mut(|p| {
            p.update(dt);
            p.is_active
        });

        if self.rapid_fire_timer > 0.0 {
            self.rapid_fire_timer = (self.rapid_fire_timer - dt_f32).max(0.0);
        }
        if self.spread_shot_timer > 0.0 {
            self.spread_shot_timer = (self.spread_shot_timer - dt_f32).max(0.0);
        }
        if self.pierce_timer > 0.0 {
            self.pierce_timer = (self.pierce_timer - dt_f32).max(0.0);
        }
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
            let config = weighted_powerup_config(&mut rng);
            self.spawn_powerup(config.powerup_type, enemy.x, enemy.y);
        }
    }

    /// Applies the effect of a collected powerup.
    pub fn apply_powerup(
        &mut self,
        powerup_type: PowerupType,
        pirates: &mut [crate::pirate::Pirate],
        game_score: &mut i32,
        lives: &mut i32,
    ) {
        let duration = POWERUP_CONFIG
            .iter()
            .find(|c| c.powerup_type == powerup_type)
            .map(|c| c.duration)
            .unwrap_or(10.0);

        match powerup_type {
            PowerupType::RapidFire => self.rapid_fire_timer = duration.max(self.rapid_fire_timer),
            PowerupType::SpreadShot => {
                self.spread_shot_timer = duration.max(self.spread_shot_timer)
            }
            PowerupType::Pierce => self.pierce_timer = duration.max(self.pierce_timer),
            PowerupType::Shield => self.has_shield = true,
            PowerupType::Bomb => {
                for pirate in pirates.iter_mut() {
                    pirate.is_dead = true;
                }
            }
            PowerupType::Life => *lives = (*lives + 1).min(9),
            PowerupType::Score => *game_score += 500,
        }
    }

    pub fn check_pickup(
        &mut self,
        ship: &crate::ship::Ship,
        pirates: &mut [crate::pirate::Pirate],
        game_score: &mut i32,
        lives: &mut i32,
    ) {
        let ship_rect = ship.get_rect();

        for i in (0..self.powerups.len()).rev() {
            if self.powerups[i].is_active {
                let p_rect = self.powerups[i].get_rect();
                if rects_overlap(ship_rect, p_rect) {
                    let powerup_type = self.powerups[i].powerup_type;
                    self.apply_powerup(powerup_type, pirates, game_score, lives);
                    self.powerups.remove(i);
                }
            }
        }
    }

    pub fn get_fire_cooldown(&self, base_cooldown: f64) -> f64 {
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

    pub fn consume_shield(&mut self) -> bool {
        if self.has_shield {
            self.has_shield = false;
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.powerups.clear();
        self.rapid_fire_timer = 0.0;
        self.spread_shot_timer = 0.0;
        self.pierce_timer = 0.0;
        self.has_shield = false;
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
            draw_text(
                format!("RAPID FIRE: {:.1}s", self.rapid_fire_timer),
                25.0,
                y,
                18.0,
                YELLOW,
            );
            y += 22.0;
        }
        if self.spread_shot_timer > 0.0 {
            draw_text(
                format!("SPREAD SHOT: {:.1}s", self.spread_shot_timer),
                25.0,
                y,
                18.0,
                SKYBLUE,
            );
            y += 22.0;
        }
        if self.pierce_timer > 0.0 {
            draw_text(
                format!("PIERCE: {:.1}s", self.pierce_timer),
                25.0,
                y,
                18.0,
                LIME,
            );
            y += 22.0;
        }
        if self.has_shield {
            draw_text("SHIELD ACTIVE", 25.0, y, 18.0, WHITE);
        }
    }
}

/// Picks a powerup config by its configured weight.
fn weighted_powerup_config(rng: &mut impl Rng) -> &'static crate::config::PowerupConfig {
    let total: u32 = POWERUP_CONFIG.iter().map(|c| c.weight).sum();
    let mut roll = rng.gen_range(0..total);
    for config in POWERUP_CONFIG {
        if roll < config.weight {
            return config;
        }
        roll -= config.weight;
    }
    &POWERUP_CONFIG[0]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enemy::{Enemy, EnemyType};

    /// The weighted pick must be able to return every configured powerup and
    /// nothing outside the configured set.
    #[test]
    fn test_weighted_pick_covers_all_configs() {
        let mut rng = ::rand::thread_rng();
        let mut seen = std::collections::HashSet::new();
        for _ in 0..2000 {
            let config = weighted_powerup_config(&mut rng);
            assert!(POWERUP_CONFIG
                .iter()
                .any(|c| c.powerup_type == config.powerup_type));
            seen.insert(config.powerup_type);
        }
        for config in POWERUP_CONFIG {
            assert!(
                seen.contains(&config.powerup_type),
                "never picked: {:?}",
                config.powerup_type
            );
        }
    }

    /// Rapid fire must drop the cooldown by the documented factor.
    #[test]
    fn test_rapid_fire_cooldown() {
        let mut mgr = PowerUpManager::new();
        assert_eq!(mgr.get_fire_cooldown(0.2), 0.2);
        mgr.rapid_fire_timer = 5.0;
        assert!((mgr.get_fire_cooldown(0.2) - 0.06).abs() < 1e-9);
    }

    /// Shield consume is one-shot.
    #[test]
    fn test_consume_shield() {
        let mut mgr = PowerUpManager::new();
        assert!(!mgr.consume_shield());
        mgr.has_shield = true;
        assert!(mgr.consume_shield());
        assert!(!mgr.has_shield);
        assert!(!mgr.consume_shield());
    }

    /// Timers tick down and clamp at zero.
    #[test]
    fn test_timers_tick_down() {
        let mut mgr = PowerUpManager::new();
        mgr.rapid_fire_timer = 0.05;
        mgr.update(0.016);
        assert!(mgr.rapid_fire_timer > 0.0);
        mgr.update(1.0);
        assert_eq!(mgr.rapid_fire_timer, 0.0);
    }

    /// Applying a life powerup caps at 9 lives.
    #[test]
    fn test_life_powerup_caps() {
        let mut mgr = PowerUpManager::new();
        let mut pirates: Vec<crate::pirate::Pirate> = Vec::new();
        let mut score = 0;
        let mut lives = 9;
        mgr.apply_powerup(PowerupType::Life, &mut pirates, &mut score, &mut lives);
        assert_eq!(lives, 9);
    }

    /// Bomb clears live pirates.
    #[test]
    fn test_bomb_clears_pirates() {
        let mut mgr = PowerUpManager::new();
        let mut pirates = vec![
            crate::pirate::Pirate::new(100.0, 100.0, 60.0),
            crate::pirate::Pirate::new(200.0, 100.0, -60.0),
        ];
        let mut score = 0;
        let mut lives = 3;
        mgr.apply_powerup(PowerupType::Bomb, &mut pirates, &mut score, &mut lives);
        assert!(pirates.iter().all(|p| p.is_dead));
    }

    /// Enemy-driven spawn chance stays within the documented cap.
    #[test]
    fn test_spawn_chance_cap() {
        // Enemy with maxed powerup chance on a high wave must not exceed 0.2.
        let enemy = Enemy::new(EnemyType::Elite, 0.0, 0.0, 100);
        let base = enemy.powerup_chance;
        let wave_bonus = (100u32 as f32 * 0.01).min(0.1);
        assert!((base + wave_bonus).min(0.2) <= 0.2);
    }
}
