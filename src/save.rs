use crate::menu::NarrativeProgress;
use crate::wave_director::WaveState;
use serde::{Deserialize, Serialize};
use std::fs;

const SAVE_FILE: &str = "savegame.bin";

#[derive(Serialize, Deserialize)]
pub struct GameSave {
    pub score: i32,
    pub lives: i32,
    pub current_wave: u32,
    pub wave_state: WaveState,
    pub wave_timer: f64,
    pub enemies_spawned: u32,
    pub powerup_rapid_fire_timer: f32,
    pub powerup_spread_shot_timer: f32,
    pub powerup_pierce_timer: f32,
    pub powerup_has_shield: bool,
    pub narrative: NarrativeProgress,
}

/// Writes a save. Saves are only taken at safe points: wave starts (auto)
/// and pause-menu saves mid-wave; saving during an active boss fight is
/// refused because a half-fought boss cannot be reconstructed.
pub fn save_game(save: &GameSave) -> Result<(), Box<dyn std::error::Error>> {
    if matches!(save.wave_state, WaveState::BossFight) {
        return Err("Cannot save during boss fight".into());
    }

    let encoded = bincode::serialize(save)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu::NarrativeProgress;

    /// A save taken at a wave start must round-trip through bincode.
    #[test]
    fn test_save_round_trip() {
        let narrative = NarrativeProgress::new();
        let save = GameSave {
            score: 1234,
            lives: 2,
            current_wave: 7,
            wave_state: WaveState::Spawning,
            wave_timer: 3.25,
            enemies_spawned: 4,
            powerup_rapid_fire_timer: 5.0,
            powerup_spread_shot_timer: 0.0,
            powerup_pierce_timer: 0.0,
            powerup_has_shield: true,
            narrative,
        };

        let bytes = bincode::serialize(&save).unwrap();
        let loaded: GameSave = bincode::deserialize(&bytes).unwrap();
        assert_eq!(loaded.score, 1234);
        assert_eq!(loaded.lives, 2);
        assert_eq!(loaded.current_wave, 7);
        assert_eq!(loaded.wave_state, WaveState::Spawning);
        assert_eq!(loaded.enemies_spawned, 4);
        assert!(loaded.powerup_has_shield);
    }
}
