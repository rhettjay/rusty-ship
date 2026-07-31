use macroquad::prelude::*;

mod config;
mod collision;
mod content;
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
mod enemy;
mod bullet;
mod powerup;
mod formation;
mod wave_director;
mod save;

use ship::*;
use pirate::*;
use cannonball::*;
use menu::*;
use config::CONFIG;
use dialogue::DialogueEngine;
use portrait::PortraitManager;
use audio::{init_audio, play_music, play_sfx, update_audio, set_music_volume, set_sfx_volume};
use boss::{Boss, BossType};
use narrative::{check_triggers, NarrativeTrigger, get_chapter_name};
use enemy::Enemy;
use bullet::Bullet;
use wave_director::{WaveDirector, WaveState};
use save::{save_game, load_game, apply_save};
use std::collections::HashMap;

#[macroquad::main("rusty-ship")]
async fn main() {
    show_mouse(false);
    
    content::load();
    if let Err(issues) = content::validate() {
        eprintln!("[content] validation issues:\n{issues}");
    }
    
    let _audio_manager = init_audio().await;
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
    
    let mut ship = Ship {
        x: screen_width() * 0.5 - CONFIG.ship_width * 0.5,
        y: screen_height() - CONFIG.ship_start_y_offset,
        w: CONFIG.ship_width,
        h: CONFIG.ship_height,
        speed: CONFIG.ship_speed,
        color: GRAY,
        gameover: false,
        has_shield: false,
        rapid_fire_timer: 0.0,
        spread_shot_timer: 0.0,
        pierce_timer: 0.0,
        invuln_timer: 0.0,
    };

    let mut enemy_vec: Vec<Enemy> = vec![];
    let mut bullet_vec: Vec<Bullet> = vec![];
    
    let mut wave_director = WaveDirector::new();
    
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

    let enemy_sprites: HashMap<String, Texture2D> = HashMap::new();
    let bullet_sprites: HashMap<String, Texture2D> = HashMap::new();

    let mut last_shot_time = get_time();

    play_music("menu_music");

    loop {
            let dt = (get_frame_time() as f64) * game.time_scale as f64;
        update_audio(dt);
        clear_background(BLACK);
        
        match game.state {
            GameState::MainMenu => {
                draw_menu(&game, &background_asset);
                
                let has_save = crate::save::has_save_file();
                let menu_items: &[&str] = if has_save {
                    &["Start Game", "Load Game", "Settings", "Quit"]
                } else {
                    &["Start Game", "Settings", "Quit"]
                };
                let item_count = menu_items.len();

                if is_key_pressed(KeyCode::Up) {
                    game.selected_menu_item = (game.selected_menu_item + item_count - 1) % item_count;
                }
                if is_key_pressed(KeyCode::Down) {
                    game.selected_menu_item = (game.selected_menu_item + 1) % item_count;
                }
                if is_key_pressed(KeyCode::Enter) {
                    match menu_items[game.selected_menu_item] {
                        "Start Game" => {
                            game.state = GameState::Playing;
                            reset_game(&mut ship, &mut cannonball_vec, &mut pirate_vec, 
                                     &mut enemy_vec, &mut bullet_vec, &mut pirate_count, 
                                     &mut game_score, &mut lives);
                            wave_director.start_wave(1);
                            play_music("gameplay_music");
                            show_mouse(false);
                        }
                        "Load Game" => {
                            match load_game() {
                                Ok(save) => {
                                    apply_save(save, &mut ship, &mut wave_director, &mut game, &mut game_score, &mut lives);
                                    enemy_vec.clear();
                                    bullet_vec.clear();
                                    cannonball_vec.clear();
                                    pirate_vec.clear();
                                    game.state = GameState::Playing;
                                    play_music("gameplay_music");
                                    show_mouse(false);
                                }
                                Err(e) => {
                                    eprintln!("Load failed: {}", e);
                                }
                            }
                        }
                        "Settings" => {
                        }
                        "Quit" => {
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
                // Pause handling
                if is_key_pressed(KeyCode::Escape) {
                    game.state = GameState::Paused { selected_item: 0 };
                    show_mouse(true);
                }
                
                // Console handling (forward slash key)
                if is_key_pressed(KeyCode::Slash) {
                    game.state = GameState::Console {
                        input: String::new(),
                        history: Vec::new(),
                        cursor_pos: 0,
                    };
                    show_mouse(true);
                }
                
                if is_key_down(KeyCode::Left) {
                    ship.left();
                }
                if is_key_down(KeyCode::Right) {
                    ship.right();
                }
                if is_key_down(KeyCode::Space) {
                    let now = get_time();
                    let cooldown = wave_director.get_cannonball_cooldown(CONFIG.cannonball_cooldown);
                    if now - last_shot_time >= cooldown {
                        let bullet_type = if wave_director.is_spread_shot_active() {
                            crate::bullet::BulletType::PlayerSpread
                        } else if wave_director.is_pierce_active() {
                            crate::bullet::BulletType::PlayerPierce
                        } else {
                            crate::bullet::BulletType::PlayerStandard
                        };
                        
                        let bullet_speed = bullet_type.speed();
                        
                        let bullets = if bullet_type == crate::bullet::BulletType::PlayerSpread {
                            crate::bullet::create_spread_bullets(
                                bullet_type,
                                ship.x + ship.w * 0.5,
                                ship.y - CONFIG.cannonball_height,
                                0.0,
                                -bullet_speed,
                                3,
                                std::f32::consts::FRAC_PI_4,
                            )
                        } else {
                            vec![Bullet::new(
                                bullet_type,
                                ship.x + ship.w * 0.5 - CONFIG.cannonball_width * 0.5,
                                ship.y - CONFIG.cannonball_height,
                                0.0,
                                -bullet_speed,
                            )]
                        };
                        
                        for b in bullets {
                            bullet_vec.push(b);
                        }
                        
                        play_sfx("cannon_fire");
                        last_shot_time = now;
                    }
                }
                
                let prev_wave_state = wave_director.wave_state;
                
                wave_director.update(
                    dt, 
                    &mut enemy_vec,
                    &mut bullet_vec,
                    &mut ship, 
                    &mut cannonball_vec, 
                    &mut pirate_vec, 
                    &mut game_score, 
                    &mut lives
                );
                
                // Auto-save on wave completion or boss defeated
                let prev_wave_complete = matches!(prev_wave_state, WaveState::Complete | WaveState::BossDefeated);
                let wave_complete = wave_director.is_wave_complete();
                
                if wave_complete && !prev_wave_complete {
                    let save_result = save_game(
                        game_score,
                        lives,
                        wave_director.current_wave,
                        wave_director.wave_state,
                        ship.x,
                        ship.y,
                        ship.has_shield,
                        ship.rapid_fire_timer,
                        ship.spread_shot_timer,
                        ship.pierce_timer,
                        game.narrative.clone(),
                        wave_director.wave_timer,
                        wave_director.enemies_spawned,
                        wave_director.powerup_manager.rapid_fire_timer,
                        wave_director.powerup_manager.spread_shot_timer,
                        wave_director.powerup_manager.pierce_timer,
                        wave_director.powerup_manager.has_shield,
                    );
                    if let Err(e) = save_result {
                        eprintln!("Auto-save failed: {}", e);
                    }
                    
                    if wave_director.is_boss_wave {
                        let boss_type = boss_for_wave(wave_director.current_wave);
                        if let Some(dialogue) = check_triggers(&mut game.narrative, NarrativeTrigger::BossDefeated(boss_type)) {
                            game.state = GameState::Dialogue(dialogue.clone());
                            dialogue_engine.start_dialogue(&dialogue).ok();
                        }
                    } else {
                        wave_director.start_wave(wave_director.current_wave + 1);
                        if wave_director.is_boss_wave {
                            game.state = GameState::BossIntro(boss_for_wave(wave_director.current_wave));
                        }
                    }
                }
                
                wave_director.check_powerup_pickup(
                    &mut ship, 
                    &mut cannonball_vec, 
                    &mut pirate_vec, 
                    &mut game_score, 
                    &mut lives
                );
                
                for enemy in &mut enemy_vec {
                    enemy.update(dt, ship.x, ship.y);
                    
                    if enemy.can_shoot() {
                        enemy.reset_shoot_timer();
                        let (vel_x, vel_y) = enemy.get_shoot_direction(ship.x, ship.y);
                        let bullet_type = match enemy.shoot_pattern {
                            crate::enemy::ShootPattern::Straight => crate::bullet::BulletType::EnemyStraight,
                            crate::enemy::ShootPattern::Aimed => crate::bullet::BulletType::EnemyAimed,
                            crate::enemy::ShootPattern::Bomb => crate::bullet::BulletType::EnemyBomb,
                            crate::enemy::ShootPattern::Spread => crate::bullet::BulletType::EnemySpread,
                            crate::enemy::ShootPattern::None => continue,
                        };
                        let bullet = Bullet::new(
                            bullet_type,
                            enemy.x,
                            enemy.y + enemy.height / 2.0,
                            vel_x * bullet_type.speed(),
                            vel_y * bullet_type.speed(),
                        );
                        bullet_vec.push(bullet);
                    }
                }
                
                if ship.invuln_timer > 0.0 {
                    ship.invuln_timer -= dt as f32;
                }

                for bullet in &mut bullet_vec {
                    bullet.update(dt);
                }
                bullet_vec.retain(|b| !b.is_dead);
                
                for cannonball in &mut cannonball_vec {
                    cannonball.update();
                }
                cannonball_vec.retain(|c| c.y > 0.0);
                
                check_collisions(
                    &mut enemy_vec,
                    &mut bullet_vec,
                    &mut cannonball_vec,
                    &mut ship,
                    &mut pirate_vec,
                    &mut pirate_count,
                    &mut game_score,
                    &mut lives,
                    &mut wave_director,
                );
                
                if ship.gameover {
                    game.state = GameState::GameOver;
                    play_music("menu_music");
                }
                
                if game_score >= CONFIG.bonus_score_threshold && !wave_director.is_boss_active() && wave_complete && !prev_wave_complete {
                    if let Some(dialogue) = check_triggers(&mut game.narrative, NarrativeTrigger::ScoreThreshold(game_score)) {
                        game.state = GameState::Dialogue(dialogue.clone());
                        dialogue_engine.start_dialogue(&dialogue).ok();
                    }
                }
                
                // RENDERING
                draw_texture(&background_asset, 0.0, 0.0, WHITE);
                
                // Draw cannonballs
                for cannonball in &cannonball_vec {
                    cannonball.draw();
                }
                
                // Draw pirates
                for pirate in &pirate_vec {
                    pirate.draw(&pirate_sprite);
                }
                
                // Draw enemies
                for enemy in &enemy_vec {
                    if let Some(tex) = enemy_sprites.get(enemy.enemy_type.sprite_name()) {
                        enemy.draw(Some(tex));
                    } else {
                        enemy.draw(None);
                    }
                }
                
                // Draw bullets
                for bullet in &bullet_vec {
                    if let Some(tex) = bullet_sprites.get(bullet.bullet_type.sprite_name()) {
                        bullet.draw(Some(tex));
                    } else {
                        bullet.draw(None);
                    }
                }
                
                // Draw powerups
                let powerup_textures = std::collections::HashMap::new(); // empty for now
                wave_director.powerup_manager.draw(&powerup_textures);
                
                // Draw boss if active
                if wave_director.is_boss_active() {
                    if let Some(boss) = wave_director.get_current_boss() {
                        boss.draw();
                    }
                }
                
                // Draw ship
                ship.draw(&ship_sprite);
                
                if game.debug_hitbox_visible {
                    for enemy in &enemy_vec {
                        if !enemy.is_dead {
                            let (x, y, w, h) = enemy.get_rect();
                            draw_rectangle_lines(x, y, w, h, 1.0, RED);
                        }
                    }
                    for bullet in &bullet_vec {
                        if !bullet.is_dead {
                            let (x, y, w, h) = bullet.get_rect();
                            draw_rectangle_lines(x, y, w, h, 1.0, YELLOW);
                        }
                    }
                    let (sx, sy, sw, sh) = ship.get_rect();
                    draw_rectangle_lines(sx, sy, sw, sh, 1.0, GREEN);
                    if let Some(boss) = wave_director.get_current_boss() {
                        let (bx, by, bw, bh) = boss.get_rect();
                        draw_rectangle_lines(bx, by, bw, bh, 1.0, RED);
                        for p in boss.get_projectiles() {
                            draw_circle_lines(p.x, p.y, p.size / 2.0, 1.0, RED);
                        }
                    }
                }
                
                // Draw UI
                draw_ui(game_score, lives, wave_director.current_wave, &game.narrative);
                wave_director.draw_wave_info();

                if cfg!(debug_assertions) {
                    draw_text("DEBUG MODE", screen_width() - 120.0, screen_height() - 10.0, 16.0, Color::new(0.5, 0.5, 0.5, 1.0));
                }
                
                if wave_director.is_boss_active() {
                    if let Some(boss) = wave_director.get_current_boss() {
                        draw_boss_health_bar(boss);
                    }
                }
                
                // Draw powerup effect indicators
                wave_director.powerup_manager.draw_effect_indicators();
            }
            GameState::Dialogue(ref context) => {
                draw_texture(&background_asset, 0.0, 0.0, WHITE);
                
                for cannonball in &cannonball_vec {
                    cannonball.draw();
                }
                for pirate in &pirate_vec {
                    pirate.draw(&pirate_sprite);
                }
                for enemy in &enemy_vec {
                    if let Some(tex) = enemy_sprites.get(enemy.enemy_type.sprite_name()) {
                        enemy.draw(Some(tex));
                    } else {
                        enemy.draw(None);
                    }
                }
                for bullet in &bullet_vec {
                    if let Some(tex) = bullet_sprites.get(bullet.bullet_type.sprite_name()) {
                        bullet.draw(Some(tex));
                    } else {
                        bullet.draw(None);
                    }
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
                            wave_director.start_wave(wave_director.current_wave + 1);
                        }
                        DialogueCallback::GameComplete => {
                            game.state = GameState::Victory;
                            play_music("victory_music");
                        }
                    }
                }
                
                dialogue_engine.draw(&portraits);
            }
            GameState::Paused { ref mut selected_item } => {
                draw_texture(&background_asset, 0.0, 0.0, WHITE);
                
                // Draw game in background (frozen)
                for cannonball in &cannonball_vec {
                    cannonball.draw();
                }
                for pirate in &pirate_vec {
                    pirate.draw(&pirate_sprite);
                }
                for enemy in &enemy_vec {
                    if let Some(tex) = enemy_sprites.get(enemy.enemy_type.sprite_name()) {
                        enemy.draw(Some(tex));
                    } else {
                        enemy.draw(None);
                    }
                }
                for bullet in &bullet_vec {
                    if let Some(tex) = bullet_sprites.get(bullet.bullet_type.sprite_name()) {
                        bullet.draw(Some(tex));
                    } else {
                        bullet.draw(None);
                    }
                }
                let powerup_textures = std::collections::HashMap::new();
                wave_director.powerup_manager.draw(&powerup_textures);
                if wave_director.is_boss_active() {
                    if let Some(boss) = wave_director.get_current_boss() {
                        boss.draw();
                    }
                }
                ship.draw(&ship_sprite);
                draw_ui(game_score, lives, wave_director.current_wave, &game.narrative);
                wave_director.draw_wave_info();
                if wave_director.is_boss_active() {
                    if let Some(boss) = wave_director.get_current_boss() {
                        draw_boss_health_bar(boss);
                    }
                }
                wave_director.powerup_manager.draw_effect_indicators();
                
                // Draw pause menu on top
                draw_pause_menu(*selected_item);
                
                // Handle pause menu input
                if is_key_pressed(KeyCode::Up) {
                    *selected_item = (*selected_item + 4) % 5;
                }
                if is_key_pressed(KeyCode::Down) {
                    *selected_item = (*selected_item + 1) % 5;
                }
                if is_key_pressed(KeyCode::Enter) {
                    match *selected_item {
                        0 => { // Resume
                            game.state = GameState::Playing;
                            show_mouse(false);
                        }
                        1 => { // Save Game
                            let save_result = save_game(
                                game_score,
                                lives,
                                wave_director.current_wave,
                                wave_director.wave_state,
                                ship.x,
                                ship.y,
                                ship.has_shield,
                                ship.rapid_fire_timer,
                                ship.spread_shot_timer,
                                ship.pierce_timer,
                                game.narrative.clone(),
                                wave_director.wave_timer,
                                wave_director.enemies_spawned,
                                wave_director.powerup_manager.rapid_fire_timer,
                                wave_director.powerup_manager.spread_shot_timer,
                                wave_director.powerup_manager.pierce_timer,
                                wave_director.powerup_manager.has_shield,
                            );
                            if let Err(e) = save_result {
                                eprintln!("Save failed: {}", e);
                            } else {
                                // Could show "Game Saved!" toast here
                            }
                        }
                        2 => { // Console
                            game.state = GameState::Console { input: String::new(), history: Vec::new(), cursor_pos: 0 };
                            show_mouse(true);
                        }
                        3 => { // Quit to Menu
                            game.state = GameState::MainMenu;
                            show_mouse(true);
                            play_music("menu_music");
                        }
                        4 => { // Quit Game
                            std::process::exit(0);
                        }
                        _ => {}
                    }
                }
                if is_key_pressed(KeyCode::Escape) {
                    game.state = GameState::Playing;
                    show_mouse(false);
                }
            }
            GameState::Console { input: ref mut _input, history: ref mut _history, cursor_pos: ref mut _cursor_pos } => {
                // We need to handle console differently to avoid borrow issues
                // Extract state to local variables, then update back
                let result: (Option<GameState>, String, Vec<String>, usize) = {
                    // Use a block to limit the borrow scope
                    if let GameState::Console { ref mut input, ref mut history, ref mut cursor_pos } = game.state {
                        // Handle text input
                        let mut chars_to_add = Vec::new();
                        while let Some(key) = get_char_pressed() {
                            if key != '\u{8}' && key != '\u{7f}' {
                                chars_to_add.push(key);
                            }
                        }
                        
                        for ch in chars_to_add {
                            input.insert(*cursor_pos, ch);
                            *cursor_pos += 1;
                        }
                        
                        // Backspace
                        if is_key_pressed(KeyCode::Backspace) && *cursor_pos > 0 {
                            input.remove(*cursor_pos - 1);
                            *cursor_pos -= 1;
                        }
                        
                        // Cursor movement
                        if is_key_pressed(KeyCode::Left) && *cursor_pos > 0 {
                            *cursor_pos -= 1;
                        }
                        if is_key_pressed(KeyCode::Right) && *cursor_pos < input.len() {
                            *cursor_pos += 1;
                        }
                        if is_key_pressed(KeyCode::Home) {
                            *cursor_pos = 0;
                        }
                        if is_key_pressed(KeyCode::End) {
                            *cursor_pos = input.len();
                        }
                        
                        // Execute command on Enter
                        let new_state = if is_key_pressed(KeyCode::Enter) {
                            let cmd = input.trim().to_lowercase();
                            if !cmd.is_empty() {
                                history.push(format!("> {}", cmd));
                                let state = execute_console_command(&cmd, &mut game.debug_hitbox_visible, &mut game.time_scale, &mut ship, &mut wave_director, &mut game_score, &mut lives, &mut enemy_vec, &mut bullet_vec, &mut pirate_vec, &mut cannonball_vec, &mut pirate_count, history);
                                if let Some(s) = state {
                                    input.clear();
                                    *cursor_pos = 0;
                                    Some(s)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        };
                        
                        // Close console with forward slash
                        let new_state2 = if is_key_pressed(KeyCode::Slash) {
                            Some(GameState::Playing)
                        } else {
                            None
                        };
                        
                        let final_state = new_state.or(new_state2);
                        
                        // Clone values we need for drawing
                        let input_clone = input.clone();
                        let history_clone = history.clone();
                        let cursor_pos_clone = *cursor_pos;
                        
                        if let Some(s) = final_state {
                            if matches!(s, GameState::Playing) {
                                show_mouse(false);
                            } else {
                                show_mouse(true);
                            }
                            (Some(s), input_clone, history_clone, cursor_pos_clone)
                        } else {
                            (None, input_clone, history_clone, cursor_pos_clone)
                        }
                    } else {
                        (None, String::new(), Vec::new(), 0)
                    }
                };
                
                let (new_state, input_clone, history_clone, cursor_pos_clone) = result;
                
                // Update game state if needed
                if let Some(state) = new_state {
                    game.state = state;
                }
                
                // Draw game in background (frozen)
                draw_texture(&background_asset, 0.0, 0.0, WHITE);
                for cannonball in &cannonball_vec {
                    cannonball.draw();
                }
                for pirate in &pirate_vec {
                    pirate.draw(&pirate_sprite);
                }
                for enemy in &enemy_vec {
                    if let Some(tex) = enemy_sprites.get(enemy.enemy_type.sprite_name()) {
                        enemy.draw(Some(tex));
                    } else {
                        enemy.draw(None);
                    }
                }
                for bullet in &bullet_vec {
                    if let Some(tex) = bullet_sprites.get(bullet.bullet_type.sprite_name()) {
                        bullet.draw(Some(tex));
                    } else {
                        bullet.draw(None);
                    }
                }
                let powerup_textures = std::collections::HashMap::new();
                wave_director.powerup_manager.draw(&powerup_textures);
                if wave_director.is_boss_active() {
                    if let Some(boss) = wave_director.get_current_boss() {
                        boss.draw();
                    }
                }
                ship.draw(&ship_sprite);
                draw_ui(game_score, lives, wave_director.current_wave, &game.narrative);
                wave_director.draw_wave_info();
                if wave_director.is_boss_active() {
                    if let Some(boss) = wave_director.get_current_boss() {
                        draw_boss_health_bar(boss);
                    }
                }
                wave_director.powerup_manager.draw_effect_indicators();
                
                // Draw console on top
                draw_console(&input_clone, &history_clone, cursor_pos_clone);
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
                    wave_director.current_boss = Some(boss);
                    wave_director.wave_state = wave_director::WaveState::BossFight;
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
                             &mut enemy_vec, &mut bullet_vec, &mut pirate_count, 
                             &mut game_score, &mut lives);
                    wave_director.start_wave(1);
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
                             &mut enemy_vec, &mut bullet_vec, &mut pirate_count, 
                             &mut game_score, &mut lives);
                    wave_director.start_wave(1);
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
    enemy_vec: &mut Vec<Enemy>,
    bullet_vec: &mut Vec<Bullet>,
    pirate_count: &mut i32,
    game_score: &mut i32,
    lives: &mut i32,
) {
    ship.x = screen_width() * 0.5 - CONFIG.ship_width * 0.5;
    ship.y = screen_height() - CONFIG.ship_start_y_offset;
    ship.gameover = false;
    cannonball_vec.clear();
    pirate_vec.clear();
    enemy_vec.clear();
    bullet_vec.clear();
    *pirate_count = CONFIG.starting_pirate_count;
    *game_score = 0;
    *lives = CONFIG.starting_lives;
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

fn execute_console_command(
    cmd: &str,
    debug_hitbox_visible: &mut bool,
    time_scale: &mut f32,
    ship: &mut Ship,
    wave_director: &mut WaveDirector,
    game_score: &mut i32,
    lives: &mut i32,
    enemy_vec: &mut Vec<Enemy>,
    bullet_vec: &mut Vec<Bullet>,
    pirate_vec: &mut Vec<Pirate>,
    cannonball_vec: &mut Vec<Cannonball>,
    pirate_count: &mut i32,
    history: &mut Vec<String>,
) -> Option<GameState> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }
    
    match parts[0] {
        "help" => {
            history.push("Available commands:".to_string());
            history.push("  help                    - Show this help".to_string());
            history.push("  god                     - Toggle god mode (invincibility)".to_string());
            history.push("  heal                    - Full heal ship".to_string());
            history.push("  wave <n>                - Jump to wave n".to_string());
            history.push("  score <n>               - Set score to n".to_string());
            history.push("  lives <n>               - Set lives to n".to_string());
            history.push("  spawn <enemy_type>      - Spawn enemy (scout, fighter, bomber, interceptor, elite)".to_string());
            history.push("  spawn <boss-name>       - Spawn boss (blowfish, twofish, rufus, molly, davey, deadbeef)".to_string());
            history.push("  killall                 - Kill all enemies".to_string());
            history.push("  powerup <type>          - Give powerup (rapid, spread, pierce, shield)".to_string());
            history.push("  hitbox                  - Toggle hitbox overlay".to_string());
            history.push("  time <scale>            - Set time scale (0.x slow, 2.x fast)".to_string());
            history.push("  state                   - Print current game state".to_string());
            history.push("  damage <boss> <n>       - Damage current boss by n".to_string());
            history.push("  reload                  - Reload content JSON from disk".to_string());
            history.push("  fps                     - Toggle FPS display".to_string());
            history.push("  quit                    - Quit to main menu".to_string());
        }
        "god" => {
            ship.has_shield = !ship.has_shield;
            if ship.has_shield {
                history.push("God mode ENABLED".to_string());
            } else {
                history.push("God mode DISABLED".to_string());
            }
        }
        "heal" => {
            ship.has_shield = true;
            ship.rapid_fire_timer = 30.0;
            ship.spread_shot_timer = 30.0;
            ship.pierce_timer = 30.0;
            wave_director.powerup_manager.rapid_fire_timer = 30.0;
            wave_director.powerup_manager.spread_shot_timer = 30.0;
            wave_director.powerup_manager.pierce_timer = 30.0;
            wave_director.powerup_manager.has_shield = true;
            history.push("Ship healed and powered up!".to_string());
        }
        "wave" => {
            if parts.len() > 1 {
                if let Ok(wave_num) = parts[1].parse::<u32>() {
                    if wave_num >= 1 && wave_num <= 15 {
                        enemy_vec.clear();
                        bullet_vec.clear();
                        cannonball_vec.clear();
                        pirate_vec.clear();
                        *pirate_count = CONFIG.starting_pirate_count;
                        
                        wave_director.start_wave(wave_num);
                        *game_score = wave_director.current_wave as i32 * 1000;
                        history.push(format!("Jumped to wave {}", wave_num));
                    } else {
                        history.push("Wave must be 1-15".to_string());
                    }
                } else {
                    history.push("Invalid wave number".to_string());
                }
            } else {
                history.push("Usage: wave <n>".to_string());
            }
        }
        "score" => {
            if parts.len() > 1 {
                if let Ok(score) = parts[1].parse::<i32>() {
                    *game_score = score;
                    history.push(format!("Score set to {}", score));
                } else {
                    history.push("Invalid score".to_string());
                }
            } else {
                history.push("Usage: score <n>".to_string());
            }
        }
        "lives" => {
            if parts.len() > 1 {
                if let Ok(l) = parts[1].parse::<i32>() {
                    *lives = l.clamp(0, 9);
                    history.push(format!("Lives set to {}", *lives));
                } else {
                    history.push("Invalid lives".to_string());
                }
            } else {
                history.push("Usage: lives <n>".to_string());
            }
        }
        "spawn" => {
            if parts.len() > 1 {
                use crate::enemy::{Enemy, EnemyType};
                match parts[1].to_lowercase().as_str() {
                    "scout" => {
                        let enemy = Enemy::new(EnemyType::Scout, screen_width() / 2.0, -50.0, wave_director.current_wave);
                        enemy_vec.push(enemy);
                        history.push("Spawned Scout".to_string());
                    }
                    "fighter" => {
                        let enemy = Enemy::new(EnemyType::Fighter, screen_width() / 2.0, -50.0, wave_director.current_wave);
                        enemy_vec.push(enemy);
                        history.push("Spawned Fighter".to_string());
                    }
                    "bomber" => {
                        let enemy = Enemy::new(EnemyType::Bomber, screen_width() / 2.0, -50.0, wave_director.current_wave);
                        enemy_vec.push(enemy);
                        history.push("Spawned Bomber".to_string());
                    }
                    "interceptor" => {
                        let enemy = Enemy::new(EnemyType::Interceptor, screen_width() / 2.0, -50.0, wave_director.current_wave);
                        enemy_vec.push(enemy);
                        history.push("Spawned Interceptor".to_string());
                    }
                    "elite" => {
                        let enemy = Enemy::new(EnemyType::Elite, screen_width() / 2.0, -50.0, wave_director.current_wave);
                        enemy_vec.push(enemy);
                        history.push("Spawned Elite".to_string());
                    }
                    "blowfish" | "twofish" | "rufus" | "rufusreverse" | "molly" | "mollyhashpass" | "davey" | "captaindavey" | "deadbeef" => {
                        use crate::boss::BossType;
                        let boss_type = match parts[1].to_lowercase().as_str() {
                            "blowfish" => BossType::Blowfish,
                            "twofish" => BossType::Twofish,
                            "rufus" | "rufusreverse" => BossType::RufusReverse,
                            "molly" | "mollyhashpass" => BossType::MollyHashpass,
                            "davey" | "captaindavey" => BossType::CaptainDavey,
                            _ => BossType::Deadbeef,
                        };
                        enemy_vec.clear();
                        bullet_vec.clear();
                        cannonball_vec.clear();
                        pirate_vec.clear();
                        *pirate_count = 0;
                        wave_director.current_boss = Some(Boss::new(boss_type));
                        wave_director.wave_state = WaveState::BossFight;
                        wave_director.is_boss_wave = true;
                        wave_director.current_wave = 99;
                        history.push(format!("Spawned boss: {:?}", boss_type));
                        return Some(GameState::Playing);
                    }
                    _ => {
                        history.push("Unknown entity. Use: scout, fighter, bomber, interceptor, elite, or a boss name".to_string());
                    }
                }
            } else {
                history.push("Usage: spawn <entity>".to_string());
            }
        }
        "killall" => {
            for enemy in enemy_vec.iter_mut() {
                enemy.is_dead = true;
            }
            for pirate in pirate_vec.iter_mut() {
                pirate.is_dead = true;
            }
            bullet_vec.clear();
            cannonball_vec.clear();
            history.push("All enemies killed".to_string());
        }
        "powerup" => {
            if parts.len() > 1 {
                use crate::ship::PowerupEffectType;
                let effect = match parts[1].to_lowercase().as_str() {
                    "rapid" => PowerupEffectType::RapidFire,
                    "spread" => PowerupEffectType::SpreadShot,
                    "pierce" => PowerupEffectType::Pierce,
                    "shield" => PowerupEffectType::Shield,
                    _ => {
                        history.push("Unknown powerup. Use: rapid, spread, pierce, shield".to_string());
                        return None;
                    }
                };
                ship.apply_powerup(effect, 30.0);
                let mgr = &mut wave_director.powerup_manager;
                match effect {
                    PowerupEffectType::RapidFire => mgr.rapid_fire_timer = 30.0,
                    PowerupEffectType::SpreadShot => mgr.spread_shot_timer = 30.0,
                    PowerupEffectType::Pierce => mgr.pierce_timer = 30.0,
                    PowerupEffectType::Shield => mgr.has_shield = true,
                }
                history.push(format!("Gave powerup: {:?}", effect));
            } else {
                history.push("Usage: powerup <type>".to_string());
            }
        }
        "hitbox" => {
            *debug_hitbox_visible = !*debug_hitbox_visible;
            history.push(if *debug_hitbox_visible { "Hitbox overlay ON".to_string() } else { "Hitbox overlay OFF".to_string() });
        }
        "time" => {
            if parts.len() > 1 {
                if let Ok(scale) = parts[1].parse::<f32>() {
                    *time_scale = scale.max(0.01).min(10.0);
                    history.push(format!("Time scale set to {}", *time_scale));
                } else {
                    history.push("Invalid time scale".to_string());
                }
            } else {
                history.push(format!("Current time scale: {}", *time_scale));
            }
        }
        "state" => {
            history.push(format!("Wave: {} | State: {:?} | Boss wave: {}", wave_director.current_wave, wave_director.wave_state, wave_director.is_boss_wave));
            history.push(format!("Score: {} | Lives: {} | Pirate count: {}", game_score, lives, pirate_count));
            history.push(format!("Ship: ({:.0}, {:.0}) | Shield: {} | Invuln: {:.1}s", ship.x, ship.y, ship.has_shield, ship.invuln_timer));
            if let Some(boss) = &wave_director.current_boss {
                history.push(format!("Boss: {} | HP: {}/{} | Phase: {:?} | Invuln: {}", boss.boss_type.name(), boss.health, boss.max_health, boss.phase, boss.invulnerable));
            }
            history.push(format!("Powerups: rapid_fire={:.1}s spread={:.1}s pierce={:.1}s shield={}", 
                wave_director.powerup_manager.rapid_fire_timer,
                wave_director.powerup_manager.spread_shot_timer,
                wave_director.powerup_manager.pierce_timer,
                wave_director.powerup_manager.has_shield,
            ));
        }
        "damage" => {
            if let Some(boss) = &mut wave_director.current_boss {
                if parts.len() > 1 {
                    if let Ok(dmg) = parts[1].parse::<i32>() {
                        if boss.take_damage(dmg) {
                            *game_score += 1000;
                            wave_director.wave_state = WaveState::BossDefeated;
                            history.push("Boss defeated!".to_string());
                        } else {
                            history.push(format!("Boss took {} damage. HP: {}/{}", dmg, boss.health, boss.max_health));
                        }
                    } else {
                        history.push("Invalid damage amount".to_string());
                    }
                } else {
                    history.push("Usage: damage <n>".to_string());
                }
            } else {
                history.push("No boss active".to_string());
            }
        }
        "fps" => {
            history.push("FPS display toggle not implemented yet".to_string());
        }
        "reload" => {
            match content::reload() {
                Ok(()) => {
                    history.push("Content reloaded from assets/content/".to_string());
                    if let Err(issues) = content::validate() {
                        history.push("[content] validation issues:".to_string());
                        for line in issues.lines() {
                            history.push(format!("  {line}"));
                        }
                    }
                    for key in ["scout", "fighter", "bomber", "interceptor", "elite"] {
                        if content::enemy(key).is_none() {
                            history.push(format!("WARNING: no archetype for enemy \"{key}\""));
                        }
                    }
                    for boss_key in content::load().bosses.keys() {
                        if BossType::from_key(boss_key).is_none() {
                            history.push(format!("WARNING: unknown boss key \"{boss_key}\""));
                        }
                    }
                }
                Err(err) => history.push(format!("Reload failed: {err}")),
            }
        }
        "quit" => {
            history.push("Returning to main menu...".to_string());
            return Some(GameState::MainMenu);
        }
        _ => {
            history.push(format!("Unknown command: {}. Type 'help' for list.", cmd));
        }
    }
    None
}

fn check_collisions(
    enemy_vec: &mut Vec<Enemy>,
    bullet_vec: &mut Vec<Bullet>,
    cannonball_vec: &mut Vec<Cannonball>,
    ship: &mut Ship,
    pirate_vec: &mut Vec<Pirate>,
    pirate_count: &mut i32,
    game_score: &mut i32,
    lives: &mut i32,
    wave_director: &mut WaveDirector,
) {
    let mut rng = ::rand::thread_rng();
    
    for cannonball in cannonball_vec.iter_mut() {
        for enemy in enemy_vec.iter_mut() {
            if !enemy.is_dead {
                let (ex, ey, ew, eh) = enemy.get_rect();
                let (cx, cy, cw, ch) = cannonball.get_rect();
                if cx < ex + ew && cx + cw > ex && cy < ey + eh && cy + ch > ey {
                    if enemy.take_damage(cannonball.damage) {
                        *game_score += enemy.score_value;
                        wave_director.try_spawn_powerup(enemy);
                    }
                    cannonball.y = -100.0;
                    break;
                }
            }
        }
        
        for pirate in pirate_vec.iter_mut() {
            if !pirate.is_dead {
                let (px, py, pw, ph) = pirate.get_rect();
                let (cx, cy, cw, ch) = cannonball.get_rect();
                if cx < px + pw && cx + cw > px && cy < py + ph && cy + ch > py {
                    pirate.is_dead = true;
                    *game_score += 1;
                    break;
                }
            }
        }
    }
    
    for bullet in bullet_vec.iter_mut() {
        if bullet.is_dead || bullet.is_laser {
            continue;
        }
        
        for enemy in enemy_vec.iter_mut() {
            if !enemy.is_dead {
                let (ex, ey, ew, eh) = enemy.get_rect();
                let (bx, by, bw, bh) = bullet.get_rect();
                if bx < ex + ew && bx + bw > ex && by < ey + eh && by + bh > ey {
                    if enemy.take_damage(bullet.damage) {
                        *game_score += enemy.score_value;
                        wave_director.try_spawn_powerup(enemy);
                    }
                    
                    if !bullet.pierce() {
                        bullet.is_dead = true;
                    }
                    bullet.hit_flash = 0.1;
                    break;
                }
            }
        }
        
        for pirate in pirate_vec.iter_mut() {
            if !pirate.is_dead {
                let (px, py, pw, ph) = pirate.get_rect();
                let (bx, by, bw, bh) = bullet.get_rect();
                if bx < px + pw && bx + bw > px && by < py + ph && by + bh > py {
                    pirate.is_dead = true;
                    *game_score += 1;
                    if !bullet.pierce() {
                        bullet.is_dead = true;
                    }
                    bullet.hit_flash = 0.1;
                }
            }
        }
    }
    
    for bullet in bullet_vec.iter_mut() {
        if !bullet.is_dead && bullet.is_laser {
            for enemy in enemy_vec.iter_mut() {
                if !enemy.is_dead {
                    let (ex, ey, ew, eh) = enemy.get_rect();
                    let (bx, by, bw, bh) = bullet.get_rect();
                    if bx < ex + ew && bx + bw > ex && by < ey + eh && by + bh > ey {
                        if enemy.take_damage(bullet.damage) {
                            *game_score += enemy.score_value;
                            wave_director.try_spawn_powerup(enemy);
                        }
                    }
                }
            }
        }
    }
    
    for bullet in bullet_vec.iter_mut() {
        if bullet.is_dead || ship.invuln_timer > 0.0 {
            continue;
        }
        
        let (bx, by, bw, bh) = bullet.get_rect();
        let (sx, sy, sw, sh) = ship.get_rect();
        
        if bx < sx + sw && bx + bw > sx && by < sy + sh && by + bh > sy {
            bullet.is_dead = true;
            if ship.has_shield() {
                ship.consume_shield();
            } else if *lives > 0 {
                *lives -= 1;
                ship.invuln_timer = 2.0;
            } else {
                ship.gameover = true;
            }
        }
    }
    
    for pirate in pirate_vec.iter() {
        if !pirate.is_dead {
            let (px, py, pw, ph) = pirate.get_rect();
            let (sx, sy, sw, sh) = ship.get_rect();
            if sx < px + pw && sx + sw > px && sy < py + ph && sy + sh > py {
                if *lives > 0 {
                    *lives -= 1;
                } else {
                    ship.gameover = true;
                }
            }
        }
    }
    
    enemy_vec.retain(|e| !e.is_dead);
    pirate_vec.retain(|p| !p.is_dead && p.y < screen_height());
}

fn boss_for_wave(wave: u32) -> BossType {
    crate::content::boss_for_wave(wave)
        .and_then(BossType::from_key)
        .unwrap_or(BossType::Blowfish)
}