use macroquad::prelude::*;
use crate::boss::BossType;
use serde::{Serialize, Deserialize};

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum GameState {
    MainMenu,
    Playing,
    Paused { selected_item: usize },
    Console { input: String, history: Vec<String>, cursor_pos: usize },
    GameOver,
    Dialogue(DialogueContext),
    BossIntro(BossType),
    Victory,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct DialogueContext {
    pub dialogue_id: String,
    pub on_complete: DialogueCallback,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DialogueCallback {
    ResumeGame,
    SpawnBoss(BossType),
    NextChapter,
    GameComplete,
}

pub struct Game {
    pub state: GameState,
    pub selected_menu_item: usize,
    pub narrative: NarrativeProgress,
    pub debug_hitbox_visible: bool,
    pub time_scale: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NarrativeProgress {
    pub current_chapter: u8,
    pub defeated_bosses: Vec<String>,
    pub flags: Vec<String>,
    pub score_at_chapter_start: i32,
    pub current_wave: u32,
}

impl NarrativeProgress {
    pub fn new() -> Self {
        Self {
            current_chapter: 0,
            defeated_bosses: Vec::new(),
            flags: Vec::new(),
            score_at_chapter_start: 0,
            current_wave: 0,
        }
    }
}

impl Game {
    pub fn new() -> Self {
        Self {
            state: GameState::MainMenu,
            selected_menu_item: 0,
            narrative: NarrativeProgress::new(),
            debug_hitbox_visible: false,
            time_scale: 1.0,
        }
    }
}

pub fn draw_menu(game: &Game, background: &Texture2D) {
    draw_texture(background, 0.0, 0.0, WHITE);
    
    let title = "RUSTY SHIP";
    let title_size = 80.0;
    let title_x = screen_width() * 0.5 - measure_text(title, None, title_size as u16, 1.0).width * 0.5;
    draw_text(title, title_x, screen_height() * 0.25, title_size, GOLD);
    
    let subtitle = "Hacker-Pirate Space Combat";
    let sub_size = 24.0;
    let sub_x = screen_width() * 0.5 - measure_text(subtitle, None, sub_size as u16, 1.0).width * 0.5;
    draw_text(subtitle, sub_x, screen_height() * 0.35, sub_size, LIGHTGRAY);
    
    let has_save = crate::save::has_save_file();
    let menu_items = if has_save {
        vec!["Start Game", "Load Game", "Settings", "Quit"]
    } else {
        vec!["Start Game", "Settings", "Quit"]
    };
    let item_size = 36.0;
    let start_y = screen_height() * 0.5;
    
    for (i, item) in menu_items.iter().enumerate() {
        let color = if i == game.selected_menu_item { LIME } else { WHITE };
        let x = screen_width() * 0.5 - measure_text(item, None, item_size as u16, 1.0).width * 0.5;
        let y = start_y + i as f32 * 60.0;
        
        if i == game.selected_menu_item {
            draw_text("► ", x - 30.0, y, item_size, LIME);
        }
        draw_text(item, x, y, item_size, color);
    }
    
    let controls = "UP/DOWN: Navigate  |  ENTER: Select  |  ESC: Quit";
    let ctrl_size = 18.0;
    let ctrl_x = screen_width() * 0.5 - measure_text(controls, None, ctrl_size as u16, 1.0).width * 0.5;
    draw_text(controls, ctrl_x, screen_height() * 0.85, ctrl_size, GRAY);
}

pub fn draw_settings() {
    draw_text("SETTINGS (Coming Soon)", screen_width() * 0.5 - 120.0, screen_height() * 0.5, 36.0, WHITE);
    draw_text("Press ESC to return", screen_width() * 0.5 - 100.0, screen_height() * 0.5 + 50.0, 24.0, GRAY);
}

pub fn draw_pause_menu(selected_item: usize) {
    let overlay_color = Color::new(0.0, 0.0, 0.0, 0.7);
    draw_rectangle(0.0, 0.0, screen_width(), screen_height(), overlay_color);
    
    let title = "PAUSED";
    let title_size = 60.0;
    let title_x = screen_width() * 0.5 - measure_text(title, None, title_size as u16, 1.0).width * 0.5;
    draw_text(title, title_x, screen_height() * 0.3, title_size, GOLD);
    
    let menu_items = ["Resume", "Save Game", "Console", "Quit to Menu", "Quit Game"];
    let item_size = 32.0;
    let start_y = screen_height() * 0.45;
    
    for (i, item) in menu_items.iter().enumerate() {
        let color = if i == selected_item { LIME } else { WHITE };
        let x = screen_width() * 0.5 - measure_text(item, None, item_size as u16, 1.0).width * 0.5;
        let y = start_y + i as f32 * 55.0;
        
        if i == selected_item {
            draw_text("► ", x - 30.0, y, item_size, LIME);
        }
        draw_text(item, x, y, item_size, color);
    }
    
    let controls = "UP/DOWN: Navigate  |  ENTER: Select  |  ESC: Resume  |  /: Console";
    let ctrl_size = 18.0;
    let ctrl_x = screen_width() * 0.5 - measure_text(controls, None, ctrl_size as u16, 1.0).width * 0.5;
    draw_text(controls, ctrl_x, screen_height() * 0.85, ctrl_size, GRAY);
}

pub(crate) fn char_to_byte(input: &str, char_idx: usize) -> usize {
    input.char_indices().nth(char_idx).map(|(b, _)| b).unwrap_or(input.len())
}

pub fn draw_console(input: &str, history: &[String], cursor_pos: usize) {
    let overlay_color = Color::new(0.0, 0.0, 0.0, 0.85);
    draw_rectangle(0.0, 0.0, screen_width(), screen_height(), overlay_color);
    
    let title = "DEVELOPER CONSOLE (/ to close)";
    let title_size = 28.0;
    let title_x = 20.0;
    draw_text(title, title_x, 40.0, title_size, GOLD);
    
    // Draw history (last 20 lines)
    let history_start = history.len().saturating_sub(20);
    let mut y = 80.0;
    for line in &history[history_start..] {
        draw_text(line, 20.0, y, 18.0, LIGHTGRAY);
        y += 22.0;
    }
    
    // Draw input line
    let prompt = "> ";
    let input_x = 20.0;
    let input_y = screen_height() - 60.0;
    draw_text(prompt, input_x, input_y, 22.0, WHITE);
    
    let text_len = input.chars().count();
    let input_text = &input[..char_to_byte(input, cursor_pos.min(text_len))];
    let cursor_x = input_x + measure_text(prompt, None, 22, 1.0).width + measure_text(input_text, None, 22, 1.0).width;
    
    draw_text(input, input_x + measure_text(prompt, None, 22, 1.0).width, input_y, 22.0, WHITE);
    
    // Blinking cursor
    if (get_time() * 2.0).sin() > 0.0 {
        draw_text("_", cursor_x, input_y, 22.0, WHITE);
    }
    
    let help = "Commands: help, god, heal, wave <n>, score <n>, lives <n>, spawn <enemy>, killall, fps, quit";
    draw_text(help, 20.0, screen_height() - 30.0, 16.0, GRAY);
}

#[cfg(test)]
mod tests {
    use super::char_to_byte;

    #[test]
    fn test_char_to_byte_maps_char_indices() {
        let s = "r\u{e9}load"; // 'é' is 2 bytes in UTF-8
        assert_eq!(s.chars().count(), 6);
        assert_eq!(char_to_byte(s, 0), 0);
        assert_eq!(char_to_byte(s, 1), 1); // start of 'é'
        assert_eq!(char_to_byte(s, 2), 3); // past 'é'
        assert_eq!(char_to_byte(s, 5), 6); // start of last char 'd'
        assert_eq!(char_to_byte(s, 6), 7); // out of range -> len
        assert_eq!(char_to_byte(s, 99), 7);
    }

    #[test]
    fn test_insert_remove_at_char_boundary() {
        let mut input = String::from("r\u{e9}load");
        let byte_idx = char_to_byte(&input, 6);
        input.insert(byte_idx, '!');
        assert_eq!(input, "r\u{e9}load!");

        let byte_idx = char_to_byte(&input, 6);
        input.remove(byte_idx);
        assert_eq!(input, "r\u{e9}load");

        let byte_idx = char_to_byte(&input, 5);
        input.remove(byte_idx);
        assert_eq!(input, "r\u{e9}loa");
    }

    #[test]
    fn test_cursor_slice_is_char_safe() {
        let s = "r\u{e9}load";
        let n = s.chars().count();
        for pos in 0..=n {
            let text = &s[..char_to_byte(s, pos)];
            assert_eq!(text.chars().count(), pos);
        }
    }
}