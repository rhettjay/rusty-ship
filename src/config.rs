use macroquad::prelude::*;

pub struct GameConfig {
    pub window_width: f32,
    pub window_height: f32,
    pub window_title: &'static str,
    
    pub ship_speed: f32,
    pub ship_width: f32,
    pub ship_height: f32,
    pub ship_start_y_offset: f32,
    
    pub cannonball_speed: f32,
    pub cannonball_width: f32,
    pub cannonball_height: f32,
    pub cannonball_cooldown: f64,
    
    pub pirate_base_speed_x_range: (f32, f32),
    pub pirate_base_speed_y_range: (f32, f32),
    pub pirate_spawn_chance: u32,
    pub pirate_max_count: i32,
    pub pirate_width: f32,
    pub pirate_height: f32,
    pub pirate_rotation: f32,
    
    pub starting_lives: i32,
    pub starting_pirate_count: i32,
    
    pub waves_per_chapter: u32,
    pub boss_waves: &'static [u32],
    pub bonus_score_threshold: i32,
    
    pub dialogue_typewriter_speed: f32,
    pub dialogue_box_padding: f32,
    
    pub music_volume: f32,
    pub sfx_volume: f32,
}

impl GameConfig {
    pub const fn new() -> Self {
        Self {
            window_width: 800.0,
            window_height: 800.0,
            window_title: "rusty-ship",
            
            ship_speed: 5.0,
            ship_width: 60.0,
            ship_height: 64.0,
            ship_start_y_offset: 100.0,
            
            cannonball_speed: 10.0,
            cannonball_width: 5.0,
            cannonball_height: 15.0,
            cannonball_cooldown: 0.2,
            
            pirate_base_speed_x_range: (1.0, 8.0),
            pirate_base_speed_y_range: (1.0, 3.0),
            pirate_spawn_chance: 25,
            pirate_max_count: 10,
            pirate_width: 15.0,
            pirate_height: 15.0,
            pirate_rotation: 3.14159265359,
            
            starting_lives: 3,
            starting_pirate_count: 10,
            
            waves_per_chapter: 3,
            boss_waves: &[5, 10, 15],
            bonus_score_threshold: 50000,
            
            dialogue_typewriter_speed: 30.0,
            dialogue_box_padding: 20.0,
            
            music_volume: 0.5,
            sfx_volume: 0.7,
        }
    }
}

pub const CONFIG: GameConfig = GameConfig::new();

pub struct WaveConfig {
    pub wave_number: u32,
    pub duration: f32,
    pub max_enemies: u32,
    pub spawn_interval: f32,
    pub formation_types: &'static [FormationType],
    pub powerup_chance: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FormationType {
    Random,
    Vee,
    Line,
    Circle,
    Escort,
    Grid,
    Chaos,
}

pub struct EnemyConfig {
    pub base_hp: i32,
    pub base_armor: i32,
    pub speed_range: (f32, f32),
    pub size: (f32, f32),
    pub shoot_pattern: &'static str,
    pub shoot_interval: f64,
    pub score_value: i32,
    pub powerup_chance: f32,
}

pub const ENEMY_CONFIG: &[EnemyConfig] = &[
    EnemyConfig {
        base_hp: 1,
        base_armor: 0,
        speed_range: (3.0, 5.0),
        size: (24.0, 24.0),
        shoot_pattern: "none",
        shoot_interval: 0.0,
        score_value: 10,
        powerup_chance: 0.03,
    },
    EnemyConfig {
        base_hp: 2,
        base_armor: 0,
        speed_range: (1.5, 3.0),
        size: (32.0, 32.0),
        shoot_pattern: "straight",
        shoot_interval: 2.0,
        score_value: 25,
        powerup_chance: 0.05,
    },
    EnemyConfig {
        base_hp: 3,
        base_armor: 1,
        speed_range: (0.8, 1.5),
        size: (40.0, 40.0),
        shoot_pattern: "bomb",
        shoot_interval: 3.0,
        score_value: 50,
        powerup_chance: 0.10,
    },
    EnemyConfig {
        base_hp: 2,
        base_armor: 0,
        speed_range: (2.5, 4.0),
        size: (28.0, 28.0),
        shoot_pattern: "aimed",
        shoot_interval: 1.5,
        score_value: 40,
        powerup_chance: 0.07,
    },
    EnemyConfig {
        base_hp: 4,
        base_armor: 2,
        speed_range: (2.0, 3.5),
        size: (32.0, 32.0),
        shoot_pattern: "spread",
        shoot_interval: 1.2,
        score_value: 100,
        powerup_chance: 0.25,
    },
];

pub const WAVE_CONFIG: &[WaveConfig] = &[
    WaveConfig { wave_number: 1, duration: 20.0, max_enemies: 6, spawn_interval: 2.0, formation_types: &[FormationType::Random], powerup_chance: 0.08 },
    WaveConfig { wave_number: 2, duration: 25.0, max_enemies: 8, spawn_interval: 1.5, formation_types: &[FormationType::Random], powerup_chance: 0.07 },
    WaveConfig { wave_number: 3, duration: 30.0, max_enemies: 10, spawn_interval: 1.8, formation_types: &[FormationType::Vee], powerup_chance: 0.06 },
    WaveConfig { wave_number: 4, duration: 30.0, max_enemies: 12, spawn_interval: 1.5, formation_types: &[FormationType::Line], powerup_chance: 0.06 },
    WaveConfig { wave_number: 5, duration: 0.0, max_enemies: 0, spawn_interval: 0.0, formation_types: &[], powerup_chance: 0.0 }, // Boss
    WaveConfig { wave_number: 6, duration: 35.0, max_enemies: 14, spawn_interval: 1.2, formation_types: &[FormationType::Line, FormationType::Vee], powerup_chance: 0.05 },
    WaveConfig { wave_number: 7, duration: 35.0, max_enemies: 16, spawn_interval: 1.0, formation_types: &[FormationType::Circle], powerup_chance: 0.05 },
    WaveConfig { wave_number: 8, duration: 40.0, max_enemies: 14, spawn_interval: 1.0, formation_types: &[FormationType::Escort], powerup_chance: 0.05 },
    WaveConfig { wave_number: 9, duration: 40.0, max_enemies: 16, spawn_interval: 0.8, formation_types: &[FormationType::Vee], powerup_chance: 0.05 },
    WaveConfig { wave_number: 10, duration: 0.0, max_enemies: 0, spawn_interval: 0.0, formation_types: &[], powerup_chance: 0.0 }, // Boss
    WaveConfig { wave_number: 11, duration: 45.0, max_enemies: 18, spawn_interval: 0.9, formation_types: &[FormationType::Grid, FormationType::Circle], powerup_chance: 0.04 },
    WaveConfig { wave_number: 12, duration: 45.0, max_enemies: 20, spawn_interval: 0.7, formation_types: &[FormationType::Circle, FormationType::Grid], powerup_chance: 0.04 },
    WaveConfig { wave_number: 13, duration: 50.0, max_enemies: 22, spawn_interval: 0.6, formation_types: &[FormationType::Chaos], powerup_chance: 0.03 },
    WaveConfig { wave_number: 14, duration: 50.0, max_enemies: 24, spawn_interval: 0.5, formation_types: &[FormationType::Chaos], powerup_chance: 0.03 },
    WaveConfig { wave_number: 15, duration: 0.0, max_enemies: 0, spawn_interval: 0.0, formation_types: &[], powerup_chance: 0.0 }, // Final Boss
];

pub const POWERUP_CONFIG: &[PowerupConfig] = &[
    PowerupConfig { powerup_type: PowerupType::RapidFire, duration: 10.0, weight: 25 },
    PowerupConfig { powerup_type: PowerupType::SpreadShot, duration: 10.0, weight: 20 },
    PowerupConfig { powerup_type: PowerupType::Pierce, duration: 10.0, weight: 15 },
    PowerupConfig { powerup_type: PowerupType::Shield, duration: 0.0, weight: 15 },
    PowerupConfig { powerup_type: PowerupType::Bomb, duration: 0.0, weight: 10 },
    PowerupConfig { powerup_type: PowerupType::Life, duration: 0.0, weight: 10 },
    PowerupConfig { powerup_type: PowerupType::Score, duration: 0.0, weight: 5 },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerupType {
    RapidFire,
    SpreadShot,
    Pierce,
    Shield,
    Bomb,
    Life,
    Score,
}

impl PowerupType {
    pub fn sprite_name(&self) -> &'static str {
        match self {
            PowerupType::RapidFire => "powerup_rapid_fire",
            PowerupType::SpreadShot => "powerup_spread_shot",
            PowerupType::Pierce => "powerup_pierce",
            PowerupType::Shield => "powerup_shield",
            PowerupType::Bomb => "powerup_bomb",
            PowerupType::Life => "powerup_life",
            PowerupType::Score => "powerup_score",
        }
    }
    
    pub fn color(&self) -> Color {
        match self {
            PowerupType::RapidFire => YELLOW,
            PowerupType::SpreadShot => SKYBLUE,
            PowerupType::Pierce => LIME,
            PowerupType::Shield => WHITE,
            PowerupType::Bomb => RED,
            PowerupType::Life => PINK,
            PowerupType::Score => GOLD,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PowerupConfig {
    pub powerup_type: PowerupType,
    pub duration: f32,
    pub weight: u32,
}