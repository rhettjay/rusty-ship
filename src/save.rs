use std::fs;
use bincode;
use serde::{Serialize, Deserialize};
use crate::menu::NarrativeProgress;
use crate::wave_director::WaveState;

const SAVE_FILE: &str = "savegame.bin";

#[derive(Serialize, Deserialize)]
pub struct WaveStateSave {
    pub current_wave: u32,
    pub wave_state: WaveState,
    pub wave_timer: f64,
    pub enemies_spawned: u32,
    pub powerup_rapid_fire_timer: f32,
    pub powerup_spread_shot_timer: f32,
    pub powerup_pierce_timer: f32,
    pub powerup_has_shield: bool,
}

#[derive(Serialize, Deserialize)]
pub struct GameSave {
    pub score: i32,
    pub lives: i32,
    pub current_wave: u32,
    pub wave_state_save: WaveStateSave,
    pub ship_x: f32,
    pub ship_y: f32,
    pub ship_has_shield: bool,
    pub ship_rapid_fire_timer: f32,
    pub ship_spread_shot_timer: f32,
    pub ship_pierce_timer: f32,
    pub narrative: NarrativeProgress,
}

pub fn save_game(
    score: i32,
    lives: i32,
    current_wave: u32,
    wave_state: WaveState,
    ship_x: f32,
    ship_y: f32,
    ship_has_shield: bool,
    ship_rapid_fire_timer: f32,
    ship_spread_shot_timer: f32,
    ship_pierce_timer: f32,
    narrative: NarrativeProgress,
    wave_timer: f64,
    enemies_spawned: u32,
    powerup_rapid_fire_timer: f32,
    powerup_spread_shot_timer: f32,
    powerup_pierce_timer: f32,
    powerup_has_shield: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if matches!(wave_state, WaveState::BossFight) {
        return Err("Cannot save during boss fight".into());
    }

    let wave_state_save = WaveStateSave {
        current_wave,
        wave_state,
        wave_timer,
        enemies_spawned,
        powerup_rapid_fire_timer,
        powerup_spread_shot_timer,
        powerup_pierce_timer,
        powerup_has_shield,
    };

    let save = GameSave {
        score,
        lives,
        current_wave,
        wave_state_save,
        ship_x,
        ship_y,
        ship_has_shield,
        ship_rapid_fire_timer,
        ship_spread_shot_timer,
        ship_pierce_timer,
        narrative,
    };

    let encoded = bincode::serialize(&save)?;
    fs::write(SAVE_FILE, encoded)?;
    Ok(())
}

pub fn load_game() -> Result<GameSave, Box<dyn std::error::Error>> {
    let data = fs::read(SAVE_FILE)?;
    let save: GameSave = bincode::deserialize(&data)?;
    Ok(save)
}

pub fn has_save_file() -> bool {
    std::path::Path::new(SAVE_FILE).exists()
}

pub fn delete_save_file() {
    let _ = fs::remove_file(SAVE_FILE);
}

pub fn apply_save(
    save: GameSave,
    ship: &mut crate::ship::Ship,
    wave_director: &mut crate::wave_director::WaveDirector,
    game: &mut crate::menu::Game,
    score: &mut i32,
    lives: &mut i32,
) {
    *score = save.score;
    *lives = save.lives;
    
    ship.x = save.ship_x;
    ship.y = save.ship_y;
    ship.has_shield = save.ship_has_shield;
    ship.rapid_fire_timer = save.ship_rapid_fire_timer;
    ship.spread_shot_timer = save.ship_spread_shot_timer;
    ship.pierce_timer = save.ship_pierce_timer;

    game.narrative = save.narrative;
    
    wave_director.current_wave = save.wave_state_save.current_wave;
    wave_director.wave_state = save.wave_state_save.wave_state;
    wave_director.wave_timer = save.wave_state_save.wave_timer;
    wave_director.enemies_spawned = save.wave_state_save.enemies_spawned;
    wave_director.powerup_manager.rapid_fire_timer = save.wave_state_save.powerup_rapid_fire_timer;
    wave_director.powerup_manager.spread_shot_timer = save.wave_state_save.powerup_spread_shot_timer;
    wave_director.powerup_manager.pierce_timer = save.wave_state_save.powerup_pierce_timer;
    wave_director.powerup_manager.has_shield = save.wave_state_save.powerup_has_shield;

    if matches!(wave_director.wave_state, crate::wave_director::WaveState::BossIntro) {
        wave_director.is_boss_wave = true;
    } else {
        wave_director.is_boss_wave = matches!(save.wave_state_save.current_wave, 5 | 10 | 15);
    }
}