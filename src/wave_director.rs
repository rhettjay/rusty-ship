use macroquad::prelude::*;
use crate::enemy::{Enemy, select_enemy_type};
use crate::formation::{get_wave_formations, get_wave_duration, get_spawn_interval_for_wave};
use crate::powerup::{PowerUpManager, PowerupEffect};
use crate::config::PowerupType;
use crate::boss::{Boss, BossType};
use crate::bullet::Bullet;
use ::rand::Rng;
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum WaveState {
    Inactive,
    Spawning,
    Active,
    Clearing,
    Complete,
    BossIntro,
    BossFight,
    BossDefeated,
}

pub struct WaveDirector {
    pub current_wave: u32,
    pub wave_state: WaveState,
    pub wave_timer: f64,
    pub wave_duration: f32,
    pub spawn_timer: f64,
    pub spawn_interval: f32,
    pub max_enemies: u32,
    pub enemies_spawned: u32,
    pub powerup_manager: PowerUpManager,
    pub current_boss: Option<Boss>,
    pub is_boss_wave: bool,
}

impl WaveDirector {
    pub fn new() -> Self {
        Self {
            current_wave: 0,
            wave_state: WaveState::Inactive,
            wave_timer: 0.0,
            wave_duration: 0.0,
            spawn_timer: 0.0,
            spawn_interval: 2.0,
            max_enemies: 0,
            enemies_spawned: 0,
            powerup_manager: PowerUpManager::new(),
            current_boss: None,
            is_boss_wave: false,
        }
    }

    pub fn start_wave(&mut self, wave: u32) {
        self.current_wave = wave;
        self.is_boss_wave = matches!(wave, 5 | 10 | 15);
        self.wave_state = if self.is_boss_wave { WaveState::BossIntro } else { WaveState::Spawning };
        self.wave_timer = 0.0;
        self.wave_duration = get_wave_duration(wave);
        self.spawn_timer = 0.0;
        self.spawn_interval = get_spawn_interval_for_wave(wave);
        self.max_enemies = self.get_max_enemies_for_wave(wave);
        self.enemies_spawned = 0;
        self.powerup_manager.powerups.clear();
        self.current_boss = None;
    }

    pub fn get_max_enemies_for_wave(&self, wave: u32) -> u32 {
        match wave {
            1 => 6, 2 => 8, 3 => 10, 4 => 12,
            5 => 0,
            6 => 14, 7 => 16, 8 => 14, 9 => 16,
            10 => 0,
            11 => 18, 12 => 20, 13 => 22, 14 => 24,
            15 => 0,
            _ => 24,
        }
    }

    pub fn update(&mut self, dt: f64, enemy_vec: &mut Vec<Enemy>, bullet_vec: &mut Vec<Bullet>, ship: &mut crate::ship::Ship, cannonball_vec: &mut Vec<crate::cannonball::Cannonball>, pirate_vec: &mut Vec<crate::pirate::Pirate>, game_score: &mut i32, lives: &mut i32) {
        if self.wave_state == WaveState::Inactive {
            return;
        }

        self.wave_timer += dt;

        if self.is_boss_wave {
            self.update_boss_wave(dt, enemy_vec, bullet_vec, ship, cannonball_vec, pirate_vec, game_score, lives);
        } else {
            self.update_regular_wave(dt, enemy_vec, ship, cannonball_vec, pirate_vec, game_score, lives);
        }

        self.powerup_manager.update(dt, ship, cannonball_vec, pirate_vec, game_score, lives);
    }

    fn update_regular_wave(&mut self, dt: f64, enemy_vec: &mut Vec<Enemy>, ship: &mut crate::ship::Ship, cannonball_vec: &mut Vec<crate::cannonball::Cannonball>, pirate_vec: &mut Vec<crate::pirate::Pirate>, game_score: &mut i32, lives: &mut i32) {
        let mut rng = ::rand::thread_rng();
        
        match self.wave_state {
            WaveState::Spawning => {
                if self.enemies_spawned < self.max_enemies {
                    self.spawn_timer += dt;
                    
                    if self.spawn_timer >= self.spawn_interval as f64 {
                        self.spawn_timer = 0.0;
                        
                        let x = rng.gen_range(80.0..screen_width() - 80.0);
                        let y = -50.0;
                        let enemy_type = select_enemy_type(self.current_wave, &mut rng);
                        let enemy = Enemy::new(enemy_type, x, y, self.current_wave);
                        enemy_vec.push(enemy);
                        self.enemies_spawned += 1;
                    }
                } else {
                    self.wave_state = WaveState::Active;
                }
            }
            WaveState::Active => {
                let alive_count = enemy_vec.iter().filter(|e| !e.is_dead).count() as u32;
                let pirate_alive = pirate_vec.iter().filter(|p| !p.is_dead).count() as u32;
                
                if self.wave_timer >= self.wave_duration as f64 && (alive_count == 0 && pirate_alive == 0) {
                    self.wave_state = WaveState::Clearing;
                }
            }
            WaveState::Clearing => {
                let alive_count = enemy_vec.iter().filter(|e| !e.is_dead).count() as u32;
                let pirate_alive = pirate_vec.iter().filter(|p| !p.is_dead).count() as u32;
                
                if alive_count == 0 && pirate_alive == 0 {
                    self.wave_state = WaveState::Complete;
                }
            }
            _ => {}
        }
    }

    fn update_boss_wave(&mut self, dt: f64, enemy_vec: &mut Vec<Enemy>, bullet_vec: &mut Vec<Bullet>, ship: &mut crate::ship::Ship, cannonball_vec: &mut Vec<crate::cannonball::Cannonball>, pirate_vec: &mut Vec<crate::pirate::Pirate>, game_score: &mut i32, lives: &mut i32) {
        match self.wave_state {
            WaveState::BossIntro => {
            }
            WaveState::BossFight => {
                if let Some(boss) = &mut self.current_boss {
                    boss.update(dt, ship.x, ship.y);
                    
                    for bullet in bullet_vec.iter_mut() {
                        if bullet.is_dead || bullet.is_laser {
                            continue;
                        }
                        let (bx, by, bw, bh) = boss.get_rect();
                        let (bul_x, bul_y, bul_w, bul_h) = bullet.get_rect();
                        if bul_x < bx + bw && bul_x + bul_w > bx && bul_y < by + bh && bul_y + bul_h > by {
                            if boss.take_damage(bullet.damage) {
                                *game_score += 1000;
                                self.wave_state = WaveState::BossDefeated;
                            }
                            if !bullet.pierce() {
                                bullet.is_dead = true;
                            }
                            bullet.hit_flash = 0.1;
                        }
                    }
                    
                    for projectile in boss.get_projectiles() {
                        if ship.invuln_timer > 0.0 {
                            continue;
                        }
                        let (bx, by, bw, bh) = projectile.get_rect();
                        let (sx, sy, sw, sh) = ship.get_rect();
                        if bx < sx + sw && bx + bw > sx && by < sy + sh && by + bh > sy {
                            if ship.has_shield() {
                                ship.consume_shield();
                            } else {
                                *lives = lives.saturating_sub(1);
                                ship.invuln_timer = 2.0;
                                if *lives == 0 {
                                    ship.gameover = true;
                                }
                            }
                        }
                    }
                    
                    if boss.is_dead {
                        self.wave_state = WaveState::BossDefeated;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn check_powerup_pickup(&mut self, ship: &mut crate::ship::Ship, cannonball_vec: &mut Vec<crate::cannonball::Cannonball>, pirate_vec: &mut Vec<crate::pirate::Pirate>, game_score: &mut i32, lives: &mut i32) {
        self.powerup_manager.check_pickup(ship, cannonball_vec, pirate_vec, game_score, lives);
    }

    pub fn try_spawn_powerup(&mut self, enemy: &Enemy) {
        self.powerup_manager.try_spawn_from_enemy(enemy, self.current_wave);
    }

    pub fn get_cannonball_cooldown(&self, base_cooldown: f64) -> f64 {
        self.powerup_manager.get_cannonball_cooldown(base_cooldown)
    }

    pub fn is_spread_shot_active(&self) -> bool {
        self.powerup_manager.is_spread_shot_active()
    }

    pub fn is_pierce_active(&self) -> bool {
        self.powerup_manager.is_pierce_active()
    }

    pub fn draw(&self, textures: &std::collections::HashMap<String, Texture2D>) {
        if let Some(boss) = &self.current_boss {
            boss.draw();
        }
        self.powerup_manager.draw(textures);
        self.powerup_manager.draw_effect_indicators();
    }

    pub fn draw_wave_info(&self) {
        let wave_text = if self.is_boss_wave {
            format!("BOSS WAVE {}", self.current_wave)
        } else {
            format!("WAVE {}", self.current_wave)
        };
        draw_text(&wave_text, screen_width() * 0.5 - measure_text(&wave_text, None, 32, 1.0).width * 0.5, 30.0, 32.0, GOLD);

        if cfg!(debug_assertions) && !self.is_boss_wave && self.wave_state != WaveState::Inactive {
            let progress = self.wave_timer / self.wave_duration as f64;
            let bar_w = 200.0;
            let bar_x = screen_width() * 0.5 - bar_w * 0.5;
            draw_rectangle(bar_x, 60.0, bar_w, 8.0, Color::new(0.2, 0.2, 0.2, 0.8));
            draw_rectangle(bar_x, 60.0, bar_w * progress as f32, 8.0, GREEN);
        }
    }

    pub fn is_wave_complete(&self) -> bool {
        matches!(self.wave_state, WaveState::Complete | WaveState::BossDefeated)
    }

    pub fn is_boss_active(&self) -> bool {
        matches!(self.wave_state, WaveState::BossIntro | WaveState::BossFight)
    }

    pub fn get_current_boss(&self) -> Option<&Boss> {
        self.current_boss.as_ref()
    }

    pub fn get_current_boss_mut(&mut self) -> Option<&mut Boss> {
        self.current_boss.as_mut()
    }
}

fn rects_overlap(r1: (f32, f32, f32, f32), r2: (f32, f32, f32, f32)) -> bool {
    r1.0 < r2.0 + r2.2 &&
    r1.0 + r1.2 > r2.0 &&
    r1.1 < r2.1 + r2.3 &&
    r1.1 + r1.3 > r2.1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_wave_director() {
        let wd = WaveDirector::new();
        assert_eq!(wd.current_wave, 0);
        assert_eq!(wd.wave_state, WaveState::Inactive);
        assert!(!wd.is_boss_wave);
        assert!(wd.current_boss.is_none());
        assert!(!wd.is_wave_complete());
        assert!(!wd.is_boss_active());
    }

    #[test]
    fn test_boss_wave_detection() {
        let mut wd = WaveDirector::new();
        wd.start_wave(5);
        assert!(wd.is_boss_wave);
        assert_eq!(wd.current_wave, 5);
        assert_eq!(wd.wave_state, WaveState::BossIntro);
    }

    #[test]
    fn test_regular_wave_detection() {
        let mut wd = WaveDirector::new();
        wd.start_wave(1);
        assert!(!wd.is_boss_wave);
        assert_eq!(wd.current_wave, 1);
        assert_eq!(wd.wave_state, WaveState::Spawning);
    }

    #[test]
    fn test_wave_10_is_boss() {
        let mut wd = WaveDirector::new();
        wd.start_wave(10);
        assert!(wd.is_boss_wave);
    }

    #[test]
    fn test_wave_15_is_boss() {
        let mut wd = WaveDirector::new();
        wd.start_wave(15);
        assert!(wd.is_boss_wave);
    }

    #[test]
    fn test_get_max_enemies() {
        let wd = WaveDirector::new();
        assert_eq!(wd.get_max_enemies_for_wave(1), 6);
        assert_eq!(wd.get_max_enemies_for_wave(5), 0);
        assert_eq!(wd.get_max_enemies_for_wave(6), 14);
        assert_eq!(wd.get_max_enemies_for_wave(15), 0);
    }

    #[test]
    fn test_rects_overlap() {
        assert!(rects_overlap((0.0, 0.0, 10.0, 10.0), (5.0, 5.0, 10.0, 10.0)));
        assert!(!rects_overlap((0.0, 0.0, 5.0, 5.0), (10.0, 10.0, 5.0, 5.0)));
    }

    #[test]
    fn test_boss_active_check() {
        let mut wd = WaveDirector::new();
        assert!(!wd.is_boss_active());
        wd.wave_state = WaveState::BossIntro;
        assert!(wd.is_boss_active());
        wd.wave_state = WaveState::BossFight;
        assert!(wd.is_boss_active());
        wd.wave_state = WaveState::BossDefeated;
        assert!(!wd.is_boss_active());
    }

    #[test]
    fn test_wave_complete_check() {
        let mut wd = WaveDirector::new();
        assert!(!wd.is_wave_complete());
        wd.wave_state = WaveState::Complete;
        assert!(wd.is_wave_complete());
        wd.wave_state = WaveState::BossDefeated;
        assert!(wd.is_wave_complete());
    }
}