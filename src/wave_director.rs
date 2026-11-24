use crate::boss::Boss;
use crate::bullet::Bullet;
use crate::collision::rects_overlap;
use crate::config::CONFIG;
use crate::enemy::{select_enemy_type, Enemy};
use crate::pirate::Pirate;
use crate::powerup::PowerUpManager;
use ::rand::Rng;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

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

/// How many fresh pirates a single minion signal may spawn.
const MINION_SPAWN_CAP: usize = 3;

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
    /// High-water mark of simultaneous pirate minions, so "revive minions"
    /// can restore the roster to its recent size.
    minions_peak: usize,
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
            minions_peak: 0,
        }
    }

    /// (Re)starts a wave. All wave parameters come from the content tables;
    /// nothing about a wave is hardcoded here.
    pub fn start_wave(&mut self, wave: u32) {
        let def = crate::content::wave(wave);
        self.current_wave = wave;
        self.is_boss_wave = def.boss.is_some();
        self.wave_state = if self.is_boss_wave {
            WaveState::BossIntro
        } else {
            WaveState::Spawning
        };
        self.wave_timer = 0.0;
        self.wave_duration = def.duration;
        self.spawn_timer = 0.0;
        self.spawn_interval = def.spawn_interval;
        self.max_enemies = def.max_enemies;
        self.enemies_spawned = 0;
        self.powerup_manager.powerups.clear();
        self.current_boss = None;
        self.minions_peak = 0;
    }

    /// Restores in-progress wave bookkeeping on top of a `start_wave` for the
    /// same wave — used by save loading so spawn parameters always match the
    /// content tables instead of stale serialized copies.
    pub fn restore_in_progress(
        &mut self,
        wave_state: WaveState,
        wave_timer: f64,
        enemies_spawned: u32,
    ) {
        self.wave_state = wave_state;
        self.wave_timer = wave_timer;
        self.enemies_spawned = enemies_spawned;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        dt: f64,
        enemy_vec: &mut Vec<Enemy>,
        bullet_vec: &mut [Bullet],
        ship: &mut crate::ship::Ship,
        pirate_vec: &mut Vec<Pirate>,
        game_score: &mut i32,
        lives: &mut i32,
    ) {
        if self.wave_state == WaveState::Inactive {
            return;
        }

        self.wave_timer += dt;

        // Dispatch on wave STATE, not the is_boss_wave flag: the bonus boss
        // (Deadbeef) is installed mid-regular-wave and must still simulate.
        match self.wave_state {
            WaveState::BossIntro | WaveState::BossFight | WaveState::BossDefeated => {
                self.update_boss_wave(dt, bullet_vec, ship, game_score, lives);
            }
            _ => {
                self.update_regular_wave(dt, enemy_vec, pirate_vec);
            }
        }

        // Pirate minions always move, whatever the wave state.
        for pirate in pirate_vec.iter_mut() {
            pirate.update(dt as f32);
        }
        pirate_vec.retain(|p| !p.is_dead && p.y < screen_height());
        self.minions_peak = self.minions_peak.max(pirate_vec.len());

        self.powerup_manager.update(dt);
    }

    fn update_regular_wave(&mut self, dt: f64, enemy_vec: &mut Vec<Enemy>, pirate_vec: &[Pirate]) {
        match self.wave_state {
            WaveState::Spawning => {
                if self.enemies_spawned < self.max_enemies {
                    self.spawn_timer += dt;

                    if self.spawn_timer >= self.spawn_interval as f64 {
                        self.spawn_timer = 0.0;

                        let mut rng = ::rand::thread_rng();
                        let x = rng.gen_range(80.0..screen_width() - 80.0);
                        let enemy_type = select_enemy_type(self.current_wave, &mut rng);
                        let enemy = Enemy::new(enemy_type, x, -50.0, self.current_wave);
                        enemy_vec.push(enemy);
                        self.enemies_spawned += 1;
                    }
                } else {
                    self.wave_state = WaveState::Active;
                }
            }
            WaveState::Active => {
                let alive_count = enemy_vec.iter().filter(|e| !e.is_dead).count();
                let pirate_alive = pirate_vec.iter().filter(|p| !p.is_dead).count();

                if self.wave_timer >= self.wave_duration as f64
                    && alive_count == 0
                    && pirate_alive == 0
                {
                    self.wave_state = WaveState::Clearing;
                }
            }
            WaveState::Clearing => {
                let alive_count = enemy_vec.iter().filter(|e| !e.is_dead).count();
                let pirate_alive = pirate_vec.iter().filter(|p| !p.is_dead).count();

                if alive_count == 0 && pirate_alive == 0 {
                    self.wave_state = WaveState::Complete;
                }
            }
            _ => {}
        }
    }

    fn update_boss_wave(
        &mut self,
        dt: f64,
        bullet_vec: &mut [Bullet],
        ship: &mut crate::ship::Ship,
        game_score: &mut i32,
        lives: &mut i32,
    ) {
        match self.wave_state {
            WaveState::BossIntro => {}
            WaveState::BossFight => {
                let Some(boss) = &mut self.current_boss else {
                    return;
                };
                boss.update(dt, ship.x, ship.y); // Player bullets vs boss. hit_flash debounces damage so
                                                 // piercing bullets cannot drain the boss every frame.
                let boss_rect = boss.get_rect();
                for bullet in bullet_vec.iter_mut() {
                    if bullet.is_dead || !bullet.bullet_type.is_player() || !bullet.can_damage() {
                        continue;
                    }
                    if rects_overlap(bullet.get_rect(), boss_rect) && boss.can_take_damage() {
                        let killed = boss.take_damage(bullet.damage);
                        if !bullet.pierce() {
                            bullet.is_dead = true;
                        }
                        bullet.hit_flash = 0.1;
                        if killed {
                            *game_score += 1000;
                            self.wave_state = WaveState::BossDefeated;
                        }
                    }
                }

                // Boss projectiles vs ship.
                let mut hit_ship = false;
                if ship.invuln_timer <= 0.0 {
                    let ship_rect = ship.get_rect();
                    for projectile in boss.get_projectiles() {
                        if rects_overlap(projectile.get_rect(), ship_rect) {
                            hit_ship = true;
                            break;
                        }
                    }
                }
                if hit_ship {
                    if self.powerup_manager.has_shield {
                        self.powerup_manager.consume_shield();
                    } else if *lives > 0 {
                        *lives -= 1;
                        ship.invuln_timer = 2.0;
                    } else {
                        ship.gameover = true;
                    }
                }

                if boss.is_dead {
                    self.wave_state = WaveState::BossDefeated;
                }
            }
            _ => {}
        }
    }

    /// True while the active boss has the player's controls reversed.
    pub fn is_reverse_zone_active(&self) -> bool {
        self.current_boss
            .as_ref()
            .is_some_and(|b| b.is_reverse_zone_active())
    }

    /// Spawns a pirate minion at the top of the screen. `screen_w` is
    /// injected so tests can spawn without a GPU context.
    fn spawn_minion(pirate_vec: &mut Vec<Pirate>, screen_w: f32) {
        let mut rng = ::rand::thread_rng();
        let x = rng.gen_range(60.0..screen_w - 60.0);
        let speed_x = rng
            .gen_range(CONFIG.pirate_base_speed_x_range.0..CONFIG.pirate_base_speed_x_range.1)
            * if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
        pirate_vec.push(Pirate::new(x, 20.0, speed_x));
    }

    /// Consumes the boss's minion signals (spawn / revive), which is what
    /// makes Captain Davey's crew and Molly's revives actually appear.
    /// `screen_w` is injected so tests can run without a GPU context.
    pub fn drain_minion_signals(&mut self, pirate_vec: &mut Vec<Pirate>, screen_w: f32) {
        let Some(boss) = &mut self.current_boss else {
            return;
        };

        if boss.check_spawn_minions() {
            boss.clear_spawn_minions();
            for _ in 0..MINION_SPAWN_CAP {
                Self::spawn_minion(pirate_vec, screen_w);
            }
        }

        if boss.check_revive_signal() {
            boss.clear_revive_signal();
            // Restore the roster toward its recent peak (dead minions are
            // pruned immediately, so "revive" respawns fresh pirates).
            let target = self.minions_peak.min(pirate_vec.len() + MINION_SPAWN_CAP);
            while pirate_vec.len() < target {
                Self::spawn_minion(pirate_vec, screen_w);
            }
        }
    }

    pub fn check_powerup_pickup(
        &mut self,
        ship: &mut crate::ship::Ship,
        pirate_vec: &mut [Pirate],
        game_score: &mut i32,
        lives: &mut i32,
    ) {
        self.powerup_manager
            .check_pickup(ship, pirate_vec, game_score, lives);
    }

    pub fn try_spawn_powerup(&mut self, enemy: &Enemy) {
        self.powerup_manager
            .try_spawn_from_enemy(enemy, self.current_wave);
    }

    pub fn get_fire_cooldown(&self, base_cooldown: f64) -> f64 {
        self.powerup_manager.get_fire_cooldown(base_cooldown)
    }

    pub fn is_spread_shot_active(&self) -> bool {
        self.powerup_manager.is_spread_shot_active()
    }

    pub fn is_pierce_active(&self) -> bool {
        self.powerup_manager.is_pierce_active()
    }

    pub fn draw_wave_info(&self) {
        let wave_text = if self.is_boss_wave {
            format!("BOSS WAVE {}", self.current_wave)
        } else {
            format!("WAVE {}", self.current_wave)
        };
        draw_text(
            &wave_text,
            screen_width() * 0.5 - measure_text(&wave_text, None, 32, 1.0).width * 0.5,
            30.0,
            32.0,
            GOLD,
        );

        if cfg!(debug_assertions) && !self.is_boss_wave && self.wave_state != WaveState::Inactive {
            let progress = self.wave_timer / self.wave_duration as f64;
            let bar_w = 200.0;
            let bar_x = screen_width() * 0.5 - bar_w * 0.5;
            draw_rectangle(bar_x, 60.0, bar_w, 8.0, Color::new(0.2, 0.2, 0.2, 0.8));
            draw_rectangle(bar_x, 60.0, bar_w * progress as f32, 8.0, GREEN);
        }
    }

    pub fn is_wave_complete(&self) -> bool {
        matches!(
            self.wave_state,
            WaveState::Complete | WaveState::BossDefeated
        )
    }

    pub fn is_boss_active(&self) -> bool {
        matches!(self.wave_state, WaveState::BossIntro | WaveState::BossFight)
    }

    pub fn get_current_boss(&self) -> Option<&Boss> {
        self.current_boss.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boss::make_test_boss;

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

    /// start_wave must derive every parameter from the content tables.
    #[test]
    fn test_start_wave_uses_content_params() {
        let mut wd = WaveDirector::new();
        wd.start_wave(1);
        assert_eq!(wd.max_enemies, 6);
        assert_eq!(wd.spawn_interval, 2.0);
        assert_eq!(wd.wave_duration, 20.0);

        wd.start_wave(5);
        assert_eq!(wd.max_enemies, 0); // boss wave: no regular spawns
        assert!(wd.is_boss_wave);
    }

    /// restore_in_progress keeps content-derived params while overriding the
    /// serialized progress markers.
    #[test]
    fn test_restore_in_progress() {
        let mut wd = WaveDirector::new();
        wd.start_wave(2);
        wd.restore_in_progress(WaveState::Active, 12.5, 4);
        assert_eq!(wd.wave_state, WaveState::Active);
        assert_eq!(wd.wave_timer, 12.5);
        assert_eq!(wd.enemies_spawned, 4);
        // Content params survive the restore.
        assert_eq!(wd.max_enemies, 8);
        assert_eq!(wd.spawn_interval, 1.5);
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

    /// Minion signals must actually create pirates.
    #[test]
    fn test_drain_minion_signals_spawns_pirates() {
        let mut wd = WaveDirector::new();
        wd.start_wave(15);
        wd.current_boss = Some(make_test_boss(500, 500));
        wd.current_boss
            .as_mut()
            .unwrap()
            .pattern_state
            .insert("spawn_minions".to_string(), 1.0);

        let mut pirates: Vec<Pirate> = Vec::new();
        wd.drain_minion_signals(&mut pirates, 800.0);
        assert_eq!(pirates.len(), MINION_SPAWN_CAP);
        // Signal consumed — a second drain is a no-op.
        wd.drain_minion_signals(&mut pirates, 800.0);
        assert_eq!(pirates.len(), MINION_SPAWN_CAP);
    }

    /// Revive restores the roster toward its recent peak.
    #[test]
    fn test_drain_revive_signal() {
        let mut wd = WaveDirector::new();
        wd.start_wave(10);
        wd.current_boss = Some(make_test_boss(220, 220));

        let mut pirates: Vec<Pirate> = vec![
            Pirate::new(100.0, 100.0, 60.0),
            Pirate::new(200.0, 100.0, 60.0),
        ];
        // Simulate a recent peak of 5.
        wd.minions_peak = 5;

        wd.current_boss
            .as_mut()
            .unwrap()
            .pattern_state
            .insert("revive_signal".to_string(), 1.0);
        wd.drain_minion_signals(&mut pirates, 800.0);
        assert_eq!(pirates.len(), 5); // 2 + min(3, 5-2) = restored to peak
    }
}
