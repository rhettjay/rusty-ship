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
            
            bonus_score_threshold: 50000,
            
            dialogue_typewriter_speed: 30.0,
            dialogue_box_padding: 20.0,
            
            music_volume: 0.5,
            sfx_volume: 0.7,
        }
    }
}

pub const CONFIG: GameConfig = GameConfig::new();

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