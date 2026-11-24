use macroquad::prelude::*;

pub struct GameConfig {
    pub window_width: f32,
    pub window_height: f32,
    pub window_title: &'static str,

    pub ship_speed: f32,
    pub ship_width: f32,
    pub ship_height: f32,
    pub ship_start_y_offset: f32,

    /// Seconds between player shots (rapid fire scales this down).
    pub fire_cooldown: f64,

    pub pirate_base_speed_x_range: (f32, f32),
    pub pirate_width: f32,
    pub pirate_height: f32,

    pub starting_lives: i32,

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

            // Pixels per second.
            ship_speed: 300.0,
            ship_width: 60.0,
            ship_height: 64.0,
            ship_start_y_offset: 100.0,

            fire_cooldown: 0.2,

            pirate_base_speed_x_range: (60.0, 480.0),
            pirate_width: 15.0,
            pirate_height: 15.0,

            starting_lives: 3,

            bonus_score_threshold: 50000,

            dialogue_typewriter_speed: 50.0,
            dialogue_box_padding: 20.0,

            music_volume: 0.5,
            sfx_volume: 0.7,
        }
    }
}

pub const CONFIG: GameConfig = GameConfig::new();

pub const POWERUP_CONFIG: &[PowerupConfig] = &[
    PowerupConfig {
        powerup_type: PowerupType::RapidFire,
        duration: 10.0,
        weight: 25,
    },
    PowerupConfig {
        powerup_type: PowerupType::SpreadShot,
        duration: 10.0,
        weight: 20,
    },
    PowerupConfig {
        powerup_type: PowerupType::Pierce,
        duration: 10.0,
        weight: 15,
    },
    PowerupConfig {
        powerup_type: PowerupType::Shield,
        duration: 0.0,
        weight: 15,
    },
    PowerupConfig {
        powerup_type: PowerupType::Bomb,
        duration: 0.0,
        weight: 10,
    },
    PowerupConfig {
        powerup_type: PowerupType::Life,
        duration: 0.0,
        weight: 10,
    },
    PowerupConfig {
        powerup_type: PowerupType::Score,
        duration: 0.0,
        weight: 5,
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
    /// Asset file stem under assets/powerups/.
    pub fn sprite_name(&self) -> &'static str {
        match self {
            PowerupType::RapidFire => "rapid_fire",
            PowerupType::SpreadShot => "spread_shot",
            PowerupType::Pierce => "pierce",
            PowerupType::Shield => "shield",
            PowerupType::Bomb => "bomb",
            PowerupType::Life => "life",
            PowerupType::Score => "score",
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
    /// Relative chance of dropping when a powerup drop is rolled.
    pub weight: u32,
}
