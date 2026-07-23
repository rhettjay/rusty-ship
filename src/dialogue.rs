use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use crate::portrait::{Character, PortraitManager};
use crate::menu::{DialogueCallback, DialogueContext};
use std::collections::HashMap;
use std::fs;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DialogueLine {
    pub text: String,
    #[serde(default)]
    pub character: Option<Character>,
    #[serde(default)]
    pub portrait: Option<String>,
    #[serde(default)]
    pub emotion: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DialogueChoice {
    pub text: String,
    pub next_dialogue_id: Option<String>,
    #[serde(default)]
    pub callback: Option<DialogueCallbackType>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum DialogueCallbackType {
    ResumeGame,
    SpawnBoss(String),
    NextChapter,
    GameComplete,
}

impl DialogueCallbackType {
    fn to_callback(&self) -> DialogueCallback {
        match self {
            DialogueCallbackType::ResumeGame => DialogueCallback::ResumeGame,
            DialogueCallbackType::SpawnBoss(name) => {
                if let Ok(boss_type) = name.parse::<crate::boss::BossType>() {
                    DialogueCallback::SpawnBoss(boss_type)
                } else {
                    DialogueCallback::ResumeGame
                }
            }
            DialogueCallbackType::NextChapter => DialogueCallback::NextChapter,
            DialogueCallbackType::GameComplete => DialogueCallback::GameComplete,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DialogueData {
    pub id: String,
    #[serde(default)]
    pub character: Character,
    #[serde(default)]
    pub portrait: String,
    #[serde(default)]
    pub background_music: Option<String>,
    pub lines: Vec<DialogueLine>,
    #[serde(default)]
    pub choices: Option<Vec<DialogueChoice>>,
    #[serde(default)]
    pub on_complete: Option<DialogueCallbackType>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DialogueState {
    Typing { line_idx: usize, char_idx: usize },
    WaitingForInput { line_idx: usize },
    Choosing { line_idx: usize, selected: usize },
    Complete,
}

pub struct DialogueEngine {
    pub current_dialogue: Option<DialogueData>,
    pub state: DialogueState,
    pub typewriter_timer: f64,
    pub typewriter_speed: f32,
    pub skip_typing: bool,
    pub dialogue_cache: HashMap<String, DialogueData>,
}

impl DialogueEngine {
    pub fn new() -> Self {
        Self {
            current_dialogue: None,
            state: DialogueState::Complete,
            typewriter_timer: 0.0,
            typewriter_speed: 30.0,
            skip_typing: false,
            dialogue_cache: HashMap::new(),
        }
    }

    pub fn load_dialogue(&mut self, dialogue_id: &str) -> Result<(), String> {
        if let Some(cached) = self.dialogue_cache.get(dialogue_id) {
            self.current_dialogue = Some(cached.clone());
            self.reset_state();
            return Ok(());
        }

        let path = format!("assets/dialogue/{}.json", dialogue_id);
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read dialogue file {}: {}", path, e))?;

        let data: DialogueData = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse dialogue JSON {}: {}", dialogue_id, e))?;

        self.dialogue_cache.insert(dialogue_id.to_string(), data.clone());
        self.current_dialogue = Some(data);
        self.reset_state();
        Ok(())
    }

    fn reset_state(&mut self) {
        self.state = DialogueState::Typing { line_idx: 0, char_idx: 0 };
        self.typewriter_timer = 0.0;
        self.skip_typing = false;
    }

    pub fn start_dialogue(&mut self, context: &DialogueContext) -> Result<(), String> {
        self.load_dialogue(&context.dialogue_id)?;
        
        if let Some(dialogue) = &self.current_dialogue {
            if let Some(music) = &dialogue.background_music {
                crate::audio::play_music(music);
            }
        }
        Ok(())
    }

    pub fn update(&mut self, dt: f64) -> Option<DialogueCallback> {
        if self.current_dialogue.is_none() {
            return None;
        }

        match self.state {
            DialogueState::Typing { line_idx, char_idx } => {
                if self.skip_typing {
                    self.advance_line();
                    self.skip_typing = false;
                } else {
                    self.typewriter_timer += dt;
                    let chars_per_second = self.typewriter_speed as f64;
                    let target_chars = (self.typewriter_timer * chars_per_second) as usize;
                    
                    if let Some(dialogue) = &self.current_dialogue {
                        if line_idx < dialogue.lines.len() {
                            let line_text = &dialogue.lines[line_idx].text;
                            if target_chars >= line_text.chars().count() {
                                self.state = DialogueState::WaitingForInput { line_idx };
                                crate::audio::play_sfx("dialogue_blip");
                            }
                        }
                    }
                }
            }
            DialogueState::WaitingForInput { line_idx } => {
                if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) {
                    self.advance_line();
                }
            }
            DialogueState::Choosing { line_idx, selected } => {
                if let Some(dialogue) = &self.current_dialogue {
                    if let Some(choices) = &dialogue.choices {
                        let choice_count = choices.len();
                        if is_key_pressed(KeyCode::Up) {
                            let new_selected = if selected > 0 { selected - 1 } else { choice_count - 1 };
                            self.state = DialogueState::Choosing { line_idx, selected: new_selected };
                            crate::audio::play_sfx("dialogue_blip");
                        }
                        if is_key_pressed(KeyCode::Down) {
                            let new_selected = (selected + 1) % choice_count;
                            self.state = DialogueState::Choosing { line_idx, selected: new_selected };
                            crate::audio::play_sfx("dialogue_blip");
                        }
                        if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) {
                            return self.select_choice(selected);
                        }
                    }
                }
            }
            DialogueState::Complete => {
                if let Some(dialogue) = &self.current_dialogue {
                    if let Some(cb_type) = &dialogue.on_complete {
                        return Some(cb_type.to_callback());
                    }
                }
            }
        }
        None
    }

    fn advance_line(&mut self) {
        if let Some(dialogue) = &self.current_dialogue {
            match self.state {
                DialogueState::Typing { line_idx, .. } | DialogueState::WaitingForInput { line_idx } => {
                    let next_idx = line_idx + 1;
                    if next_idx < dialogue.lines.len() {
                        self.state = DialogueState::Typing { line_idx: next_idx, char_idx: 0 };
                        self.typewriter_timer = 0.0;
                    } else if dialogue.choices.is_some() {
                        self.state = DialogueState::Choosing { line_idx: next_idx - 1, selected: 0 };
                    } else {
                        self.state = DialogueState::Complete;
                    }
                }
                _ => {}
            }
        }
    }

    fn select_choice(&mut self, choice_idx: usize) -> Option<DialogueCallback> {
        let (next_id, callback) = {
            if let Some(dialogue) = &self.current_dialogue {
                if let Some(choices) = &dialogue.choices {
                    if choice_idx < choices.len() {
                        let choice = &choices[choice_idx];
                        (choice.next_dialogue_id.clone(), choice.callback.clone())
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            } else {
                return None;
            }
        };

        if let Some(next_id) = next_id {
            if let Err(e) = self.load_dialogue(&next_id) {
                eprintln!("Failed to load choice dialogue: {}", e);
                return None;
            }
            self.reset_state();
            crate::audio::play_sfx("dialogue_blip");
            return None;
        } else if let Some(cb_type) = callback {
            self.reset_state();
            crate::audio::play_sfx("dialogue_blip");
            return Some(cb_type.to_callback());
        }
        None
    }

    pub fn skip(&mut self) {
        self.skip_typing = true;
    }

    pub fn draw(&self, portraits: &PortraitManager) {
        if let Some(dialogue) = &self.current_dialogue {
            self.draw_dialogue_box(dialogue, portraits);
        }
    }

    fn draw_dialogue_box(&self, dialogue: &DialogueData, portraits: &PortraitManager) {
        let box_height = 220.0;
        let box_y = screen_height() - box_height - 20.0;
        let padding = 20.0;
        let portrait_size = 128.0;

        draw_rectangle(0.0, box_y, screen_width(), box_height, Color::new(0.0, 0.0, 0.0, 0.9));
        draw_rectangle_lines(0.0, box_y, screen_width(), box_height, 3.0, GOLD);

        let portrait_x = padding;
        let portrait_y = box_y + (box_height - portrait_size) / 2.0;
        
        let portrait_name = &dialogue.portrait;
        if let Some(portrait) = portraits.get(portrait_name) {
            draw_texture_ex(
                portrait,
                portrait_x,
                portrait_y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(portrait_size, portrait_size)),
                    ..Default::default()
                }
            );
        }

        let text_x = portrait_x + portrait_size + padding;
        let text_max_width = screen_width() - text_x - padding;
        let name_y = box_y + 20.0;
        let text_y = box_y + 55.0;

        draw_text(dialogue.character.name(), text_x, name_y, 26.0, dialogue.character.color());

        let current_line_text = self.get_current_display_text(dialogue);
        self.draw_wrapped_text(&current_line_text, text_x, text_y, text_max_width, 24.0);

        match self.state {
            DialogueState::WaitingForInput { .. } => {
                self.draw_continue_prompt(box_y + box_height - 35.0);
            }
            DialogueState::Choosing { line_idx: _, selected } => {
                if let Some(choices) = &dialogue.choices {
                    let choice_start_y = text_y + 80.0;
                    for (i, choice) in choices.iter().enumerate() {
                        let y = choice_start_y + i as f32 * 35.0;
                        let prefix = if i == selected { "► " } else { "  " };
                        let color = if i == selected { LIME } else { WHITE };
                        draw_text(&format!("{}{}", prefix, choice.text), text_x, y, 22.0, color);
                    }
                }
            }
            _ => {}
        }
    }

    fn get_current_display_text(&self, dialogue: &DialogueData) -> String {
        match self.state {
            DialogueState::Typing { line_idx, char_idx } => {
                if line_idx < dialogue.lines.len() {
                    let line = &dialogue.lines[line_idx];
                    line.text.chars().take(char_idx).collect()
                } else { String::new() }
            }
            DialogueState::WaitingForInput { line_idx } | DialogueState::Choosing { line_idx, .. } => {
                if line_idx < dialogue.lines.len() {
                    dialogue.lines[line_idx].text.clone()
                } else if let Some(last) = dialogue.lines.last() {
                    last.text.clone()
                } else { String::new() }
            }
            DialogueState::Complete => String::new(),
        }
    }

    fn draw_wrapped_text(&self, text: &str, x: f32, y: f32, max_width: f32, font_size: f32) {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut line = String::new();
        let mut current_y = y;
        
        for word in words {
            let test_line = if line.is_empty() { word.to_string() } else { format!("{} {}", line, word) };
            let text_width = measure_text(&test_line, None, font_size as u16, 1.0).width;
            
            if text_width > max_width && !line.is_empty() {
                draw_text(&line, x, current_y, font_size, WHITE);
                current_y += font_size + 4.0;
                line = word.to_string();
            } else {
                line = test_line;
            }
        }
        
        if !line.is_empty() {
            draw_text(&line, x, current_y, font_size, WHITE);
        }
    }

    fn draw_continue_prompt(&self, y: f32) {
        let prompt = "► Press SPACE to continue";
        let x = screen_width() - measure_text(prompt, None, 18, 1.0).width - 20.0;
        let alpha = ((get_time() * 3.0).sin() * 0.5 + 0.5) as f32;
        draw_text(prompt, x, y, 18.0, Color::new(1.0, 1.0, 1.0, alpha));
    }
}