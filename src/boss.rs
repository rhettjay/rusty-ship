use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ::rand::Rng;
use crate::content::{self, AttackDef, BossPhaseDef, BulletDef, MovementDef};

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
        content::boss(self.key()).unwrap_or_else(|| panic!("missing boss definition: {}", self.key()))
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

    pub fn max_health(&self) -> i32 {
        self.def().max_health
    }

    pub fn portrait_filename(&self) -> &'static str {
        match self {
            BossType::Blowfish => "blowfish",
            BossType::Twofish => "twofish",
            BossType::RufusReverse => "rufus_reverse",
            BossType::MollyHashpass => "molly_hashpass",
            BossType::CaptainDavey => "captain_davey_portscan",
            BossType::Deadbeef => "deadbeef",
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

    pub fn dialogue_intro_id(&self) -> String {
        format!("boss_intro_{}", self.name().to_lowercase().replace(" ", "_"))
    }

    pub fn dialogue_defeat_id(&self) -> String {
        format!("boss_defeat_{}", self.name().to_lowercase().replace(" ", "_"))
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
    pub damage: i32,
    pub size: f32,
    pub color: Color,
    pub lifetime: f64,
    pub pattern: ProjectilePattern,
}

impl BossProjectile {
    pub fn get_rect(&self) -> (f32, f32, f32, f32) {
        (self.x - self.size / 2.0, self.y - self.size / 2.0, self.size, self.size)
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
    pub portrait: Option<Texture2D>,
    pub color: Color,
    pub entry_anim: bool,
    pub entry_timer: f64,
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
                MovementDef::Patrol { speed_x, .. } => *speed_x,
                _ => 2.0,
            },
            is_dead: false,
            invulnerable: false,
            invuln_timer: 0.0,
            projectiles: Vec::new(),
            pattern_state: HashMap::new(),
            movement: def.movement.clone(),
            phases: def.phases.clone(),
            sprite: None,
            portrait: None,
            color: WHITE,
            entry_anim: true,
            entry_timer: 0.0,
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
                self.pattern_state.insert("teleport_cooldown".to_string(), 0.0);
                self.pattern_state.insert("reverse_zone_active".to_string(), 0.0);
            }
            BossType::MollyHashpass => {
                self.pattern_state.insert("heal_cooldown".to_string(), 0.0);
                self.pattern_state.insert("poison_clouds".to_string(), 0.0);
            }
            BossType::CaptainDavey => {
                self.pattern_state.insert("scan_cooldown".to_string(), 0.0);
                self.pattern_state.insert("broadside_ready".to_string(), 0.0);
                self.pattern_state.insert("minion_spawn_timer".to_string(), 0.0);
            }
            BossType::Deadbeef => {
                self.pattern_state.insert("food_type".to_string(), 0.0);
                self.pattern_state.insert("feast_timer".to_string(), 0.0);
            }
        }
    }

    pub async fn load_assets(&mut self) {
        let sprite_path = format!("assets/bosses/{}.png", self.boss_type.sprite_filename());
        let portrait_path = format!("assets/portraits/{}.png", self.boss_type.portrait_filename());

        if let Ok(sprite) = load_texture(&sprite_path).await {
            sprite.set_filter(FilterMode::Nearest);
            self.sprite = Some(sprite);
        } else {
            eprintln!("Warning: Could not load boss sprite: {}", sprite_path);
        }

        if let Ok(portrait) = load_texture(&portrait_path).await {
            portrait.set_filter(FilterMode::Nearest);
            self.portrait = Some(portrait);
        } else {
            eprintln!("Warning: Could not load boss portrait: {}", portrait_path);
        }
    }

    pub fn update(&mut self, dt: f64, player_x: f32, player_y: f32) {
        if self.entry_anim {
            self.entry_timer += dt;
            self.y += 2.0;
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

        self.update_phase();
        self.update_movement(dt);
        self.update_attacks(dt, player_x, player_y);
        self.update_projectiles(dt);
    }

    fn update_phase(&mut self) {
        let health_pct = self.health as f32 / self.max_health as f32;
        let new_phase = self.phase_for_health(health_pct);

        if new_phase != self.phase {
            self.phase = new_phase;
            self.phase_timer = 0.0;
            crate::audio::play_sfx("boss_laugh");
        }
        self.phase_timer += 1.0 / 60.0;
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
            MovementDef::Patrol { base_y, amp_y, freq_y, .. } => {
                self.x += self.speed_x * dt as f32;
                if self.x <= 50.0 || self.x >= screen_width() - self.width - 50.0 {
                    self.speed_x = -self.speed_x;
                }
                self.y = *base_y + (self.move_timer * *freq_y as f64).sin() as f32 * *amp_y;
            }
            MovementDef::CenteredSway { amp_x, freq_x, base_y, amp_y, freq_y } => {
                self.x = screen_width() / 2.0 - self.width / 2.0
                    + (self.move_timer * *freq_x as f64).sin() as f32 * *amp_x;
                self.y = *base_y + (self.move_timer * *freq_y as f64).sin() as f32 * *amp_y;
            }
            MovementDef::Teleport { cooldown } => {
                if *self.pattern_state.get("teleport_cooldown").unwrap_or(&0.0) <= 0.0 {
                    if rand::gen_range(0, 100) < 2 {
                        self.x = rand::gen_range(100.0, screen_width() - self.width - 100.0);
                        self.y = rand::gen_range(50.0, 200.0);
                        self.pattern_state.insert("teleport_cooldown".to_string(), *cooldown);
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
            AttackDef::Burst { count, speed, bullet, chance } => {
                if rng.gen::<f32>() < *chance {
                    self.ring_attack(*count, *speed, bullet);
                }
            }
            AttackDef::Aimed { count, speed, spread, mirror, bullet, chance } => {
                if rng.gen::<f32>() < *chance {
                    self.aimed_attack(*count, *speed, *spread, *mirror, bullet, player_x, player_y);
                }
            }
            AttackDef::Scatter { count, speed, spread, bullet, chance } => {
                if rng.gen::<f32>() < *chance {
                    self.aimed_attack(*count, *speed, *spread, false, bullet, player_x, player_y);
                }
            }
            AttackDef::Ring { count, speed, bullet, chance } => {
                if rng.gen::<f32>() < *chance {
                    self.ring_attack(*count, *speed, bullet);
                }
            }
            AttackDef::Homing { count, bullet, chance } => {
                if rng.gen::<f32>() < *chance {
                    self.homing_missiles(*count, bullet);
                }
            }
            AttackDef::Charge { speed, invuln, chance } => {
                if rng.gen::<f32>() < *chance {
                    self.charge_attack(*speed, *invuln, player_x);
                }
            }
            AttackDef::Heal { amount, chance } => {
                if rng.gen::<f32>() < *chance {
                    self.health = (self.health + *amount).min(self.max_health);
                }
            }
            AttackDef::HealBeam { amount, invuln, chance } => {
                if rng.gen::<f32>() < *chance {
                    self.invulnerable = true;
                    self.invuln_timer = *invuln as f64;
                    self.color = Color::new(0.0, 1.0, 0.0, 0.7);
                    self.health = (self.health + *amount).min(self.max_health);
                }
            }
            AttackDef::PoisonCloud { count, bullet, chance } => {
                if rng.gen::<f32>() < *chance {
                    self.poison_cloud(*count, bullet, player_x, player_y);
                }
            }
            AttackDef::ReviveMinions => self.revive_minions(),
            AttackDef::SpawnMinions => self.spawn_minions(),
            AttackDef::ReverseZone { duration, chance } => {
                if rng.gen::<f32>() < *chance {
                    self.pattern_state.insert("reverse_zone_active".to_string(), *duration);
                }
            }
            AttackDef::Wall { rows, cols, speed, gap, bullet, chance } => {
                if rng.gen::<f32>() < *chance {
                    self.wall_attack(*rows, *cols, *speed, *gap, bullet);
                }
            }
            AttackDef::Beam { bullet, chance } => {
                if rng.gen::<f32>() < *chance {
                    self.scanning_beam(bullet, player_x);
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

    fn make_projectile(&self, x: f32, y: f32, vel_x: f32, vel_y: f32, bullet: &BulletDef) -> BossProjectile {
        BossProjectile {
            x,
            y,
            vel_x,
            vel_y,
            damage: bullet.damage,
            size: bullet.size,
            color: content::parse_color(&bullet.color),
            lifetime: bullet.lifetime,
            pattern: ProjectilePattern::from_key(&bullet.pattern),
        }
    }

    fn ring_attack(&mut self, count: u32, speed: f32, bullet: &BulletDef) {
        let center_x = self.x + self.width / 2.0;
        let center_y = self.y + self.height;
        let angle_step = std::f32::consts::TAU / count as f32;

        for i in 0..count {
            let angle = i as f32 * angle_step;
            let p = self.make_projectile(center_x, center_y, angle.cos() * speed, angle.sin() * speed, bullet);
            self.projectiles.push(p);
        }
    }

    fn aimed_attack(&mut self, count: u32, speed: f32, spread: f32, mirror: bool, bullet: &BulletDef, player_x: f32, player_y: f32) {
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
            for i in 0..count {
                let dir = if i % 2 == 0 { 1.0 } else { -1.0 };
                let p = self.make_projectile(center_x, center_y, nx * speed * dir, ny * speed, bullet);
                self.projectiles.push(p);
            }
        } else {
            let base_angle = ny.atan2(nx);
            for i in 0..count {
                let t = if count > 1 { i as f32 / (count - 1) as f32 } else { 0.5 };
                let angle = base_angle + (t - 0.5) * spread;
                let p = self.make_projectile(center_x, center_y, angle.cos() * speed, angle.sin() * speed, bullet);
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
        let dir = if player_x > self.x + self.width / 2.0 { 1.0 } else { -1.0 };
        self.speed_x = dir * speed;
        self.invulnerable = true;
        self.invuln_timer = invuln as f64;
        self.color = RED;
    }

    fn poison_cloud(&mut self, count: u32, bullet: &BulletDef, player_x: f32, player_y: f32) {
        for _ in 0..count {
            let px = player_x + rand::gen_range(-50.0, 50.0);
            let py = player_y + rand::gen_range(-50.0, 50.0);
            let p = self.make_projectile(px, py, 0.0, 0.0, bullet);
            self.projectiles.push(p);
        }
    }

    fn revive_minions(&mut self) {
        self.pattern_state.insert("revive_signal".to_string(), 1.0);
    }

    fn spawn_minions(&mut self) {
        self.pattern_state.insert("spawn_minions".to_string(), 1.0);
    }

    fn scanning_beam(&mut self, bullet: &BulletDef, player_x: f32) {
        let beam_x = player_x;
        for y in (0..screen_height() as i32).step_by(20) {
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
            (dx / dist) * 3.0,
            (dy / dist) * 3.0 - 2.0,
            bullet,
        );
        p.color = content::parse_color(color);
        self.projectiles.push(p);
    }

    fn feast_mode(&mut self, count: u32) {
        for _ in 0..count {
            self.projectiles.push(BossProjectile {
                x: rand::gen_range(0.0, screen_width()),
                y: -50.0,
                vel_x: rand::gen_range(-1.0, 1.0),
                vel_y: rand::gen_range(3.0, 6.0),
                damage: 1,
                size: rand::gen_range(15.0, 25.0),
                color: Color::new(
                    rand::gen_range(0.5, 1.0),
                    rand::gen_range(0.3, 0.8),
                    rand::gen_range(0.0, 0.5),
                    1.0
                ),
                lifetime: 10.0,
                pattern: ProjectilePattern::Straight,
            });
        }
    }

    fn update_projectiles(&mut self, dt: f64) {
        self.projectiles.retain_mut(|p| {
            p.lifetime -= dt;
            if p.lifetime <= 0.0 {
                return false;
            }

            match p.pattern {
                ProjectilePattern::Straight => {
                    p.x += p.vel_x * dt as f32;
                    p.y += p.vel_y * dt as f32;
                }
                ProjectilePattern::Spiral => {
                    p.vel_x = (p.vel_x + p.vel_y * 0.1).sin() * 4.0;
                    p.vel_y = (p.vel_y - p.vel_x * 0.1).cos() * 4.0;
                    p.x += p.vel_x * dt as f32;
                    p.y += p.vel_y * dt as f32;
                }
                ProjectilePattern::Homing => {
                    let target_x = screen_width() / 2.0;
                    let target_y = screen_height() - 100.0;
                    let dx = target_x - p.x;
                    let dy = target_y - p.y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist > 0.0 {
                        p.vel_x += (dx / dist) * 0.3;
                        p.vel_y += (dy / dist) * 0.3;
                        let speed = (p.vel_x * p.vel_x + p.vel_y * p.vel_y).sqrt();
                        if speed > 5.0 {
                            p.vel_x = p.vel_x / speed * 5.0;
                            p.vel_y = p.vel_y / speed * 5.0;
                        }
                    }
                    p.x += p.vel_x * dt as f32;
                    p.y += p.vel_y * dt as f32;
                }
                ProjectilePattern::Bounce => {
                    p.vel_y += 0.3 * dt as f32;
                    p.x += p.vel_x * dt as f32;
                    p.y += p.vel_y * dt as f32;
                    if p.y > screen_height() - 50.0 {
                        p.y = screen_height() - 50.0;
                        p.vel_y = -p.vel_y * 0.7;
                    }
                }
                ProjectilePattern::Wave => {
                    p.x += p.vel_x * dt as f32;
                    p.y += p.vel_y * dt as f32 + (p.x * 0.05).sin() * 2.0 * dt as f32;
                }
            }

            p.x > -50.0 && p.x < screen_width() + 50.0 && p.y > -50.0 && p.y < screen_height() + 50.0
        });
    }

    pub fn draw(&self) {
        if let Some(sprite) = &self.sprite {
            draw_texture_ex(
                sprite,
                self.x,
                self.y,
                self.color,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(self.width, self.height)),
                    ..Default::default()
                }
            );
        } else {
            draw_rectangle(self.x, self.y, self.width, self.height, self.color);
        }

        self.draw_health_bar();
        self.draw_projectiles();
    }

    fn draw_health_bar(&self) {
        let bar_width = self.width;
        let bar_height = 8.0;
        let bar_x = self.x;
        let bar_y = self.y - 15.0;
        
        draw_rectangle(bar_x, bar_y, bar_width, bar_height, Color::new(0.2, 0.0, 0.0, 0.8));
        let health_pct = self.health as f32 / self.max_health as f32;
        draw_rectangle(bar_x, bar_y, bar_width * health_pct, bar_height, RED);
        draw_rectangle_lines(bar_x, bar_y, bar_width, bar_height, 2.0, WHITE);
        
        let name = self.boss_type.name();
        let name_x = self.x + self.width / 2.0 - measure_text(name, None, 16, 1.0).width / 2.0;
        draw_text(name, name_x, bar_y - 5.0, 16.0, WHITE);
    }

    fn draw_projectiles(&self) {
        for p in &self.projectiles {
            draw_circle(p.x, p.y, p.size / 2.0, p.color);
            draw_circle_lines(p.x, p.y, p.size / 2.0, 2.0, WHITE);
        }
    }

    pub fn take_damage(&mut self, damage: i32) -> bool {
        if self.invulnerable || self.entry_anim {
            return false;
        }
        
        self.health -= damage;
        self.invulnerable = true;
        self.invuln_timer = 0.1;
        self.color = RED;
        
        if self.health <= 0 {
            self.health = 0;
            self.is_dead = true;
            crate::audio::play_sfx("boss_defeat");
            return true;
        }
        false
    }

    pub fn get_rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.width, self.height)
    }

    pub fn check_projectile_collision(&self, px: f32, py: f32, pw: f32, ph: f32) -> bool {
        for p in &self.projectiles {
            if px < p.x + p.size && px + pw > p.x - p.size
                && py < p.y + p.size && py + ph > p.y - p.size {
                return true;
            }
        }
        false
    }

    pub fn get_projectiles(&self) -> &Vec<BossProjectile> {
        &self.projectiles
    }

    pub fn clear_revive_signal(&mut self) {
        self.pattern_state.insert("revive_signal".to_string(), 0.0);
    }

    pub fn check_revive_signal(&self) -> bool {
        *self.pattern_state.get("revive_signal").unwrap_or(&0.0) > 0.0
    }

    pub fn check_spawn_minions(&self) -> bool {
        *self.pattern_state.get("spawn_minions").unwrap_or(&0.0) > 0.0
    }

    pub fn clear_spawn_minions(&mut self) {
        self.pattern_state.insert("spawn_minions".to_string(), 0.0);
    }

    pub fn is_reverse_zone_active(&self) -> bool {
        *self.pattern_state.get("reverse_zone_active").unwrap_or(&0.0) > 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_boss(health: i32, max_health: i32) -> Boss {
        let mut state = std::collections::HashMap::new();
        state.insert("burst_count".to_string(), 0.0);
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
            speed_x: 2.0,
            is_dead: false,
            invulnerable: false,
            invuln_timer: 0.0,
            projectiles: Vec::new(),
            pattern_state: state,
            movement: MovementDef::Patrol { speed_x: 2.0, base_y: 50.0, amp_y: 30.0, freq_y: 0.5 },
            phases: vec![
                BossPhaseDef { health_threshold: 1.0, attack_interval: 2.5, attacks: vec![] },
                BossPhaseDef { health_threshold: 0.5, attack_interval: 1.8, attacks: vec![] },
                BossPhaseDef { health_threshold: 0.2, attack_interval: 1.2, attacks: vec![] },
            ],
            sprite: None,
            portrait: None,
            color: WHITE,
            entry_anim: false,
            entry_timer: 0.0,
        }
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
    fn test_invulnerable_prevents_damage() {
        let mut boss = make_boss(150, 150);
        boss.invulnerable = true;
        let result = boss.take_damage(10);
        assert!(!result);
        assert_eq!(boss.health, 150);
    }

    #[test]
    fn test_entry_anim_prevents_damage() {
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
        assert_eq!(boss.invuln_timer, 0.1);
    }

    #[test]
    fn test_phase_transition_to_phase2() {
        let mut boss = make_boss(75, 150);
        boss.update_phase();
        assert_eq!(boss.phase, BossPhase::Phase2);
    }

    #[test]
    fn test_phase_transition_to_phase3() {
        let mut boss = make_boss(30, 150);
        boss.update_phase();
        assert_eq!(boss.phase, BossPhase::Phase3);
    }

    #[test]
    fn test_phase_remains_phase1_above_half() {
        let mut boss = make_boss(100, 150);
        boss.update_phase();
        assert_eq!(boss.phase, BossPhase::Phase1);
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

    #[test]
    fn test_boss_phase_defaults_to_phase1() {
        let boss = make_boss(150, 150);
        assert_eq!(boss.phase, BossPhase::Phase1);
        assert!(!boss.is_dead);
    }
}
