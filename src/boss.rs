use crate::content::{self, AttackDef, BossPhaseDef, BulletDef, MovementDef};
use ::rand::Rng;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum BossType {
    Blowfish,
    Twofish,
    RufusReverse,
    MollyHashpass,
    CaptainDavey,
    Deadbeef,
}

impl BossType {
    pub fn key(&self) -> &'static str {
        match self {
            BossType::Blowfish => "blowfish",
            BossType::Twofish => "twofish",
            BossType::RufusReverse => "rufus_reverse",
            BossType::MollyHashpass => "molly_hashpass",
            BossType::CaptainDavey => "captain_davey",
            BossType::Deadbeef => "deadbeef",
        }
    }

    pub fn from_key(key: &str) -> Option<BossType> {
        match key {
            "blowfish" => Some(BossType::Blowfish),
            "twofish" => Some(BossType::Twofish),
            "rufus_reverse" | "rufus" => Some(BossType::RufusReverse),
            "molly_hashpass" | "molly" => Some(BossType::MollyHashpass),
            "captain_davey" | "davey" => Some(BossType::CaptainDavey),
            "deadbeef" => Some(BossType::Deadbeef),
            _ => None,
        }
    }

    pub fn def(&self) -> &'static content::BossDef {
        content::boss(self.key())
            .unwrap_or_else(|| panic!("missing boss definition: {}", self.key()))
    }

    pub fn name(&self) -> &'static str {
        match self {
            BossType::Blowfish => "Blowfish",
            BossType::Twofish => "Twofish",
            BossType::RufusReverse => "Rufus Reverse",
            BossType::MollyHashpass => "Molly Hashpass",
            BossType::CaptainDavey => "Captain Davey Portscan",
            BossType::Deadbeef => "Deadbeef",
        }
    }

    pub fn sprite_filename(&self) -> &'static str {
        match self {
            BossType::Blowfish => "blowfish",
            BossType::Twofish => "twofish",
            BossType::RufusReverse => "rufus_reverse",
            BossType::MollyHashpass => "molly_hashpass",
            BossType::CaptainDavey => "captain_davey",
            BossType::Deadbeef => "deadbeef",
        }
    }
}

impl std::str::FromStr for BossType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        BossType::from_key(s).ok_or(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BossPhase {
    Phase1,
    Phase2,
    Phase3,
}

#[derive(Clone, Copy, Debug)]
pub struct BossProjectile {
    pub x: f32,
    pub y: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub size: f32,
    pub color: Color,
    pub lifetime: f64,
    pub pattern: ProjectilePattern,
    /// Deadbeef's thrown food draws as tumbling squares instead of circles.
    pub food: bool,
}

impl BossProjectile {
    pub fn get_rect(&self) -> (f32, f32, f32, f32) {
        (
            self.x - self.size / 2.0,
            self.y - self.size / 2.0,
            self.size,
            self.size,
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ProjectilePattern {
    Straight,
    Spiral,
    Homing,
    Bounce,
    Wave,
}

impl ProjectilePattern {
    fn from_key(key: &str) -> ProjectilePattern {
        match key {
            "spiral" => ProjectilePattern::Spiral,
            "homing" => ProjectilePattern::Homing,
            "bounce" => ProjectilePattern::Bounce,
            "wave" => ProjectilePattern::Wave,
            _ => ProjectilePattern::Straight,
        }
    }
}

/// Content-authored speeds are pixels-per-frame at 60 fps.
const FPS_SCALE: f32 = 60.0;
/// Boss entry descent, pixels per second.
const ENTRY_SPEED: f32 = 120.0;
/// Damage-debounce window after the boss takes a hit, seconds.
const HIT_INVULN: f64 = 0.1;
/// How long Davey's beam column telegraphs before firing, seconds.
const BEAM_TELEGRAPH_TIME: f32 = 0.7;
/// Post-defeat explosion duration, seconds.
const DEATH_FX_TIME: f32 = 1.0;

/// Davey's scanning beam telegraph: the column position is locked in, the
/// beam fires when the timer expires.
struct BeamTelegraph {
    remaining: f32,
    x: f32,
    bullet: BulletDef,
}

pub struct Boss {
    pub boss_type: BossType,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub health: i32,
    pub max_health: i32,
    pub phase: BossPhase,
    pub phase_timer: f64,
    pub attack_timer: f64,
    pub move_timer: f64,
    pub speed_x: f32,
    pub is_dead: bool,
    pub invulnerable: bool,
    pub invuln_timer: f64,
    pub projectiles: Vec<BossProjectile>,
    pub pattern_state: HashMap<String, f32>,
    pub movement: MovementDef,
    pub phases: Vec<BossPhaseDef>,
    pub sprite: Option<Texture2D>,
    pub color: Color,
    pub entry_anim: bool,
    // --- visual fx (seconds remaining; 0 = inactive) ---
    /// Phase-transition flash + shockwave ring.
    pub phase_flash: f32,
    /// Blowfish charge puff-up.
    inflate_fx: f32,
    /// Twofish mirrored ghost sprite.
    mirror_fx: f32,
    /// Rufus teleport chromatic glitch.
    glitch_fx: f32,
    /// Molly heal ring.
    heal_fx: f32,
    /// Post-defeat explosion countdown; the fight settles when it hits 0.
    pub death_timer: f32,
    beam_telegraph: Option<BeamTelegraph>,
}

impl Boss {
    pub fn new(boss_type: BossType) -> Self {
        let def = boss_type.def();
        let (width, height) = def.size;

        let mut boss = Self {
            boss_type,
            x: screen_width() / 2.0 - width / 2.0,
            y: -height,
            width,
            height,
            health: def.max_health,
            max_health: def.max_health,
            phase: BossPhase::Phase1,
            phase_timer: 0.0,
            attack_timer: 0.0,
            move_timer: 0.0,
            speed_x: match &def.movement {
                MovementDef::Patrol { speed_x, .. } => *speed_x * FPS_SCALE,
                _ => 120.0,
            },
            is_dead: false,
            invulnerable: false,
            invuln_timer: 0.0,
            projectiles: Vec::new(),
            pattern_state: HashMap::new(),
            movement: def.movement.clone(),
            phases: def.phases.clone(),
            sprite: None,
            color: WHITE,
            entry_anim: true,
            phase_flash: 0.0,
            inflate_fx: 0.0,
            mirror_fx: 0.0,
            glitch_fx: 0.0,
            heal_fx: 0.0,
            death_timer: 0.0,
            beam_telegraph: None,
        };

        boss.init_pattern_state();
        boss
    }

    fn init_pattern_state(&mut self) {
        match self.boss_type {
            BossType::Blowfish => {
                self.pattern_state.insert("burst_count".to_string(), 0.0);
                self.pattern_state.insert("charge_ready".to_string(), 0.0);
            }
            BossType::Twofish => {
                self.pattern_state.insert("mirror_offset".to_string(), 0.0);
                self.pattern_state.insert("split_count".to_string(), 0.0);
            }
            BossType::RufusReverse => {
                self.pattern_state
                    .insert("teleport_cooldown".to_string(), 0.0);
                self.pattern_state
                    .insert("reverse_zone_active".to_string(), 0.0);
            }
            BossType::MollyHashpass => {
                self.pattern_state.insert("heal_cooldown".to_string(), 0.0);
                self.pattern_state.insert("poison_clouds".to_string(), 0.0);
            }
            BossType::CaptainDavey => {
                self.pattern_state.insert("scan_cooldown".to_string(), 0.0);
                self.pattern_state
                    .insert("broadside_ready".to_string(), 0.0);
                self.pattern_state
                    .insert("minion_spawn_timer".to_string(), 0.0);
            }
            BossType::Deadbeef => {
                self.pattern_state.insert("food_type".to_string(), 0.0);
                self.pattern_state.insert("feast_timer".to_string(), 0.0);
            }
        }
    }

    pub async fn load_assets(&mut self) {
        let sprite_path = format!("assets/bosses/{}.png", self.boss_type.sprite_filename());

        if let Ok(sprite) = load_texture(&sprite_path).await {
            sprite.set_filter(FilterMode::Nearest);
            self.sprite = Some(sprite);
        } else {
            eprintln!("Warning: Could not load boss sprite: {}", sprite_path);
        }
    }

    pub fn update(&mut self, dt: f64, player_x: f32, player_y: f32) {
        if self.entry_anim {
            self.y += ENTRY_SPEED * dt as f32;
            if self.y >= 50.0 {
                self.y = 50.0;
                self.entry_anim = false;
            }
            return;
        }

        if self.invulnerable {
            self.invuln_timer -= dt;
            if self.invuln_timer <= 0.0 {
                self.invulnerable = false;
                self.color = WHITE;
            }
        }

        // Rufus Reverse's control-reversal zone ticks down here.
        self.tick_reverse_zone(dt);

        self.update_phase(dt);
        self.update_movement(dt);
        self.update_attacks(dt, player_x, player_y);
        self.update_projectiles(dt, player_x, player_y);
    }

    /// True while a reverse zone is active (player controls inverted).
    pub fn is_reverse_zone_active(&self) -> bool {
        *self
            .pattern_state
            .get("reverse_zone_active")
            .unwrap_or(&0.0)
            > 0.0
    }

    /// Ticks down the reverse-zone timer.
    fn tick_reverse_zone(&mut self, dt: f64) {
        if let Some(zone) = self.pattern_state.get_mut("reverse_zone_active") {
            if *zone > 0.0 {
                *zone -= dt as f32;
            }
        }
    }

    fn update_phase(&mut self, dt: f64) {
        let health_pct = self.health as f32 / self.max_health as f32;
        let new_phase = self.phase_for_health(health_pct);

        if new_phase != self.phase {
            self.phase = new_phase;
            self.phase_timer = 0.0;
            self.phase_flash = 0.5;
            // Phase transitions drop a powerup — consumed by the wave
            // director, which owns the powerup roster.
            self.pattern_state
                .insert("phase_powerup_drop".to_string(), 1.0);
            crate::audio::play_sfx("boss_laugh");
        }
        self.phase_timer += dt;
    }

    fn phase_for_health(&self, health_pct: f32) -> BossPhase {
        match self.active_phase_index(health_pct) {
            0 => BossPhase::Phase1,
            1 => BossPhase::Phase2,
            _ => BossPhase::Phase3,
        }
    }

    /// Phases are listed in descending `health_threshold` order (1.0, 0.5, 0.2);
    /// the active phase is the last one whose threshold is still met.
    fn active_phase_index(&self, health_pct: f32) -> usize {
        let mut idx = 0;
        for (i, phase) in self.phases.iter().enumerate() {
            if health_pct <= phase.health_threshold {
                idx = i;
            }
        }
        idx
    }

    fn update_movement(&mut self, dt: f64) {
        self.move_timer += dt;

        match &self.movement {
            MovementDef::Patrol {
                base_y,
                amp_y,
                freq_y,
                ..
            } => {
                self.x += self.speed_x * dt as f32;
                if self.x <= 50.0 || self.x >= screen_width() - self.width - 50.0 {
                    self.speed_x = -self.speed_x;
                }
                self.y = *base_y + (self.move_timer * *freq_y as f64).sin() as f32 * *amp_y;
            }
            MovementDef::CenteredSway {
                amp_x,
                freq_x,
                base_y,
                amp_y,
                freq_y,
            } => {
                self.x = screen_width() / 2.0 - self.width / 2.0
                    + (self.move_timer * *freq_x as f64).sin() as f32 * *amp_x;
                self.y = *base_y + (self.move_timer * *freq_y as f64).sin() as f32 * *amp_y;
            }
            MovementDef::Teleport { cooldown } => {
                if *self.pattern_state.get("teleport_cooldown").unwrap_or(&0.0) <= 0.0 {
                    if rand::gen_range(0, 100) < 2 {
                        self.x = rand::gen_range(100.0, screen_width() - self.width - 100.0);
                        self.y = rand::gen_range(50.0, 200.0);
                        self.glitch_fx = 0.3;
                        self.pattern_state
                            .insert("teleport_cooldown".to_string(), *cooldown);
                    }
                } else {
                    *self.pattern_state.get_mut("teleport_cooldown").unwrap() -= dt as f32;
                }
            }
        }
    }

    fn update_attacks(&mut self, dt: f64, player_x: f32, player_y: f32) {
        self.attack_timer += dt;

        let health_pct = self.health as f32 / self.max_health as f32;
        let idx = self.active_phase_index(health_pct);
        let Some(phase) = self.phases.get(idx) else {
            return;
        };

        if self.attack_timer >= phase.attack_interval {
            self.attack_timer = 0.0;
            let attacks = phase.attacks.clone();
            for attack in &attacks {
                self.execute_attack(attack, player_x, player_y);
            }
        }
    }

    fn execute_attack(&mut self, attack: &AttackDef, player_x: f32, player_y: f32) {
        let mut rng = ::rand::thread_rng();

        match attack {
            AttackDef::Burst {
                count,
                speed,
                bullet,
                chance,
            } => {
                if rng.gen::<f32>() < *chance {
                    self.ring_attack(*count, *speed * FPS_SCALE, bullet);
                }
            }
            AttackDef::Aimed {
                count,
                speed,
                spread,
                mirror,
                bullet,
                chance,
            } => {
                if rng.gen::<f32>() < *chance {
                    self.aimed_attack(
                        *count,
                        *speed * FPS_SCALE,
                        *spread,
                        *mirror,
                        bullet,
                        player_x,
                        player_y,
                    );
                }
            }
            AttackDef::Scatter {
                count,
                speed,
                spread,
                bullet,
                chance,
            } => {
                if rng.gen::<f32>() < *chance {
                    self.aimed_attack(
                        *count,
                        *speed * FPS_SCALE,
                        *spread,
                        false,
                        bullet,
                        player_x,
                        player_y,
                    );
                }
            }
            AttackDef::Ring {
                count,
                speed,
                bullet,
                chance,
            } => {
                if rng.gen::<f32>() < *chance {
                    self.ring_attack(*count, *speed * FPS_SCALE, bullet);
                }
            }
            AttackDef::Homing {
                count,
                bullet,
                chance,
            } => {
                if rng.gen::<f32>() < *chance {
                    self.homing_missiles(*count, bullet);
                }
            }
            AttackDef::Charge {
                speed,
                invuln,
                chance,
            } => {
                if rng.gen::<f32>() < *chance {
                    self.charge_attack(*speed * FPS_SCALE, *invuln, player_x);
                }
            }
            AttackDef::Heal { amount, chance } => {
                if rng.gen::<f32>() < *chance {
                    self.health = (self.health + *amount).min(self.max_health);
                    self.heal_fx = 0.8;
                }
            }
            AttackDef::HealBeam {
                amount,
                invuln,
                chance,
            } => {
                if rng.gen::<f32>() < *chance {
                    self.invulnerable = true;
                    self.invuln_timer = *invuln as f64;
                    self.color = Color::new(0.0, 1.0, 0.0, 0.7);
                    self.health = (self.health + *amount).min(self.max_health);
                    self.heal_fx = *invuln;
                }
            }
            AttackDef::PoisonCloud {
                count,
                bullet,
                chance,
            } => {
                if rng.gen::<f32>() < *chance {
                    self.poison_cloud(*count, bullet, player_x, player_y);
                }
            }
            AttackDef::ReviveMinions => self.revive_minions(),
            AttackDef::SpawnMinions => self.spawn_minions(),
            AttackDef::ReverseZone { duration, chance } => {
                if rng.gen::<f32>() < *chance {
                    self.pattern_state
                        .insert("reverse_zone_active".to_string(), *duration);
                }
            }
            AttackDef::Wall {
                rows,
                cols,
                speed,
                gap,
                bullet,
                chance,
            } => {
                if rng.gen::<f32>() < *chance {
                    self.wall_attack(*rows, *cols, *speed * FPS_SCALE, *gap, bullet);
                }
            }
            AttackDef::Beam { bullet, chance } => {
                if rng.gen::<f32>() < *chance {
                    // Telegraph first: a full-height column at the player's
                    // position flashes for BEAM_TELEGRAPH_TIME, THEN fires —
                    // the beam is dodgeable instead of a guaranteed hit.
                    self.beam_telegraph = Some(BeamTelegraph {
                        remaining: BEAM_TELEGRAPH_TIME,
                        x: player_x,
                        bullet: bullet.clone(),
                    });
                }
            }
            AttackDef::ThrowFood { bullet, chance } => {
                if rng.gen::<f32>() < *chance {
                    self.throw_food(bullet, player_x, player_y);
                }
            }
            AttackDef::Feast { count, chance } => {
                if rng.gen::<f32>() < *chance {
                    self.feast_mode(*count);
                }
            }
        }
    }

    fn make_projectile(
        &self,
        x: f32,
        y: f32,
        vel_x: f32,
        vel_y: f32,
        bullet: &BulletDef,
    ) -> BossProjectile {
        BossProjectile {
            x,
            y,
            vel_x,
            vel_y,
            size: bullet.size,
            color: content::parse_color(&bullet.color),
            lifetime: bullet.lifetime,
            pattern: ProjectilePattern::from_key(&bullet.pattern),
            food: false,
        }
    }

    fn ring_attack(&mut self, count: u32, speed: f32, bullet: &BulletDef) {
        let center_x = self.x + self.width / 2.0;
        let center_y = self.y + self.height;
        let angle_step = std::f32::consts::TAU / count as f32;

        for i in 0..count {
            let angle = i as f32 * angle_step;
            let p = self.make_projectile(
                center_x,
                center_y,
                angle.cos() * speed,
                angle.sin() * speed,
                bullet,
            );
            self.projectiles.push(p);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn aimed_attack(
        &mut self,
        count: u32,
        speed: f32,
        spread: f32,
        mirror: bool,
        bullet: &BulletDef,
        player_x: f32,
        player_y: f32,
    ) {
        let center_x = self.x + self.width / 2.0;
        let center_y = self.y + self.height;
        let dx = player_x - center_x;
        let dy = player_y - center_y;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist <= 0.0 {
            return;
        }
        let (nx, ny) = (dx / dist, dy / dist);

        if mirror {
            self.mirror_fx = 0.4;
            for i in 0..count {
                let dir = if i % 2 == 0 { 1.0 } else { -1.0 };
                let p =
                    self.make_projectile(center_x, center_y, nx * speed * dir, ny * speed, bullet);
                self.projectiles.push(p);
            }
        } else {
            let base_angle = ny.atan2(nx);
            for i in 0..count {
                let t = if count > 1 {
                    i as f32 / (count - 1) as f32
                } else {
                    0.5
                };
                let angle = base_angle + (t - 0.5) * spread;
                let p = self.make_projectile(
                    center_x,
                    center_y,
                    angle.cos() * speed,
                    angle.sin() * speed,
                    bullet,
                );
                self.projectiles.push(p);
            }
        }
    }

    fn homing_missiles(&mut self, count: u32, bullet: &BulletDef) {
        let center_x = self.x + self.width / 2.0;
        let center_y = self.y + self.height;

        for _ in 0..count {
            let p = self.make_projectile(center_x, center_y, 0.0, 0.0, bullet);
            self.projectiles.push(p);
        }
    }

    fn charge_attack(&mut self, speed: f32, invuln: f32, player_x: f32) {
        let dir = if player_x > self.x + self.width / 2.0 {
            1.0
        } else {
            -1.0
        };
        self.speed_x = dir * speed;
        self.invulnerable = true;
        self.invuln_timer = invuln as f64;
        self.color = RED;
        self.inflate_fx = invuln;
    }

    fn poison_cloud(&mut self, count: u32, bullet: &BulletDef, player_x: f32, player_y: f32) {
        for _ in 0..count {
            let px = player_x + rand::gen_range(-50.0, 50.0);
            let py = player_y + rand::gen_range(-50.0, 50.0);
            let p = self.make_projectile(px, py, 0.0, 0.0, bullet);
            self.projectiles.push(p);
        }
    }

    /// One-shot signals consumed by the wave director, which owns the pirate
    /// minion roster.
    fn revive_minions(&mut self) {
        self.pattern_state.insert("revive_signal".to_string(), 1.0);
    }

    fn spawn_minions(&mut self) {
        self.pattern_state.insert("spawn_minions".to_string(), 1.0);
    }

    /// True when a phase transition wants a powerup drop; one-shot.
    pub fn check_phase_powerup_drop(&self) -> bool {
        *self.pattern_state.get("phase_powerup_drop").unwrap_or(&0.0) > 0.0
    }

    pub fn clear_phase_powerup_drop(&mut self) {
        self.pattern_state
            .insert("phase_powerup_drop".to_string(), 0.0);
    }

    pub fn check_revive_signal(&self) -> bool {
        *self.pattern_state.get("revive_signal").unwrap_or(&0.0) > 0.0
    }

    pub fn clear_revive_signal(&mut self) {
        self.pattern_state.insert("revive_signal".to_string(), 0.0);
    }

    pub fn check_spawn_minions(&self) -> bool {
        *self.pattern_state.get("spawn_minions").unwrap_or(&0.0) > 0.0
    }

    pub fn clear_spawn_minions(&mut self) {
        self.pattern_state.insert("spawn_minions".to_string(), 0.0);
    }

    fn scanning_beam(&mut self, bullet: &BulletDef, beam_x: f32, screen_h: f32) {
        for y in (0..screen_h as i32).step_by(20) {
            let p = self.make_projectile(beam_x, y as f32, 0.0, 0.0, bullet);
            self.projectiles.push(p);
        }
    }

    fn wall_attack(&mut self, rows: u32, cols: u32, speed: f32, gap: u32, bullet: &BulletDef) {
        let start_y = 100.0;
        let row_spacing = (screen_height() - 200.0) / rows as f32;

        for row in 0..rows {
            let gap_start = rand::gen_range(2, cols.saturating_sub(2).max(3));
            for col in 0..cols {
                if col < gap_start || col >= gap_start + gap {
                    let p = self.make_projectile(
                        col as f32 * screen_width() / cols as f32,
                        start_y + row as f32 * row_spacing,
                        0.0,
                        speed,
                        bullet,
                    );
                    self.projectiles.push(p);
                }
            }
        }
    }

    fn throw_food(&mut self, bullet: &BulletDef, player_x: f32, player_y: f32) {
        let center_x = self.x + self.width / 2.0;
        let center_y = self.y + self.height;
        let food_colors = ["brown", "orange", "yellow", "pink"];
        let color = food_colors[rand::gen_range(0, food_colors.len())];

        let dx = player_x - center_x;
        let dy = player_y - center_y;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist <= 0.0 {
            return;
        }

        let mut p = self.make_projectile(
            center_x,
            center_y,
            (dx / dist) * 180.0,
            (dy / dist) * 180.0 - 120.0,
            bullet,
        );
        p.color = content::parse_color(color);
        p.food = true;
        self.projectiles.push(p);
    }

    fn feast_mode(&mut self, count: u32) {
        for _ in 0..count {
            self.projectiles.push(BossProjectile {
                x: rand::gen_range(0.0, screen_width()),
                y: -50.0,
                vel_x: rand::gen_range(-60.0, 60.0),
                vel_y: rand::gen_range(180.0, 360.0),
                size: rand::gen_range(15.0, 25.0),
                color: Color::new(
                    rand::gen_range(0.5, 1.0),
                    rand::gen_range(0.3, 0.6),
                    rand::gen_range(0.1, 0.3),
                    1.0,
                ),
                lifetime: 10.0,
                pattern: ProjectilePattern::Straight,
                food: true,
            });
        }
    }

    fn update_projectiles(&mut self, dt: f64, player_x: f32, player_y: f32) {
        let dt_f32 = dt as f32;

        self.projectiles.retain_mut(|p| {
            p.lifetime -= dt;
            if p.lifetime <= 0.0 {
                return false;
            }

            match p.pattern {
                ProjectilePattern::Straight => {
                    p.x += p.vel_x * dt_f32;
                    p.y += p.vel_y * dt_f32;
                }
                ProjectilePattern::Spiral => {
                    p.vel_x = (p.vel_x + p.vel_y * 0.1).sin() * 240.0;
                    p.vel_y = (p.vel_y - p.vel_x * 0.1).cos() * 240.0;
                    p.x += p.vel_x * dt_f32;
                    p.y += p.vel_y * dt_f32;
                }
                ProjectilePattern::Homing => {
                    let dx = player_x - p.x;
                    let dy = player_y - p.y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist > 0.0 {
                        p.vel_x += (dx / dist) * 18.0 * dt_f32;
                        p.vel_y += (dy / dist) * 18.0 * dt_f32;
                        let speed = (p.vel_x * p.vel_x + p.vel_y * p.vel_y).sqrt();
                        if speed > 300.0 {
                            p.vel_x = p.vel_x / speed * 300.0;
                            p.vel_y = p.vel_y / speed * 300.0;
                        }
                    }
                    p.x += p.vel_x * dt_f32;
                    p.y += p.vel_y * dt_f32;
                }
                ProjectilePattern::Bounce => {
                    p.vel_y += 300.0 * dt_f32;
                    p.x += p.vel_x * dt_f32;
                    p.y += p.vel_y * dt_f32;
                    if p.y > screen_height() - 50.0 {
                        p.y = screen_height() - 50.0;
                        p.vel_y = -p.vel_y * 0.7;
                    }
                }
                ProjectilePattern::Wave => {
                    p.x += p.vel_x * dt_f32;
                    p.y += p.vel_y * dt_f32 + (p.x * 0.05).sin() * 120.0 * dt_f32;
                }
            }

            p.x > -50.0
                && p.x < screen_width() + 50.0
                && p.y > -50.0
                && p.y < screen_height() + 50.0
        });
    }

    pub fn draw(&self) {
        if self.is_dead {
            self.draw_death_explosion();
            return;
        }

        self.draw_beam_telegraph();
        self.draw_body();
        self.draw_fx();
        self.draw_projectiles();
    }

    /// The post-defeat explosion: expanding shockwave rings plus radial
    /// debris, ending in nothing (the boss is gone once the timer runs out).
    fn draw_death_explosion(&self) {
        let t = 1.0 - self.death_timer / DEATH_FX_TIME; // 0 -> 1
        if t >= 1.0 {
            return;
        }
        let cx = self.x + self.width / 2.0;
        let cy = self.y + self.height / 2.0;
        let fade = 1.0 - t;

        // Initial flash.
        if t < 0.2 {
            draw_circle(
                cx,
                cy,
                self.width * (0.5 + t * 2.0),
                Color::new(1.0, 1.0, 1.0, 0.6 * (1.0 - t / 0.2)),
            );
        }

        // Shockwave rings.
        for ring in 0..2 {
            let r = (t * 180.0) + ring as f32 * 40.0;
            draw_circle_lines(cx, cy, r, 4.0, Color::new(1.0, 0.6, 0.1, fade));
        }

        // Radial debris, deterministic angles (golden-angle spacing).
        for i in 0..12 {
            let angle = i as f32 * 2.399963;
            let dist = 20.0 + t * 160.0 * (if i % 2 == 0 { 1.0 } else { 0.6 });
            let px = cx + angle.cos() * dist;
            let py = cy + angle.sin() * dist;
            draw_rectangle(
                px - 3.0,
                py - 3.0,
                6.0,
                6.0,
                Color::new(1.0, 0.8, 0.2, fade),
            );
        }
    }

    /// Davey's beam: a flashing red column at the locked-in position while
    /// the telegraph counts down.
    fn draw_beam_telegraph(&self) {
        if let Some(x) = self.beam_telegraph_position() {
            let pulse = (get_time() * 20.0).sin() * 0.5 + 0.5;
            draw_rectangle(
                x - 10.0,
                0.0,
                20.0,
                screen_height(),
                Color::new(1.0, 0.1, 0.1, 0.10 + 0.10 * pulse as f32),
            );
            draw_rectangle(
                x - 2.0,
                0.0,
                4.0,
                screen_height(),
                Color::new(1.0, 0.3, 0.3, 0.6),
            );
        }
    }

    fn draw_body(&self) {
        let scale = 1.0 + 0.3 * self.inflate_fx; // Blowfish charge puff-up
        let w = self.width * scale;
        let h = self.height * scale;
        let x = self.x - (w - self.width) / 2.0; // keep centered
        let y = self.y - (h - self.height) / 2.0;

        if let Some(sprite) = &self.sprite {
            draw_texture_ex(
                sprite,
                x,
                y,
                self.color,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(w, h)),
                    ..Default::default()
                },
            );

            // Twofish mirror ghost: a flipped, transparent twin briefly
            // appears during mirrored aimed attacks.
            if self.mirror_fx > 0.0 {
                let offset = 30.0 + self.mirror_fx * 40.0;
                draw_texture_ex(
                    sprite,
                    x + offset,
                    y,
                    Color::new(1.0, 1.0, 1.0, self.mirror_fx),
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(w, h)),
                        flip_x: true,
                        ..Default::default()
                    },
                );
                draw_texture_ex(
                    sprite,
                    x - offset,
                    y,
                    Color::new(1.0, 1.0, 1.0, self.mirror_fx),
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(w, h)),
                        flip_x: true,
                        ..Default::default()
                    },
                );
            }

            // Rufus teleport glitch: chromatic-aberration ghosts that decay.
            if self.glitch_fx > 0.0 {
                let jitter = 4.0 + self.glitch_fx * 16.0;
                for (dx, tint) in [
                    (jitter, Color::new(1.0, 0.2, 0.2, 0.5)),
                    (-jitter, Color::new(0.2, 0.4, 1.0, 0.5)),
                ] {
                    draw_texture_ex(
                        sprite,
                        x + dx,
                        y,
                        tint,
                        DrawTextureParams {
                            dest_size: Some(Vec2::new(w, h)),
                            ..Default::default()
                        },
                    );
                }
            }
        } else {
            draw_rectangle(x, y, w, h, self.color);
        }
    }

    /// Per-boss effect overlays: phase flash + shockwave, Molly's heal ring.
    fn draw_fx(&self) {
        let cx = self.x + self.width / 2.0;
        let cy = self.y + self.height / 2.0;

        // Phase transition: white blink over the body plus an expanding ring.
        if self.phase_flash > 0.0 {
            let t = self.phase_flash / 0.5; // 1 -> 0
            draw_rectangle(
                self.x,
                self.y,
                self.width,
                self.height,
                Color::new(1.0, 1.0, 1.0, 0.35 * t),
            );
            draw_circle_lines(
                cx,
                cy,
                (1.0 - t) * 140.0 + 20.0,
                3.0,
                Color::new(1.0, 0.9, 0.3, t),
            );
        }

        // Molly's heal: pulsing green rings while regenerating.
        if self.heal_fx > 0.0 {
            let pulse = (get_time() * 8.0).sin() * 0.5 + 0.5;
            let alpha = self.heal_fx.min(1.0);
            for ring in 0..2 {
                let r = self.width * (0.7 + ring as f32 * 0.25) + pulse as f32 * 8.0;
                draw_circle_lines(
                    cx,
                    cy,
                    r,
                    3.0,
                    Color::new(0.2, 1.0, 0.3, alpha * (0.5 + 0.3 * pulse as f32)),
                );
            }
            // Small "+" above the boss to make "healing" unmistakable.
            draw_text(
                "+",
                cx - 4.0,
                self.y - 10.0,
                30.0,
                Color::new(0.2, 1.0, 0.3, alpha),
            );
        }
    }

    /// Projectile rendering keyed on pattern so each threat type reads at a
    /// glance: homing pulses, bouncing stretches vertically, spirals trail,
    /// food tumbles.
    fn draw_projectiles(&self) {
        for p in &self.projectiles {
            let r = p.size / 2.0;
            match p.pattern {
                ProjectilePattern::Homing => {
                    let pulse = ((get_time() * 10.0).sin() * 0.5 + 0.5) as f32;
                    draw_circle(p.x, p.y, r, p.color);
                    draw_circle_lines(p.x, p.y, r + 3.0 + pulse * 3.0, 2.0, WHITE);
                }
                ProjectilePattern::Bounce => {
                    let stretch = (p.vel_y / 300.0).clamp(-1.0, 1.0).abs();
                    draw_ellipse(p.x, p.y, r, r * (1.0 + 0.4 * stretch), 0.0, p.color);
                    draw_circle_lines(p.x, p.y, r, 2.0, WHITE);
                }
                ProjectilePattern::Spiral => {
                    // Two fading trail circles behind the motion.
                    for k in 1..=2 {
                        let t = k as f32 * 0.04;
                        draw_circle(
                            p.x - p.vel_x * t,
                            p.y - p.vel_y * t,
                            r * (1.0 - 0.25 * k as f32),
                            Color::new(p.color.r, p.color.g, p.color.b, 0.3),
                        );
                    }
                    draw_circle(p.x, p.y, r, p.color);
                    draw_circle_lines(p.x, p.y, r, 2.0, WHITE);
                }
                _ if p.food => {
                    // Deadbeef's food fight: tumbling squares.
                    draw_rectangle_ex(
                        p.x - r,
                        p.y - r,
                        p.size,
                        p.size,
                        DrawRectangleParams {
                            rotation: (p.x + p.y) * 0.02,
                            color: p.color,
                            ..Default::default()
                        },
                    );
                }
                _ => {
                    draw_circle(p.x, p.y, r, p.color);
                    draw_circle_lines(p.x, p.y, r, 2.0, WHITE);
                }
            }
        }
    }

    /// Ticks all visual/effect timers. Called by the wave director every
    /// frame the boss exists (including after defeat, so the explosion
    /// animates) — boss.update() only runs during the fight itself.
    pub fn update_fx(&mut self, dt: f64, screen_h: f32) {
        let dt = dt as f32;
        self.phase_flash = (self.phase_flash - dt).max(0.0);
        self.inflate_fx = (self.inflate_fx - dt).max(0.0);
        self.mirror_fx = (self.mirror_fx - dt).max(0.0);
        self.glitch_fx = (self.glitch_fx - dt).max(0.0);
        self.heal_fx = (self.heal_fx - dt).max(0.0);
        if self.is_dead && self.death_timer > 0.0 {
            self.death_timer = (self.death_timer - dt).max(0.0);
        }

        // Beam telegraph countdown -> fire.
        if let Some(mut telegraph) = self.beam_telegraph.take() {
            telegraph.remaining -= dt;
            if telegraph.remaining <= 0.0 {
                self.scanning_beam(&telegraph.bullet, telegraph.x, screen_h);
            } else {
                self.beam_telegraph = Some(telegraph);
            }
        }
    }

    /// True while a beam telegraph column should be drawn, and its x.
    pub fn beam_telegraph_position(&self) -> Option<f32> {
        self.beam_telegraph.as_ref().map(|t| t.x)
    }

    /// Returns true if this damage killed the boss. The boss briefly becomes
    /// invulnerable after each hit, which also rate-limits piercing bullets.
    pub fn take_damage(&mut self, damage: i32) -> bool {
        if !self.can_take_damage() {
            return false;
        }

        self.health -= damage;
        self.invulnerable = true;
        self.invuln_timer = HIT_INVULN;
        self.color = RED;

        if self.health <= 0 {
            self.health = 0;
            self.is_dead = true;
            self.death_timer = DEATH_FX_TIME;
            crate::audio::play_sfx("boss_defeat");
            return true;
        }
        false
    }

    pub fn get_rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.width, self.height)
    }

    /// True when a hit would register (not invulnerable, not entering, and
    /// still alive — bullets pass through a dying boss without spending
    /// pierce charges).
    pub fn can_take_damage(&self) -> bool {
        !self.invulnerable && !self.entry_anim && !self.is_dead
    }

    pub fn get_projectiles(&self) -> &[BossProjectile] {
        &self.projectiles
    }
}

/// Builds a boss via struct literal for tests — `Boss::new` reads
/// `screen_width()` and needs a GPU context.
#[cfg(test)]
pub(crate) fn make_test_boss(health: i32, max_health: i32) -> Boss {
    let mut state = std::collections::HashMap::new();
    state.insert("charge_ready".to_string(), 0.0);
    Boss {
        boss_type: BossType::Blowfish,
        x: 0.0,
        y: 0.0,
        width: 80.0,
        height: 80.0,
        health,
        max_health,
        phase: BossPhase::Phase1,
        phase_timer: 0.0,
        attack_timer: 0.0,
        move_timer: 0.0,
        speed_x: 120.0,
        is_dead: false,
        invulnerable: false,
        invuln_timer: 0.0,
        projectiles: Vec::new(),
        pattern_state: state,
        movement: MovementDef::Patrol {
            speed_x: 2.0,
            base_y: 50.0,
            amp_y: 30.0,
            freq_y: 0.5,
        },
        phases: vec![
            BossPhaseDef {
                health_threshold: 1.0,
                attack_interval: 2.5,
                attacks: vec![],
            },
            BossPhaseDef {
                health_threshold: 0.5,
                attack_interval: 1.8,
                attacks: vec![],
            },
            BossPhaseDef {
                health_threshold: 0.2,
                attack_interval: 1.2,
                attacks: vec![],
            },
        ],
        sprite: None,
        color: WHITE,
        entry_anim: false,
        phase_flash: 0.0,
        inflate_fx: 0.0,
        mirror_fx: 0.0,
        glitch_fx: 0.0,
        heal_fx: 0.0,
        death_timer: 0.0,
        beam_telegraph: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a boss via struct literal so tests don't need a GPU context
    /// (`Boss::new` reads `screen_width()`).
    fn make_boss(health: i32, max_health: i32) -> Boss {
        make_test_boss(health, max_health)
    }

    #[test]
    fn test_take_damage_reduces_health() {
        let mut boss = make_boss(150, 150);
        boss.take_damage(10);
        assert_eq!(boss.health, 140);
        assert!(!boss.is_dead);
    }

    #[test]
    fn test_take_damage_kills_boss() {
        let mut boss = make_boss(150, 150);
        let dead = boss.take_damage(150);
        assert!(dead);
        assert_eq!(boss.health, 0);
        assert!(boss.is_dead);
    }

    #[test]
    fn test_take_damage_invulnerable() {
        let mut boss = make_boss(150, 150);
        boss.take_damage(10);
        let result = boss.take_damage(10);
        assert!(!result);
        assert_eq!(boss.health, 140);
    }

    #[test]
    fn test_take_damage_entry_anim() {
        let mut boss = make_boss(150, 150);
        boss.entry_anim = true;
        let result = boss.take_damage(10);
        assert!(!result);
        assert_eq!(boss.health, 150);
    }

    #[test]
    fn test_invulnerable_sets_after_damage() {
        let mut boss = make_boss(150, 150);
        boss.take_damage(10);
        assert!(boss.invulnerable);
        assert_eq!(boss.invuln_timer, HIT_INVULN);
    }

    #[test]
    fn test_can_take_damage() {
        let mut boss = make_boss(150, 150);
        assert!(boss.can_take_damage());
        boss.invulnerable = true;
        assert!(!boss.can_take_damage());
        boss.invulnerable = false;
        boss.entry_anim = true;
        assert!(!boss.can_take_damage());
    }

    #[test]
    fn test_phase_progression() {
        let mut boss = make_boss(150, 150);
        assert_eq!(boss.phase, BossPhase::Phase1);

        boss.health = 60; // 40% -> phase 2
        boss.update_phase(0.016);
        assert_eq!(boss.phase, BossPhase::Phase2);

        boss.health = 15; // 10% -> phase 3
        boss.update_phase(0.016);
        assert_eq!(boss.phase, BossPhase::Phase3);
    }

    #[test]
    fn test_get_rect() {
        let boss = make_boss(150, 150);
        let (x, y, w, h) = boss.get_rect();
        assert_eq!(w, 80.0);
        assert_eq!(h, 80.0);
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
    }

    /// Reverse-zone signal: set by the attack, ticks down over time.
    #[test]
    fn test_reverse_zone_lifecycle() {
        let mut boss = make_boss(150, 150);
        assert!(!boss.is_reverse_zone_active());
        boss.pattern_state
            .insert("reverse_zone_active".to_string(), 5.0);
        assert!(boss.is_reverse_zone_active());
        boss.tick_reverse_zone(1.0);
        assert!(boss.is_reverse_zone_active());
        boss.tick_reverse_zone(4.0);
        assert!(!boss.is_reverse_zone_active());
    }

    /// Minion signals are one-shot: set by attacks, cleared by the consumer.
    #[test]
    fn test_minion_signals() {
        let mut boss = make_boss(150, 150);
        assert!(!boss.check_spawn_minions());
        boss.pattern_state.insert("spawn_minions".to_string(), 1.0);
        assert!(boss.check_spawn_minions());
        boss.clear_spawn_minions();
        assert!(!boss.check_spawn_minions());

        assert!(!boss.check_revive_signal());
        boss.pattern_state.insert("revive_signal".to_string(), 1.0);
        assert!(boss.check_revive_signal());
        boss.clear_revive_signal();
        assert!(!boss.check_revive_signal());
    }

    /// Phase transitions set the powerup-drop signal exactly once per
    /// transition.
    #[test]
    fn test_phase_change_drops_powerup_signal() {
        let mut boss = make_boss(150, 150);
        assert!(!boss.check_phase_powerup_drop());

        // 40% health -> phase 2 transition.
        boss.health = 60;
        boss.update_phase(0.016);
        assert!(boss.check_phase_powerup_drop());
        boss.clear_phase_powerup_drop();
        assert!(!boss.check_phase_powerup_drop());

        // No transition on the next tick: no new signal.
        boss.update_phase(0.016);
        assert!(!boss.check_phase_powerup_drop());

        // 10% health -> phase 3 transition: signal again.
        boss.health = 15;
        boss.update_phase(0.016);
        assert!(boss.check_phase_powerup_drop());
    }

    /// Phase transitions also arm the visual flash.
    #[test]
    fn test_phase_change_sets_flash() {
        let mut boss = make_boss(150, 150);
        boss.health = 60;
        boss.update_phase(0.016);
        assert!(boss.phase_flash > 0.0);
        boss.update_fx(1.0, 800.0);
        assert_eq!(boss.phase_flash, 0.0);
    }

    /// Davey's beam telegraphs before firing: no instant projectiles, the
    /// column position is locked in, and the beam spawns only when the
    /// countdown expires.
    #[test]
    fn test_beam_telegraph_delays_fire() {
        let mut boss = make_boss(500, 500);
        let bullet = crate::content::BulletDef {
            size: 12.0,
            color: "red".to_string(),
            pattern: "straight".to_string(),
            lifetime: 3.0,
            damage: 1,
        };

        boss.execute_attack(
            &crate::content::AttackDef::Beam {
                bullet: bullet.clone(),
                chance: 1.0,
            },
            400.0,
            600.0,
        );
        assert!(boss.projectiles.is_empty(), "beam must not fire instantly");
        assert_eq!(boss.beam_telegraph_position(), Some(400.0));

        // Halfway through the telegraph: still no beam.
        boss.update_fx(0.3, 800.0);
        assert!(boss.projectiles.is_empty());

        // Countdown expires: the column fires at the locked position.
        boss.update_fx(0.5, 800.0);
        assert!(!boss.projectiles.is_empty());
        assert_eq!(boss.beam_telegraph_position(), None);
        for p in &boss.projectiles {
            assert_eq!(p.x, 400.0);
        }
    }

    /// Killing the boss arms the death explosion countdown, and it decays.
    #[test]
    fn test_death_timer() {
        let mut boss = make_boss(10, 10);
        assert!(boss.take_damage(10));
        assert!(boss.is_dead);
        assert!(boss.death_timer > 0.0);
        // A dying boss cannot be damaged further.
        assert!(!boss.can_take_damage());

        boss.update_fx(0.5, 800.0);
        assert!(boss.death_timer > 0.0);
        boss.update_fx(0.6, 800.0);
        assert_eq!(boss.death_timer, 0.0);
    }
}
