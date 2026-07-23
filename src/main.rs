use macroquad::prelude::*;
use macroquad::audio::load_sound_from_bytes;

mod config;
mod collision;
mod pirate;
mod ship;
mod cannonball;
mod logic;
mod menu;
mod dialogue;
mod portrait;
mod audio;
mod boss;
mod narrative;

use ship::*;
use pirate::*;
use cannonball::*;
use logic::*;
use menu::*;
use config::CONFIG;
use dialogue::DialogueEngine;
use portrait::PortraitManager;
use audio::{init_audio, play_music, play_sfx, update_audio, set_music_volume, set_sfx_volume};
use boss::{Boss, BossType};
use narrative::{check_triggers, NarrativeTrigger, get_chapter_name};

#[macroquad::main("rusty-ship")]
async fn main() {
    show_mouse(false);
    
    let audio_manager = init_audio().await;
    set_music_volume(CONFIG.music_volume);
    set_sfx_volume(CONFIG.sfx_volume);
    
    let mut portraits = PortraitManager::new();
    portraits.load_all().await;
    
    let mut dialogue_engine = DialogueEngine::new();
    dialogue_engine.typewriter_speed = CONFIG.dialogue_typewriter_speed;
    
    let mut game = Game::new();
    
    let mut cannonball_vec: Vec<Cannonball> = vec![];
    let mut pirate_vec: Vec<Pirate> = vec![];
    let mut pirate_count: i32 = CONFIG.starting_pirate_count;
    let mut game_score: i32 = 0;
    let mut lives: i32 = CONFIG.starting_lives;
    let mut current_wave: u32 = 0;
    
    let mut ship = Ship {
        x: screen_width() * 0.5 - CONFIG.ship_width * 0.5,
        y: screen_height() - CONFIG.ship_start_y_offset,
        w: CONFIG.ship_width,
        speed: CONFIG.ship_speed,
        color: GRAY,
        gameover: false,
    };

    let mut current_boss: Option<Boss> = None;
    let boss_spawn_pending: Option<BossType> = None;
    
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
    let mut last_wave_time = get_time();
    let mut wave_clear_check = false;

    play_music("menu_music");

    loop {
        let dt = get_frame_time() as f64;
        update_audio(dt);
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
                            reset_game(&mut ship, &mut cannonball_vec, &mut pirate_vec, 
                                     &mut pirate_count, &mut game_score, &mut lives,
                                     &mut current_wave, &mut current_boss);
                            play_music("gameplay_music");
                        }
                        1 => {
                            game.state = GameState::GameOver;
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
                        play_sfx("cannon_fire");
                        last_shot_time = now;
                    }
                }
                
                if let Some(boss) = &mut current_boss {
                    if boss.entry_anim {
                        boss.update(dt, ship.x, ship.y);
                    } else {
                        let result = run_with_boss(
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
                            boss,
                            dt,
                        );
                        
                        if boss.is_dead {
                            let boss_type = boss.boss_type;
                            current_boss = None;
                            game.narrative.defeated_bosses.insert(boss_type);
                            game.narrative.current_chapter += 1;
                            
                            if let Some(dialogue) = check_triggers(&mut game.narrative, NarrativeTrigger::BossDefeated(boss_type)) {
                                game.state = GameState::Dialogue(dialogue.clone());
                                dialogue_engine.start_dialogue(&dialogue).ok();
                            }
                            
                            if game.narrative.defeated_bosses.len() >= 5 {
                                if let Some(dialogue) = check_triggers(&mut game.narrative, NarrativeTrigger::AllBossesDefeated) {
                                    game.state = GameState::Dialogue(dialogue.clone());
                                    dialogue_engine.start_dialogue(&dialogue).ok();
                                }
                            }
                        }
                    }
                } else {
                    let wave_result = run(
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
                        play_music("menu_music");
                    }
                    
                    let now = get_time();
                    if now - last_wave_time > 30.0 && pirate_vec.is_empty() && pirate_count > 0 {
                        current_wave += 1;
                        last_wave_time = now;
                        
                        if let Some(dialogue) = check_triggers(&mut game.narrative, NarrativeTrigger::WaveCleared(current_wave)) {
                            game.state = GameState::Dialogue(dialogue.clone());
                            dialogue_engine.start_dialogue(&dialogue).ok();
                        }
                        
                        if game_score >= CONFIG.bonus_score_threshold {
                            if let Some(dialogue) = check_triggers(&mut game.narrative, NarrativeTrigger::ScoreThreshold(game_score)) {
                                game.state = GameState::Dialogue(dialogue.clone());
                                dialogue_engine.start_dialogue(&dialogue).ok();
                            }
                        }
                    }
                }
                
                draw_ui(game_score, lives, current_wave, &game.narrative);
                
                if current_boss.is_some() {
                    draw_boss_health_bar(current_boss.as_ref().unwrap());
                }
            }
            GameState::Dialogue(ref context) => {
                draw_texture(&background_asset, 0.0, 0.0, WHITE);
                
                for cannonball in &cannonball_vec {
                    cannonball.draw();
                }
                for pirate in &pirate_vec {
                    pirate.draw(&pirate_sprite);
                }
                if let Some(boss) = &current_boss {
                    boss.draw();
                }
                ship.draw(&ship_sprite);
                
                if let Some(callback) = dialogue_engine.update(dt) {
                    match callback {
                        DialogueCallback::ResumeGame => game.state = GameState::Playing,
                        DialogueCallback::SpawnBoss(boss_type) => {
                            game.state = GameState::BossIntro(boss_type);
                        }
                        DialogueCallback::NextChapter => {
                            game.state = GameState::Playing;
                            game.narrative.current_chapter += 1;
                        }
                        DialogueCallback::GameComplete => {
                            game.state = GameState::Victory;
                            play_music("victory_music");
                        }
                    }
                }
                
                dialogue_engine.draw(&portraits);
            }
            GameState::BossIntro(boss_type) => {
                draw_texture(&background_asset, 0.0, 0.0, WHITE);
                
                let title = format!("BOSS APPROACHING: {}", boss_type.name());
                let title_size = 60.0;
                let title_x = screen_width() * 0.5 - measure_text(&title, None, title_size as u16, 1.0).width * 0.5;
                draw_text(&title, title_x, screen_height() * 0.3, title_size, RED);
                
                let subtitle = "PRESS SPACE TO ENGAGE";
                let sub_size = 30.0;
                let sub_x = screen_width() * 0.5 - measure_text(subtitle, None, sub_size as u16, 1.0).width * 0.5;
                draw_text(subtitle, sub_x, screen_height() * 0.5, sub_size, YELLOW);
                
                if let Some(portrait) = portraits.get_character(&match boss_type {
                    BossType::Blowfish => portrait::Character::Blowfish,
                    BossType::Twofish => portrait::Character::Twofish,
                    BossType::RufusReverse => portrait::Character::RufusReverse,
                    BossType::MollyHashpass => portrait::Character::MollyHashpass,
                    BossType::CaptainDavey => portrait::Character::CaptainDavey,
                    BossType::Deadbeef => portrait::Character::Deadbeef,
                }) {
                    let size = 200.0;
                    draw_texture_ex(portrait, screen_width() * 0.5 - size * 0.5, screen_height() * 0.55, WHITE,
                        DrawTextureParams { dest_size: Some(Vec2::new(size, size)), ..Default::default() });
                }
                
                if is_key_pressed(KeyCode::Space) {
                    let mut boss = Boss::new(boss_type);
                    boss.load_assets().await;
                    current_boss = Some(boss);
                    game.state = GameState::Playing;
                    play_music("boss_music");
                }
            }
            GameState::GameOver => {
                draw_texture(&background_asset, 0.0, 0.0, WHITE);
                draw_text("GAME OVER", screen_width() * 0.5 - 150.0, screen_height() * 0.4, 100.0, RED);
                draw_text(&format!("Final Score: {}", game_score), screen_width() * 0.5 - 90.0, screen_height() * 0.5, 36.0, YELLOW);
                draw_text("Press SPACE to play again", screen_width() * 0.5 - 150.0, screen_height() * 0.6, 28.0, WHITE);
                draw_text("Press ESC for Main Menu", screen_width() * 0.5 - 140.0, screen_height() * 0.68, 28.0, GRAY);
                
                if is_key_pressed(KeyCode::Space) {
                    reset_game(&mut ship, &mut cannonball_vec, &mut pirate_vec, 
                             &mut pirate_count, &mut game_score, &mut lives,
                             &mut current_wave, &mut current_boss);
                    game.state = GameState::Playing;
                    play_music("gameplay_music");
                }
                if is_key_pressed(KeyCode::Escape) {
                    game.state = GameState::MainMenu;
                    play_music("menu_music");
                }
            }
            GameState::Victory => {
                draw_texture(&background_asset, 0.0, 0.0, WHITE);
                draw_text("VICTORY!", screen_width() * 0.5 - 120.0, screen_height() * 0.3, 100.0, GOLD);
                draw_text(&format!("Final Score: {}", game_score), screen_width() * 0.5 - 90.0, screen_height() * 0.45, 36.0, YELLOW);
                draw_text("You defeated Captain Davey Portscan!", screen_width() * 0.5 - 200.0, screen_height() * 0.55, 28.0, WHITE);
                draw_text("Press SPACE to play again", screen_width() * 0.5 - 150.0, screen_height() * 0.65, 28.0, WHITE);
                draw_text("Press ESC for Main Menu", screen_width() * 0.5 - 140.0, screen_height() * 0.72, 28.0, GRAY);
                
                if is_key_pressed(KeyCode::Space) {
                    reset_game(&mut ship, &mut cannonball_vec, &mut pirate_vec, 
                             &mut pirate_count, &mut game_score, &mut lives,
                             &mut current_wave, &mut current_boss);
                    game.state = GameState::Playing;
                    play_music("gameplay_music");
                }
                if is_key_pressed(KeyCode::Escape) {
                    game.state = GameState::MainMenu;
                    play_music("menu_music");
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
    current_wave: &mut u32,
    current_boss: &mut Option<Boss>,
) {
    ship.x = screen_width() * 0.5 - CONFIG.ship_width * 0.5;
    ship.y = screen_height() - CONFIG.ship_start_y_offset;
    ship.gameover = false;
    cannonball_vec.clear();
    pirate_vec.clear();
    *pirate_count = CONFIG.starting_pirate_count;
    *game_score = 0;
    *lives = CONFIG.starting_lives;
    *current_wave = 0;
    *current_boss = None;
}

fn draw_ui(score: i32, lives: i32, wave: u32, narrative: &NarrativeProgress) {
    draw_text(&format!("Score: {}", score), 25.0, 25.0, 25.0, WHITE);
    draw_text(&format!("Lives: {}", lives), 25.0, 55.0, 25.0, WHITE);
    draw_text(&format!("Wave: {}", wave), 25.0, 85.0, 25.0, WHITE);
    
    if narrative.current_chapter > 0 {
        draw_text(
            get_chapter_name(narrative.current_chapter),
            25.0, 115.0, 20.0, GOLD
        );
    }
}

fn draw_boss_health_bar(boss: &Boss) {
    let bar_width = 400.0;
    let bar_height = 20.0;
    let x = screen_width() * 0.5 - bar_width * 0.5;
    let y = 20.0;
    
    draw_rectangle(x, y, bar_width, bar_height, Color::new(0.2, 0.0, 0.0, 0.8));
    let health_pct = boss.health as f32 / boss.max_health as f32;
    draw_rectangle(x, y, bar_width * health_pct, bar_height, RED);
    draw_rectangle_lines(x, y, bar_width, bar_height, 2.0, WHITE);
    
    let name = boss.boss_type.name();
    let name_x = screen_width() * 0.5 - measure_text(name, None, 24, 1.0).width * 0.5;
    draw_text(name, name_x, y - 5.0, 24.0, WHITE);
    
    let phase_text = format!("Phase {:?}", boss.phase);
    let phase_x = screen_width() * 0.5 - measure_text(&phase_text, None, 18, 1.0).width * 0.5;
    draw_text(&phase_text, phase_x, y + bar_height + 20.0, 18.0, YELLOW);
}