use macroquad::prelude::*;
use macroquad::audio::{load_sound_from_bytes, play_sound_once};

mod config;
mod collision;
mod pirate;
mod ship;
mod cannonball;
mod logic;
mod menu;

use ship::*;
use pirate::*;
use cannonball::*;
use logic::*;
use menu::*;
use config::CONFIG;

#[macroquad::main("rusty-ship")]
async fn main() {
    show_mouse(false);
    
    let mut game = Game::new();
    
    let mut cannonball_vec: Vec<Cannonball> = vec![];
    let mut pirate_vec: Vec<Pirate> = vec![];
    let mut pirate_count: i32 = CONFIG.starting_pirate_count;
    let mut game_score: i32 = 0;
    let mut lives: i32 = CONFIG.starting_lives;
    
    let mut ship = Ship {
        x: screen_width() * 0.5 - CONFIG.ship_width * 0.5,
        y: screen_height() - CONFIG.ship_start_y_offset,
        w: CONFIG.ship_width,
        speed: CONFIG.ship_speed,
        color: GRAY,
        gameover: false,
    };

    let background_asset = Texture2D::from_file_with_format(
        include_bytes!("../assets/background/background.png"),
        None,
    );

    let ship_sprite = Texture2D::from_file_with_format(
        include_bytes!("../assets/ship.png"),
        None,
    );

    let pirate_sprite = Texture2D::from_file_with_format(
        include_bytes!("../assets/pirate.png"),
        None,
    );

    let gameover_audio = load_sound_from_bytes(include_bytes!("../assets/gameover.wav")).await.unwrap();
    let cannonball_audio = load_sound_from_bytes(include_bytes!("../assets/cannonball.wav")).await.unwrap();
    let explosion_audio = load_sound_from_bytes(include_bytes!("../assets/explosion.wav")).await.unwrap();

    let mut last_shot_time = get_time();

    loop {
        clear_background(BLACK);
        
        match game.state {
            GameState::MainMenu => {
                draw_menu(&game, &background_asset);
                
                if is_key_pressed(KeyCode::Up) {
                    game.selected_menu_item = (game.selected_menu_item + 2) % 3;
                }
                if is_key_pressed(KeyCode::Down) {
                    game.selected_menu_item = (game.selected_menu_item + 1) % 3;
                }
                if is_key_pressed(KeyCode::Enter) {
                    match game.selected_menu_item {
                        0 => {
                            game.state = GameState::Playing;
                            reset_game(&mut ship, &mut cannonball_vec, &mut pirate_vec, &mut pirate_count, &mut game_score, &mut lives);
                        }
                        1 => {
                            game.state = GameState::GameOver; // placeholder for settings
                        }
                        2 => {
                            std::process::exit(0);
                        }
                        _ => {}
                    }
                }
                if is_key_pressed(KeyCode::Escape) {
                    std::process::exit(0);
                }
            }
            GameState::Playing => {
                if is_key_down(KeyCode::Left) {
                    ship.left();
                }
                if is_key_down(KeyCode::Right) {
                    ship.right();
                }
                if is_key_down(KeyCode::Space) {
                    let now = get_time();
                    if now - last_shot_time >= CONFIG.cannonball_cooldown {
                        cannonball_vec.push(Cannonball::new(
                            ship.x + ship.w * 0.5 - CONFIG.cannonball_width * 0.5,
                            ship.y - CONFIG.cannonball_height,
                            CONFIG.cannonball_speed,
                            WHITE,
                        ));
                        play_sound_once(&cannonball_audio);
                        last_shot_time = now;
                    }
                }
                
                run(
                    &mut ship,
                    &mut cannonball_vec,
                    &mut pirate_vec,
                    &mut pirate_count,
                    &mut game_score,
                    &mut lives,
                    &background_asset,
                    &ship_sprite,
                    &pirate_sprite,
                    &cannonball_audio,
                    &explosion_audio,
                    &explosion_audio,
                    &gameover_audio,
                );
                
                if ship.gameover {
                    game.state = GameState::GameOver;
                }
                
                draw_ui(game_score, lives);
            }
            GameState::GameOver => {
                draw_texture(&background_asset, 0.0, 0.0, WHITE);
                draw_text("GAME OVER", screen_width() * 0.5 - 150.0, screen_height() * 0.4, 100.0, RED);
                draw_text(&format!("Final Score: {}", game_score), screen_width() * 0.5 - 90.0, screen_height() * 0.5, 36.0, YELLOW);
                draw_text("Press SPACE to play again", screen_width() * 0.5 - 150.0, screen_height() * 0.6, 28.0, WHITE);
                draw_text("Press ESC for Main Menu", screen_width() * 0.5 - 140.0, screen_height() * 0.68, 28.0, GRAY);
                
                if is_key_pressed(KeyCode::Space) {
                    reset_game(&mut ship, &mut cannonball_vec, &mut pirate_vec, &mut pirate_count, &mut game_score, &mut lives);
                    game.state = GameState::Playing;
                }
                if is_key_pressed(KeyCode::Escape) {
                    game.state = GameState::MainMenu;
                }
            }
        }
        
        next_frame().await;
    }
}

fn reset_game(
    ship: &mut Ship,
    cannonball_vec: &mut Vec<Cannonball>,
    pirate_vec: &mut Vec<Pirate>,
    pirate_count: &mut i32,
    game_score: &mut i32,
    lives: &mut i32,
) {
    ship.x = screen_width() * 0.5 - CONFIG.ship_width * 0.5;
    ship.y = screen_height() - CONFIG.ship_start_y_offset;
    ship.gameover = false;
    cannonball_vec.clear();
    pirate_vec.clear();
    *pirate_count = CONFIG.starting_pirate_count;
    *game_score = 0;
    *lives = CONFIG.starting_lives;
}

fn draw_ui(score: i32, lives: i32) {
    draw_text(&format!("Score: {}", score), 25.0, 25.0, 25.0, WHITE);
    draw_text(&format!("Lives: {}", lives), 25.0, 55.0, 25.0, WHITE);
}