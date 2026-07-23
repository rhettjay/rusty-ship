use macroquad::audio::{load_sound_from_bytes, play_sound_once, Sound};
use macroquad::prelude::*;
use std::collections::HashMap;
use std::sync::{OnceLock, Mutex};

struct MusicTrack {
    name: String,
    sound: Option<Sound>,
    playing: bool,
}

pub struct AudioManager {
    music_tracks: HashMap<String, MusicTrack>,
    sfx: HashMap<String, Sound>,
    current_music: Option<String>,
    fade_target: Option<String>,
    fade_timer: f64,
    fade_duration: f64,
    music_volume: f32,
    sfx_volume: f32,
}

impl AudioManager {
    pub fn new() -> Self {
        Self {
            music_tracks: HashMap::new(),
            sfx: HashMap::new(),
            current_music: None,
            fade_target: None,
            fade_timer: 0.0,
            fade_duration: 1.5,
            music_volume: 0.5,
            sfx_volume: 0.7,
        }
    }

    pub async fn load_all(&mut self) {
        let music_files = [
            ("menu_music", "assets/audio/music/menu_music.ogg"),
            ("gameplay_music", "assets/audio/music/gameplay_music.ogg"),
            ("boss_music", "assets/audio/music/boss_music.ogg"),
            ("victory_music", "assets/audio/music/victory_music.ogg"),
        ];

        for (name, path) in music_files {
            if let Ok(bytes) = std::fs::read(path) {
                if !bytes.is_empty() {
                    if let Ok(sound) = load_sound_from_bytes(&bytes).await {
                        self.music_tracks.insert(name.to_string(), MusicTrack {
                            name: name.to_string(),
                            sound: Some(sound),
                            playing: false,
                        });
                    }
                }
            }
        }

        let sfx_files = [
            ("yaharr", "assets/audio/sfx/yaharr.ogg"),
            ("cannon_fire", "assets/audio/sfx/cannon_fire.ogg"),
            ("hit", "assets/audio/sfx/hit.ogg"),
            ("boss_laugh", "assets/audio/sfx/boss_laugh.ogg"),
            ("dialogue_blip", "assets/audio/sfx/dialogue_blip.ogg"),
            ("boss_defeat", "assets/audio/sfx/boss_defeat.ogg"),
            ("explosion", "assets/audio/sfx/explosion.ogg"),
            ("gameover", "assets/audio/sfx/gameover.ogg"),
        ];

        for (name, path) in sfx_files {
            if let Ok(bytes) = std::fs::read(path) {
                if !bytes.is_empty() {
                    if let Ok(sound) = load_sound_from_bytes(&bytes).await {
                        self.sfx.insert(name.to_string(), sound);
                    }
                }
            }
        }
    }

    pub fn play_music(&mut self, name: &str) {
        if self.current_music == Some(name.to_string()) {
            return;
        }
        self.fade_target = Some(name.to_string());
        self.fade_timer = 0.0;
    }

    pub fn play_music_immediate(&mut self, name: &str) {
        if let Some(track) = self.music_tracks.get_mut(name) {
            if let Some(sound) = &track.sound {
                play_sound_once(sound);
                track.playing = true;
            }
        }
        if let Some(old) = &self.current_music {
            if old != name {
                if let Some(old_track) = self.music_tracks.get_mut(old) {
                    old_track.playing = false;
                }
            }
        }
        self.current_music = Some(name.to_string());
        self.fade_target = None;
    }

    pub fn stop_music(&mut self) {
        if let Some(current) = &self.current_music {
            if let Some(track) = self.music_tracks.get_mut(current) {
                track.playing = false;
            }
        }
        self.current_music = None;
        self.fade_target = None;
    }

    pub fn play_sfx(&self, name: &str) {
        if let Some(sound) = self.sfx.get(name) {
            play_sound_once(sound);
        }
    }

    pub fn set_music_volume(&mut self, volume: f32) {
        self.music_volume = volume.clamp(0.0, 1.0);
    }

    pub fn set_sfx_volume(&mut self, volume: f32) {
        self.sfx_volume = volume.clamp(0.0, 1.0);
    }

    pub fn update(&mut self, dt: f64) {
        if let Some(target) = self.fade_target.clone() {
            self.fade_timer += dt;
            let progress = (self.fade_timer / self.fade_duration).min(1.0);
            
            if progress >= 1.0 {
                self.play_music_immediate(&target);
            }
        }
    }
}

static AUDIO_MANAGER: OnceLock<Mutex<AudioManager>> = OnceLock::new();

pub async fn init_audio() -> &'static Mutex<AudioManager> {
    let mut manager = AudioManager::new();
    manager.load_all().await;
    AUDIO_MANAGER.get_or_init(|| Mutex::new(manager))
}

pub fn play_music(name: &str) {
    if let Some(manager) = AUDIO_MANAGER.get() {
        if let Ok(mut m) = manager.lock() {
            m.play_music(name);
        }
    }
}

pub fn play_sfx(name: &str) {
    if let Some(manager) = AUDIO_MANAGER.get() {
        if let Ok(m) = manager.lock() {
            m.play_sfx(name);
        }
    }
}

pub fn set_music_volume(volume: f32) {
    if let Some(manager) = AUDIO_MANAGER.get() {
        if let Ok(mut m) = manager.lock() {
            m.set_music_volume(volume);
        }
    }
}

pub fn set_sfx_volume(volume: f32) {
    if let Some(manager) = AUDIO_MANAGER.get() {
        if let Ok(mut m) = manager.lock() {
            m.set_sfx_volume(volume);
        }
    }
}

pub fn update_audio(dt: f64) {
    if let Some(manager) = AUDIO_MANAGER.get() {
        if let Ok(mut m) = manager.lock() {
            m.update(dt);
        }
    }
}