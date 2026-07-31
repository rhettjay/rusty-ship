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
        match self {
            BossType::Blowfish => 150,
            BossType::Twofish => 200,
            BossType::RufusReverse => 180,
            BossType::MollyHashpass => 220,
            BossType::CaptainDavey => 500,
            BossType::Deadbeef => 300,
        }
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
        match s {
            "Blowfish" => Ok(BossType::Blowfish),
            "Twofish" => Ok(BossType::Twofish),
            "RufusReverse" => Ok(BossType::RufusReverse),
            "MollyHashpass" => Ok(BossType::MollyHashpass),
            "CaptainDavey" => Ok(BossType::CaptainDavey),
            "Deadbeef" => Ok(BossType::Deadbeef),
            _ => Err(()),
        }
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
    pub speed_y: f32,
    pub is_dead: bool,
    pub invulnerable: bool,
    pub invuln_timer: f64,
    pub projectiles: Vec<BossProjectile>,
    pub pattern_state: HashMap<String, f32>,
    pub sprite: Option<Texture2D>,
    pub portrait: Option<Texture2D>,
    pub color: Color,
    pub entry_anim: bool,
    pub entry_timer: f64,
}

impl Boss {
    pub fn new(boss_type: BossType) -> Self {
        let (width, height) = match boss_type {
            BossType::CaptainDavey => (120.0, 120.0),
            _ => (80.0, 80.0),
        };

        let mut boss = Self {
            boss_type,
            x: screen_width() / 2.0 - width / 2.0,
            y: -height,
            width,
            height,
            health: boss_type.max_health(),
            max_health: boss_type.max_health(),
            phase: BossPhase::Phase1,
            phase_timer: 0.0,
            attack_timer: 0.0,
            move_timer: 0.0,
            speed_x: 2.0,
            speed_y: 1.0,
            is_dead: false,
            invulnerable: false,
            invuln_timer: 0.0,
            projectiles: Vec::new(),
            pattern_state: HashMap::new(),
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
        let new_phase = if health_pct <= 0.2 {
            BossPhase::Phase3
        } else if health_pct <= 0.5 {
            BossPhase::Phase2
        } else {
            BossPhase::Phase1
        };

        if new_phase != self.phase {
            self.phase = new_phase;
            self.phase_timer = 0.0;
            crate::audio::play_sfx("boss_laugh");
        }
        self.phase_timer += 1.0 / 60.0;
    }

    fn update_movement(&mut self, dt: f64) {
        self.move_timer += dt;
        
        match self.boss_type {
            BossType::Blowfish => {
                self.x += self.speed_x * dt as f32;
                if self.x <= 50.0 || self.x >= screen_width() - self.width - 50.0 {
                    self.speed_x = -self.speed_x;
                }
                self.y = 50.0 + (self.move_timer * 0.5).sin() as f32 * 30.0;
            }
            BossType::Twofish => {
                self.x += self.speed_x * dt as f32;
                if self.x <= 50.0 || self.x >= screen_width() - self.width - 50.0 {
                    self.speed_x = -self.speed_x;
                }
                self.y = 50.0 + (self.move_timer * 0.7).sin() as f32 * 40.0;
            }
            BossType::RufusReverse => {
                if *self.pattern_state.get("teleport_cooldown").unwrap_or(&0.0) <= 0.0 {
                    if rand::gen_range(0, 100) < 2 {
                        self.x = rand::gen_range(100.0, screen_width() - self.width - 100.0);
                        self.y = rand::gen_range(50.0, 200.0);
                        self.pattern_state.insert("teleport_cooldown".to_string(), 3.0);
                    }
                } else {
                    *self.pattern_state.get_mut("teleport_cooldown").unwrap() -= dt as f32;
                }
            }
            BossType::MollyHashpass => {
                self.x += self.speed_x * dt as f32;
                if self.x <= 50.0 || self.x >= screen_width() - self.width - 50.0 {
                    self.speed_x = -self.speed_x;
                }
                self.y = 60.0 + (self.move_timer * 0.4).sin() as f32 * 20.0;
            }
            BossType::CaptainDavey => {
                self.x = screen_width() / 2.0 - self.width / 2.0 + (self.move_timer * 0.3).sin() as f32 * 100.0;
                self.y = 50.0 + (self.move_timer * 0.5).sin() as f32 * 30.0;
            }
            BossType::Deadbeef => {
                self.x += self.speed_x * dt as f32 * 1.5;
                if self.x <= 50.0 || self.x >= screen_width() - self.width - 50.0 {
                    self.speed_x = -self.speed_x;
                }
                self.y = 80.0 + (self.move_timer * 0.8).sin() as f32 * 50.0;
            }
        }
    }

    fn update_attacks(&mut self, dt: f64, player_x: f32, player_y: f32) {
        self.attack_timer += dt;
        
        match self.boss_type {
            BossType::Blowfish => self.attack_blowfish(dt, player_x, player_y),
            BossType::Twofish => self.attack_twofish(dt, player_x, player_y),
            BossType::RufusReverse => self.attack_rufus(dt, player_x, player_y),
            BossType::MollyHashpass => self.attack_molly(dt, player_x, player_y),
            BossType::CaptainDavey => self.attack_davey(dt, player_x, player_y),
            BossType::Deadbeef => self.attack_deadbeef(dt, player_x, player_y),
        }
    }

    fn attack_blowfish(&mut self, dt: f64, player_x: f32, player_y: f32) {
        let attack_interval = match self.phase {
            BossPhase::Phase1 => 2.5,
            BossPhase::Phase2 => 1.8,
            BossPhase::Phase3 => 1.2,
        };

        if self.attack_timer >= attack_interval {
            self.attack_timer = 0.0;
            
            match self.phase {
                BossPhase::Phase1 => {
                    self.burst_attack(8, player_x, player_y);
                }
                BossPhase::Phase2 => {
                    self.burst_attack(16, player_x, player_y);
                    if rand::gen_range(0, 2) == 0 {
                        self.charge_attack(player_x);
                    }
                }
                BossPhase::Phase3 => {
                    self.burst_attack(24, player_x, player_y);
                    self.charge_attack(player_x);
                }
            }
        }
    }

    fn attack_twofish(&mut self, dt: f64, player_x: f32, player_y: f32) {
        let attack_interval = match self.phase {
            BossPhase::Phase1 => 2.0,
            BossPhase::Phase2 => 1.5,
            BossPhase::Phase3 => 1.0,
        };

        if self.attack_timer >= attack_interval {
            self.attack_timer = 0.0;
            
            match self.phase {
                BossPhase::Phase1 => {
                    self.mirror_shot(player_x, player_y);
                }
                BossPhase::Phase2 => {
                    self.split_attack(player_x, player_y);
                }
                BossPhase::Phase3 => {
                    self.mirror_shot(player_x, player_y);
                    self.homing_missiles(player_x, player_y, 4);
                }
            }
        }
    }

    fn attack_rufus(&mut self, dt: f64, player_x: f32, player_y: f32) {
        let attack_interval = match self.phase {
            BossPhase::Phase1 => 3.0,
            BossPhase::Phase2 => 2.0,
            BossPhase::Phase3 => 1.5,
        };

        if self.attack_timer >= attack_interval {
            self.attack_timer = 0.0;
            
            match self.phase {
                BossPhase::Phase1 => {
                    self.reverse_zone_attack();
                    self.scatter_shot(player_x, player_y);
                }
                BossPhase::Phase2 => {
                    self.reverse_zone_attack();
                    self.mirror_shot(player_x, player_y);
                }
                BossPhase::Phase3 => {
                    self.full_reverse();
                    self.scatter_shot(player_x, player_y);
                    self.homing_missiles(player_x, player_y, 3);
                }
            }
        }
    }

    fn attack_molly(&mut self, dt: f64, player_x: f32, player_y: f32) {
        let attack_interval = match self.phase {
            BossPhase::Phase1 => 2.5,
            BossPhase::Phase2 => 2.0,
            BossPhase::Phase3 => 1.5,
        };

        if self.attack_timer >= attack_interval {
            self.attack_timer = 0.0;
            
            match self.phase {
                BossPhase::Phase1 => {
                    self.poison_shot(player_x, player_y);
                    self.heal_nearby();
                }
                BossPhase::Phase2 => {
                    self.poison_cloud(player_x, player_y);
                    self.revive_minions();
                }
                BossPhase::Phase3 => {
                    self.heal_beam();
                    self.poison_shot(player_x, player_y);
                    self.poison_cloud(player_x, player_y);
                }
            }
        }
    }

    fn attack_davey(&mut self, dt: f64, player_x: f32, player_y: f32) {
        let attack_interval = match self.phase {
            BossPhase::Phase1 => 3.0,
            BossPhase::Phase2 => 2.5,
            BossPhase::Phase3 => 2.0,
        };

        if self.attack_timer >= attack_interval {
            self.attack_timer = 0.0;
            
            match self.phase {
                BossPhase::Phase1 => {
                    self.scanning_beam(player_x);
                }
                BossPhase::Phase2 => {
                    self.scanning_beam(player_x);
                    self.spawn_minions();
                }
                BossPhase::Phase3 => {
                    self.broadside_attack();
                    self.scanning_beam(player_x);
                }
            }
        }
    }

    fn attack_deadbeef(&mut self, dt: f64, player_x: f32, player_y: f32) {
        let attack_interval = match self.phase {
            BossPhase::Phase1 => 1.5,
            BossPhase::Phase2 => 1.2,
            BossPhase::Phase3 => 0.8,
        };

        if self.attack_timer >= attack_interval {
            self.attack_timer = 0.0;
            
            match self.phase {
                BossPhase::Phase1 => {
                    self.throw_food(player_x, player_y);
                }
                BossPhase::Phase2 => {
                    self.throw_food(player_x, player_y);
                    self.spicy_sauce(player_x, player_y);
                }
                BossPhase::Phase3 => {
                    self.feast_mode();
                }
            }
        }
    }

    fn burst_attack(&mut self, count: i32, player_x: f32, player_y: f32) {
        let center_x = self.x + self.width / 2.0;
        let center_y = self.y + self.height;
        let angle_step = std::f32::consts::TAU / count as f32;
        
        for i in 0..count {
            let angle = i as f32 * angle_step;
            self.projectiles.push(BossProjectile {
                x: center_x,
                y: center_y,
                vel_x: angle.cos() * 4.0,
                vel_y: angle.sin() * 4.0,
                damage: 1,
                size: 12.0,
                color: ORANGE,
                lifetime: 5.0,
                pattern: ProjectilePattern::Straight,
            });
        }
    }

    fn charge_attack(&mut self, player_x: f32) {
        let dir = if player_x > self.x + self.width / 2.0 { 1.0 } else { -1.0 };
        self.speed_x = dir * 8.0;
        self.invulnerable = true;
        self.invuln_timer = 1.0;
        self.color = RED;
    }

    fn mirror_shot(&mut self, player_x: f32, player_y: f32) {
        let center_x = self.x + self.width / 2.0;
        let center_y = self.y + self.height;
        
        for dir in [-1.0, 1.0] {
            let dx = player_x - center_x;
            let dy = player_y - center_y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist > 0.0 {
                self.projectiles.push(BossProjectile {
                    x: center_x,
                    y: center_y,
                    vel_x: (dx / dist) * 5.0 * dir,
                    vel_y: (dy / dist) * 5.0,
                    damage: 1,
                    size: 14.0,
                    color: BLUE,
                    lifetime: 4.0,
                    pattern: ProjectilePattern::Straight,
                });
            }
        }
    }

    fn split_attack(&mut self, player_x: f32, player_y: f32) {
        let center_x = self.x + self.width / 2.0;
        let center_y = self.y + self.height;
        
        for i in 0..4 {
            let angle = i as f32 * std::f32::consts::FRAC_PI_2;
            self.projectiles.push(BossProjectile {
                x: center_x,
                y: center_y,
                vel_x: angle.cos() * 4.0,
                vel_y: angle.sin() * 4.0,
                damage: 1,
                size: 16.0,
                color: PURPLE,
                lifetime: 5.0,
                pattern: ProjectilePattern::Spiral,
            });
        }
    }

    fn homing_missiles(&mut self, player_x: f32, player_y: f32, count: i32) {
        let center_x = self.x + self.width / 2.0;
        let center_y = self.y + self.height;
        
        for _ in 0..count {
            self.projectiles.push(BossProjectile {
                x: center_x,
                y: center_y,
                vel_x: 0.0,
                vel_y: 0.0,
                damage: 1,
                size: 10.0,
                color: YELLOW,
                lifetime: 6.0,
                pattern: ProjectilePattern::Homing,
            });
        }
    }

    fn reverse_zone_attack(&mut self) {
        *self.pattern_state.get_mut("reverse_zone_active").unwrap() = 5.0;
    }

    fn scatter_shot(&mut self, player_x: f32, player_y: f32) {
        let center_x = self.x + self.width / 2.0;
        let center_y = self.y + self.height;
        let count = 8;
        let spread = std::f32::consts::FRAC_PI_2;
        let dx = player_x - center_x;
        let dy = player_y - center_y;
        let dist = (dx * dx + dy * dy).sqrt();
        let base_angle = if dist > 0.0 { dy.atan2(dx) } else { std::f32::consts::FRAC_PI_2 };
        let speed = 4.0;
        
        for i in 0..count {
            let t = i as f32 / (count - 1).max(1) as f32;
            let angle = base_angle + (t - 0.5) * spread;
            self.projectiles.push(BossProjectile {
                x: center_x,
                y: center_y,
                vel_x: angle.cos() * speed,
                vel_y: angle.sin() * speed,
                damage: 1,
                size: 10.0,
                color: PINK,
                lifetime: 4.0,
                pattern: ProjectilePattern::Straight,
            });
        }
    }

    fn full_reverse(&mut self) {
        *self.pattern_state.get_mut("reverse_zone_active").unwrap() = 10.0;
    }

    fn poison_shot(&mut self, player_x: f32, player_y: f32) {
        let center_x = self.x + self.width / 2.0;
        let center_y = self.y + self.height;
        let dx = player_x - center_x;
        let dy = player_y - center_y;
        let dist = (dx * dx + dy * dy).sqrt();
        
        if dist > 0.0 {
            for i in -1..=1 {
                let angle = (dy / dist).atan2(dx / dist) + i as f32 * 0.2;
                self.projectiles.push(BossProjectile {
                    x: center_x,
                    y: center_y,
                    vel_x: angle.cos() * 4.0,
                    vel_y: angle.sin() * 4.0,
                    damage: 1,
                    size: 12.0,
                    color: GREEN,
                    lifetime: 4.0,
                    pattern: ProjectilePattern::Straight,
                });
            }
        }
    }

    fn heal_nearby(&mut self) {
        self.health = (self.health + 5).min(self.max_health);
    }

    fn poison_cloud(&mut self, player_x: f32, player_y: f32) {
        for _ in 0..3 {
            let px = player_x + rand::gen_range(-50.0, 50.0);
            let py = player_y + rand::gen_range(-50.0, 50.0);
            self.projectiles.push(BossProjectile {
                x: px,
                y: py,
                vel_x: 0.0,
                vel_y: 0.0,
                damage: 1,
                size: 40.0,
                color: Color::new(0.0, 0.8, 0.0, 0.5),
                lifetime: 3.0,
                pattern: ProjectilePattern::Straight,
            });
        }
    }

    fn revive_minions(&mut self) {
        // Signal to game logic to revive dead pirates
        self.pattern_state.insert("revive_signal".to_string(), 1.0);
    }

    fn heal_beam(&mut self) {
        self.invulnerable = true;
        self.invuln_timer = 3.0;
        self.color = Color::new(0.0, 1.0, 0.0, 0.7);
        self.health = (self.health + 30).min(self.max_health);
    }

    fn scanning_beam(&mut self, player_x: f32) {
        let beam_x = player_x;
        for y in (0..screen_height() as i32).step_by(20) {
            self.projectiles.push(BossProjectile {
                x: beam_x,
                y: y as f32,
                vel_x: 0.0,
                vel_y: 0.0,
                damage: 2,
                size: 8.0,
                color: RED,
                lifetime: 0.5,
                pattern: ProjectilePattern::Straight,
            });
        }
    }

    fn spawn_minions(&mut self) {
        self.pattern_state.insert("spawn_minions".to_string(), 1.0);
    }

    fn broadside_attack(&mut self) {
        let rows = 5;
        let cols = 20;
        let start_y = 100.0;
        let row_spacing = (screen_height() - 200.0) / rows as f32;
        
        for row in 0..rows {
            let gap_start = rand::gen_range(2, cols - 2);
            let gap_size = 3 + self.phase as usize;
            
            for col in 0..cols {
                if col < gap_start || col >= gap_start + gap_size {
                    self.projectiles.push(BossProjectile {
                        x: col as f32 * screen_width() / cols as f32,
                        y: start_y + row as f32 * row_spacing,
                        vel_x: 0.0,
                        vel_y: 8.0,
                        damage: 1,
                        size: 16.0,
                        color: Color::new(1.0, 0.5, 0.0, 1.0),
                        lifetime: 8.0,
                        pattern: ProjectilePattern::Straight,
                    });
                }
            }
        }
    }

    fn throw_food(&mut self, player_x: f32, player_y: f32) {
        let center_x = self.x + self.width / 2.0;
        let center_y = self.y + self.height;
        let food_types = ["burger", "pizza", "taco", "donut"];
        let food_type = food_types[rand::gen_range(0, food_types.len())];
        
        let dx = player_x - center_x;
        let dy = player_y - center_y;
        let dist = (dx * dx + dy * dy).sqrt();
        
        if dist > 0.0 {
            self.projectiles.push(BossProjectile {
                x: center_x,
                y: center_y,
                vel_x: (dx / dist) * 3.0,
                vel_y: (dy / dist) * 3.0 - 2.0,
                damage: 1,
                size: 20.0,
                color: match food_type {
                    "burger" => BROWN,
                    "pizza" => ORANGE,
                    "taco" => YELLOW,
                    _ => PINK,
                },
                lifetime: 5.0,
                pattern: ProjectilePattern::Bounce,
            });
        }
    }

    fn spicy_sauce(&mut self, player_x: f32, player_y: f32) {
        let center_x = self.x + self.width / 2.0;
        let center_y = self.y + self.height;
        
        for i in 0..8 {
            let angle = i as f32 * std::f32::consts::FRAC_PI_4;
            self.projectiles.push(BossProjectile {
                x: center_x,
                y: center_y,
                vel_x: angle.cos() * 4.0,
                vel_y: angle.sin() * 4.0,
                damage: 1,
                size: 8.0,
                color: RED,
                lifetime: 4.0,
                pattern: ProjectilePattern::Straight,
            });
        }
    }

    fn feast_mode(&mut self) {
        for _ in 0..30 {
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
            speed_y: 1.0,
            is_dead: false,
            invulnerable: false,
            invuln_timer: 0.0,
            projectiles: Vec::new(),
            pattern_state: state,
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