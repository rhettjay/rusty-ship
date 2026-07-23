use macroquad::prelude::*;
use crate::boss::BossType;
use std::collections::HashSet;

#[derive(PartialEq, Clone, Debug)]
pub enum GameState {
    MainMenu,
    Playing,
    GameOver,
    Dialogue(DialogueContext),
    BossIntro(BossType),
    Victory,
}

#[derive(PartialEq, Clone, Debug)]
pub struct DialogueContext {
    pub dialogue_id: String,
    pub on_complete: DialogueCallback,
}

#[derive(Clone, Debug, PartialEq)]
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
}

#[derive(Clone, Debug)]
pub struct NarrativeProgress {
    pub current_chapter: u8,
    pub defeated_bosses: HashSet<BossType>,
    pub flags: HashSet<String>,
    pub score_at_chapter_start: i32,
    pub current_wave: u32,
}

impl Game {
    pub fn new() -> Self {
        Self {
            state: GameState::MainMenu,
            selected_menu_item: 0,
            narrative: NarrativeProgress::new(),
        }
    }
}

impl NarrativeProgress {
    pub fn new() -> Self {
        Self {
            current_chapter: 0,
            defeated_bosses: HashSet::new(),
            flags: HashSet::new(),
            score_at_chapter_start: 0,
            current_wave: 0,
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
    
    let menu_items = ["Start Game", "Settings", "Quit"];
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