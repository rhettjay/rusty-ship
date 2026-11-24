use macroquad::prelude::*;
use std::collections::HashMap;

mod audio;
mod boss;
mod bullet;
mod collision;
mod config;
mod content;
mod dialogue;
mod enemy;
mod menu;
mod narrative;
mod pirate;
mod portrait;
mod powerup;
mod save;
mod ship;
mod wave_director;

use boss::{Boss, BossType};
use bullet::{create_spread_bullets, Bullet, BulletType};
use collision::rects_overlap;
use config::CONFIG;
use dialogue::DialogueEngine;
use enemy::{Enemy, EnemyType, ShootPattern};
use menu::{char_to_byte, DialogueCallback, DialogueContext, Game, GameState, NarrativeProgress};
use narrative::{check_triggers, NarrativeTrigger};
use pirate::Pirate;
use portrait::{Character, PortraitManager};
use save::{save_game, GameSave};
use ship::Ship;
use wave_director::{WaveDirector, WaveState};

/// Mutable gameplay state, bundled so update functions don't thread half a
/// dozen `&mut` arguments through every call.
struct World {
    ship: Ship,
    enemies: Vec<Enemy>,
    bullets: Vec<Bullet>,
    pirates: Vec<Pirate>,
    wave_director: WaveDirector,
    score: i32,
    lives: i32,
}

impl World {
    fn new() -> Self {
        Self {
            ship: Ship::new(),
            enemies: Vec::new(),
            bullets: Vec::new(),
            pirates: Vec::new(),
            wave_director: WaveDirector::new(),
            score: 0,
            lives: CONFIG.starting_lives,
        }
    }

    /// Full reset for a fresh game — including powerup state, which
    /// previously leaked between playthroughs.
    fn reset(&mut self) {
        self.ship.reset_position();
        self.enemies.clear();
        self.bullets.clear();
        self.pirates.clear();
        self.score = 0;
        self.lives = CONFIG.starting_lives;
        self.wave_director.powerup_manager.reset();
    }
}

/// Textures loaded once at startup. Sprite maps are keyed by each entity
/// type's `sprite_name()`.
struct Assets {
    background: Texture2D,
    ship: Texture2D,
    pirate: Texture2D,
    enemies: HashMap<String, Texture2D>,
    bullets: HashMap<String, Texture2D>,
    powerups: HashMap<String, Texture2D>,
}

async fn load_sprite_map(dir: &str, names: &[&str]) -> HashMap<String, Texture2D> {
    let mut map = HashMap::new();
    for name in names {
        let path = format!("{dir}/{name}.png");
        if let Ok(tex) = load_texture(&path).await {
            tex.set_filter(FilterMode::Nearest);
            map.insert(name.to_string(), tex);
        }
    }
    map
}

async fn load_assets() -> Assets {
    Assets {
        background: Texture2D::from_file_with_format(
            include_bytes!("../assets/background/background.png"),
            None,
        ),
        ship: Texture2D::from_file_with_format(include_bytes!("../assets/ship.png"), None),
        pirate: Texture2D::from_file_with_format(include_bytes!("../assets/pirate.png"), None),
        enemies: load_sprite_map(
            "assets/enemies",
            &["scout", "fighter", "bomber", "interceptor"],
        )
        .await,
        bullets: load_sprite_map(
            "assets/bullets",
            &[
                "player_standard",
                "player_spread",
                "player_pierce",
                "enemy_straight",
                "enemy_aimed",
                "enemy_bomb",
                "enemy_spread",
            ],
        )
        .await,
        powerups: load_sprite_map(
            "assets/powerups",
            &[
                "rapid_fire",
                "spread_shot",
                "pierce",
                "shield",
                "bomb",
                "life",
                "score",
            ],
        )
        .await,
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: CONFIG.window_title.to_string(),
        window_width: CONFIG.window_width as i32,
        window_height: CONFIG.window_height as i32,
        ..Default::default()
    }
}

#[macroquad::main(window_conf())]
async fn main() {
    show_mouse(false);

    content::load();
    if let Err(issues) = content::validate() {
        eprintln!("[content] validation issues:\n{issues}");
    }

    let _audio_manager = audio::init_audio().await;
    audio::set_music_volume(CONFIG.music_volume);
    audio::set_sfx_volume(CONFIG.sfx_volume);

    let mut portraits = PortraitManager::new();
    portraits.load_all().await;

    let mut dialogue_engine = DialogueEngine::new();
    dialogue_engine.typewriter_speed = CONFIG.dialogue_typewriter_speed;

    let assets = load_assets().await;

    let mut game = Game::new();
    let mut world = World::new();
    let mut last_shot_time = get_time();

    audio::play_music("menu_music");

    loop {
        let dt = (get_frame_time() as f64) * game.time_scale as f64;
        audio::update_audio(dt);
        clear_background(BLACK);

        match game.state {
            GameState::MainMenu => {
                menu::draw_menu(&game, &assets.background);
                update_main_menu(&mut game, &mut world, &mut dialogue_engine);
            }
            GameState::Playing => {
                update_playing(
                    dt,
                    &mut game,
                    &mut world,
                    &mut dialogue_engine,
                    &mut last_shot_time,
                );
                draw_world(&world, &game, &assets);
            }
            GameState::Dialogue(_) => {
                draw_world(&world, &game, &assets);
                update_dialogue(dt, &mut game, &mut world, &mut dialogue_engine);
                dialogue_engine.draw(&portraits);
            }
            GameState::Paused { .. } => {
                draw_world(&world, &game, &assets);
                menu::draw_pause_menu(pause_selection(&game));
                update_paused(&mut game, &mut world);
            }
            GameState::Console { .. } => {
                draw_world(&world, &game, &assets);
                update_console(&mut game, &mut world);
            }
            GameState::BossIntro(boss_type) => {
                draw_world(&world, &game, &assets);
                update_boss_intro(&mut game, &mut world, boss_type, &portraits).await;
            }
            GameState::GameOver => {
                draw_game_over_screen(&world);
                update_end_screen(&mut game, &mut world);
            }
            GameState::Victory => {
                draw_victory_screen(&world);
                update_end_screen(&mut game, &mut world);
            }
        }

        next_frame().await;
    }
}

fn pause_selection(game: &Game) -> usize {
    match &game.state {
        GameState::Paused { selected_item } => *selected_item,
        _ => 0,
    }
}

// ---------------------------------------------------------------------------
// State: main menu
// ---------------------------------------------------------------------------

fn update_main_menu(game: &mut Game, world: &mut World, dialogue_engine: &mut DialogueEngine) {
    let has_save = save::has_save_file();
    let menu_items: &[&str] = if has_save {
        &["Start Game", "Load Game", "Quit"]
    } else {
        &["Start Game", "Quit"]
    };
    let item_count = menu_items.len();
    // Keep the selection in bounds even if the save file appeared/vanished.
    game.selected_menu_item %= item_count;

    if is_key_pressed(KeyCode::Up) {
        game.selected_menu_item = (game.selected_menu_item + item_count - 1) % item_count;
    }
    if is_key_pressed(KeyCode::Down) {
        game.selected_menu_item = (game.selected_menu_item + 1) % item_count;
    }
    if is_key_pressed(KeyCode::Enter) {
        match menu_items[game.selected_menu_item] {
            "Start Game" => {
                world.reset();
                game.narrative = NarrativeProgress::new();
                begin_wave(1, game, world);
                audio::play_music("gameplay_music");
                show_mouse(false);

                // Intro dialogue on a fresh narrative.
                if let Some(context) =
                    check_triggers(&mut game.narrative, NarrativeTrigger::GameStart)
                {
                    if start_dialogue(dialogue_engine, &context) {
                        game.state = GameState::Dialogue(context);
                    }
                }
            }
            "Load Game" => match save::load_game() {
                Ok(save_data) => {
                    world.reset();
                    apply_save(save_data, world, game);
                    route_loaded_state(game, world);
                    audio::play_music("gameplay_music");
                    show_mouse(false);
                }
                Err(e) => eprintln!("Load failed: {e}"),
            },
            "Quit" => std::process::exit(0),
            _ => {}
        }
    }
    if is_key_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
}

/// After `apply_save`, route to the right screen depending on where the save
/// was taken — this is what prevents saved-at-completion saves from
/// soft-locking.
fn route_loaded_state(game: &mut Game, world: &mut World) {
    match world.wave_director.wave_state {
        // A boss-intro save resumes at the intro screen so the boss spawns.
        WaveState::BossIntro => {
            game.state = GameState::BossIntro(boss_for_wave(world.wave_director.current_wave));
        }
        // Completion saves advance to the next wave instead of freezing.
        WaveState::Complete | WaveState::BossDefeated => {
            begin_wave(world.wave_director.current_wave + 1, game, world);
        }
        // BossFight saves are refused at write time; treat one as boss intro.
        WaveState::BossFight => {
            world.wave_director.wave_state = WaveState::BossIntro;
            game.state = GameState::BossIntro(boss_for_wave(world.wave_director.current_wave));
        }
        _ => {
            game.state = GameState::Playing;
        }
    }
}

/// Restores a save on top of a content-derived wave setup: `start_wave`
/// pulls every spawn parameter from the content tables, then the serialized
/// progress markers are layered back in.
fn apply_save(save_data: GameSave, world: &mut World, game: &mut Game) {
    world.score = save_data.score;
    world.lives = save_data.lives;
    game.narrative = save_data.narrative;

    world.wave_director.start_wave(save_data.current_wave);
    world.wave_director.restore_in_progress(
        save_data.wave_state,
        save_data.wave_timer,
        save_data.enemies_spawned,
    );

    let mgr = &mut world.wave_director.powerup_manager;
    mgr.rapid_fire_timer = save_data.powerup_rapid_fire_timer;
    mgr.spread_shot_timer = save_data.powerup_spread_shot_timer;
    mgr.pierce_timer = save_data.powerup_pierce_timer;
    mgr.has_shield = save_data.powerup_has_shield;
}

// ---------------------------------------------------------------------------
// State: playing
// ---------------------------------------------------------------------------

fn update_playing(
    dt: f64,
    game: &mut Game,
    world: &mut World,
    dialogue_engine: &mut DialogueEngine,
    last_shot_time: &mut f64,
) {
    if is_key_pressed(KeyCode::Escape) {
        game.state = GameState::Paused { selected_item: 0 };
        show_mouse(true);
        return;
    }
    if is_key_pressed(KeyCode::Slash) {
        game.state = GameState::Console {
            input: String::new(),
            history: Vec::new(),
            cursor_pos: 0,
        };
        show_mouse(true);
        return;
    }

    // Rufus Reverse's reverse zone inverts the controls.
    let reversed = world.wave_director.is_reverse_zone_active();
    world.ship.update_input(dt as f32, reversed);
    world.ship.update(dt as f32);

    // Firing.
    if is_key_down(KeyCode::Space) {
        let now = get_time();
        let cooldown = world.wave_director.get_fire_cooldown(CONFIG.fire_cooldown);
        if now - *last_shot_time >= cooldown {
            fire_player_bullets(world);
            audio::play_sfx("cannon_fire");
            *last_shot_time = now;
        }
    }

    // Wave simulation.
    let prev_wave_state = world.wave_director.wave_state;
    world.wave_director.update(
        dt,
        &mut world.enemies,
        &mut world.bullets,
        &mut world.ship,
        &mut world.pirates,
        &mut world.score,
        &mut world.lives,
    );
    // Boss minion signals (Captain Davey's crew, Molly's revives).
    world
        .wave_director
        .drain_minion_signals(&mut world.pirates, screen_width());

    // Enemy behavior and firing.
    let (ship_x, ship_y) = (world.ship.x, world.ship.y);
    for enemy in world.enemies.iter_mut() {
        enemy.update(dt, ship_x, ship_y);

        if enemy.can_shoot() {
            enemy.reset_shoot_timer();
            enemy_fire(enemy, ship_x, ship_y, &mut world.bullets);
        }
    }

    for bullet in world.bullets.iter_mut() {
        bullet.update(dt);
    }
    world.bullets.retain(|b| !b.is_dead);

    check_collisions(world);

    // Wave-completion transition (fires exactly once per completion).
    let completed_now = world.wave_director.is_wave_complete()
        && !matches!(
            prev_wave_state,
            WaveState::Complete | WaveState::BossDefeated
        );
    if completed_now {
        on_wave_complete(game, world, dialogue_engine);
    }

    // Powerup pickups.
    world.wave_director.check_powerup_pickup(
        &mut world.ship,
        &mut world.pirates,
        &mut world.score,
        &mut world.lives,
    );

    if world.ship.gameover {
        game.state = GameState::GameOver;
        audio::play_music("menu_music");
    }
}

fn fire_player_bullets(world: &mut World) {
    let bullet_type = if world.wave_director.is_spread_shot_active() {
        BulletType::PlayerSpread
    } else if world.wave_director.is_pierce_active() {
        BulletType::PlayerPierce
    } else {
        BulletType::PlayerStandard
    };

    let bullet_speed = bullet_type.speed();
    let muzzle_x = world.ship.x + world.ship.w * 0.5;
    let muzzle_y = world.ship.y - 10.0;

    let bullets = if bullet_type == BulletType::PlayerSpread {
        create_spread_bullets(
            bullet_type,
            muzzle_x,
            muzzle_y,
            0.0,
            -bullet_speed,
            3,
            std::f32::consts::FRAC_PI_4,
        )
    } else {
        vec![Bullet::new(
            bullet_type,
            muzzle_x,
            muzzle_y,
            0.0,
            -bullet_speed,
        )]
    };
    world.bullets.extend(bullets);
}

fn enemy_fire(enemy: &Enemy, ship_x: f32, ship_y: f32, bullets: &mut Vec<Bullet>) {
    match enemy.shoot_pattern {
        ShootPattern::None => {}
        ShootPattern::Spread => {
            let speed = BulletType::EnemySpread.speed();
            let spread = create_spread_bullets(
                BulletType::EnemySpread,
                enemy.x,
                enemy.y + enemy.height / 2.0,
                0.0,
                speed,
                3,
                std::f32::consts::FRAC_PI_4,
            );
            bullets.extend(spread);
        }
        pattern => {
            let bullet_type = match pattern {
                ShootPattern::Straight => BulletType::EnemyStraight,
                ShootPattern::Aimed => BulletType::EnemyAimed,
                ShootPattern::Bomb => BulletType::EnemyBomb,
                _ => BulletType::EnemyStraight,
            };
            let (dir_x, dir_y) = enemy.get_shoot_direction(ship_x, ship_y);
            bullets.push(Bullet::new(
                bullet_type,
                enemy.x,
                enemy.y + enemy.height / 2.0,
                dir_x * bullet_type.speed(),
                dir_y * bullet_type.speed(),
            ));
        }
    }
}

/// The once-per-completion transition: boss-defeat dialogue, next wave,
/// bonus boss, autosave at the new wave's start.
fn on_wave_complete(game: &mut Game, world: &mut World, dialogue_engine: &mut DialogueEngine) {
    if world.wave_director.is_boss_wave {
        // Story boss defeated — always yields a defeat dialogue.
        let boss_type = boss_for_wave(world.wave_director.current_wave);
        if let Some(context) = check_triggers(
            &mut game.narrative,
            NarrativeTrigger::BossDefeated(boss_type),
        ) {
            if start_dialogue(dialogue_engine, &context) {
                game.state = GameState::Dialogue(context);
                return;
            }
        }
        // Fail-loud fallback: dialogue asset missing, keep the game moving.
        advance_after_chapter(game, world, dialogue_engine);
    } else {
        // Either a regular wave completed, or the bonus boss (spawned
        // mid-wave by the score threshold) was defeated — in the bonus case
        // the interrupted wave restarts.
        let next_wave = if world.wave_director.current_boss.is_some() {
            world.wave_director.current_wave
        } else {
            world.wave_director.current_wave + 1
        };

        // WaveCleared may yield a boss-intro dialogue for the next wave.
        if let Some(context) = check_triggers(
            &mut game.narrative,
            NarrativeTrigger::WaveCleared(next_wave),
        ) {
            if start_dialogue(dialogue_engine, &context) {
                game.state = GameState::Dialogue(context);
                return;
            }
        }

        begin_wave(next_wave, game, world);

        // Score threshold: the bonus boss ambush (once per narrative).
        if world.score >= CONFIG.bonus_score_threshold && !world.wave_director.is_boss_active() {
            if let Some(context) = check_triggers(
                &mut game.narrative,
                NarrativeTrigger::ScoreThreshold(world.score),
            ) {
                if start_dialogue(dialogue_engine, &context) {
                    game.state = GameState::Dialogue(context);
                }
            }
        }
    }
}

/// Starts a wave, autosaves at its (always loadable) starting point, and
/// routes boss waves through their intro screen.
fn begin_wave(wave: u32, game: &mut Game, world: &mut World) {
    world.wave_director.start_wave(wave);
    game.narrative.current_wave = wave;
    game.state = GameState::Playing;

    autosave(game, world);

    if world.wave_director.is_boss_wave {
        game.state = GameState::BossIntro(boss_for_wave(wave));
    }
}

/// Post-boss-defeat progression: next chapter, or the victory ending once
/// all five story bosses are down.
fn advance_after_chapter(game: &mut Game, world: &mut World, dialogue_engine: &mut DialogueEngine) {
    game.narrative.current_chapter += 1;

    if let Some(context) = check_triggers(&mut game.narrative, NarrativeTrigger::AllBossesDefeated)
    {
        if start_dialogue(dialogue_engine, &context) {
            game.state = GameState::Dialogue(context);
            return;
        }
    }

    begin_wave(world.wave_director.current_wave + 1, game, world);
}

fn build_save(game: &Game, world: &World) -> GameSave {
    let mgr = &world.wave_director.powerup_manager;
    GameSave {
        score: world.score,
        lives: world.lives,
        current_wave: world.wave_director.current_wave,
        wave_state: world.wave_director.wave_state,
        wave_timer: world.wave_director.wave_timer,
        enemies_spawned: world.wave_director.enemies_spawned,
        powerup_rapid_fire_timer: mgr.rapid_fire_timer,
        powerup_spread_shot_timer: mgr.spread_shot_timer,
        powerup_pierce_timer: mgr.pierce_timer,
        powerup_has_shield: mgr.has_shield,
        narrative: game.narrative.clone(),
    }
}

fn autosave(game: &Game, world: &World) {
    if let Err(e) = save_game(&build_save(game, world)) {
        eprintln!("Auto-save failed: {e}");
    }
}

/// Fail-loud dialogue start: a missing dialogue asset logs and returns false
/// so progression continues instead of soft-locking.
fn start_dialogue(dialogue_engine: &mut DialogueEngine, context: &DialogueContext) -> bool {
    match dialogue_engine.start_dialogue(context) {
        Ok(()) => true,
        Err(e) => {
            eprintln!(
                "[dialogue] failed to start '{}': {e} — continuing without it",
                context.dialogue_id
            );
            false
        }
    }
}

// ---------------------------------------------------------------------------
// State: dialogue
// ---------------------------------------------------------------------------

fn update_dialogue(
    dt: f64,
    game: &mut Game,
    world: &mut World,
    dialogue_engine: &mut DialogueEngine,
) {
    let Some(callback) = dialogue_engine.update(dt) else {
        return;
    };

    match callback {
        DialogueCallback::ResumeGame => {
            game.state = GameState::Playing;
        }
        DialogueCallback::SpawnBoss(boss_type) => {
            game.state = GameState::BossIntro(boss_type);
        }
        DialogueCallback::NextChapter => {
            advance_after_chapter(game, world, dialogue_engine);
        }
        DialogueCallback::GameComplete => {
            game.state = GameState::Victory;
            audio::play_music("victory_music");
        }
    }
}

// ---------------------------------------------------------------------------
// State: paused
// ---------------------------------------------------------------------------

fn update_paused(game: &mut Game, world: &mut World) {
    let selection = {
        let GameState::Paused { selected_item } = &mut game.state else {
            return;
        };
        if is_key_pressed(KeyCode::Up) {
            *selected_item = (*selected_item + 4) % 5;
        }
        if is_key_pressed(KeyCode::Down) {
            *selected_item = (*selected_item + 1) % 5;
        }
        if is_key_pressed(KeyCode::Enter) {
            Some(*selected_item)
        } else {
            None
        }
    };

    if is_key_pressed(KeyCode::Escape) {
        game.state = GameState::Playing;
        show_mouse(false);
        return;
    }

    if let Some(item) = selection {
        match item {
            0 => {
                // Resume
                game.state = GameState::Playing;
                show_mouse(false);
            }
            1 => {
                // Save Game
                if let Err(e) = save_game(&build_save(game, world)) {
                    eprintln!("Save failed: {e}");
                }
            }
            2 => {
                // Console
                game.state = GameState::Console {
                    input: String::new(),
                    history: Vec::new(),
                    cursor_pos: 0,
                };
                show_mouse(true);
            }
            3 => {
                // Quit to Menu
                game.state = GameState::MainMenu;
                show_mouse(true);
                audio::play_music("menu_music");
            }
            4 => std::process::exit(0),
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// State: console
// ---------------------------------------------------------------------------

fn update_console(game: &mut Game, world: &mut World) {
    // Phase 1: text editing and command capture. This borrows the console
    // state out of `game.state`.
    let pending = {
        let GameState::Console {
            input,
            history,
            cursor_pos,
        } = &mut game.state
        else {
            return;
        };

        let mut chars_to_add = Vec::new();
        while let Some(key) = get_char_pressed() {
            if key != '\u{8}' && key != '\u{7f}' {
                chars_to_add.push(key);
            }
        }
        for ch in chars_to_add {
            let byte_idx = char_to_byte(input, *cursor_pos);
            input.insert(byte_idx, ch);
            *cursor_pos += 1;
        }

        if is_key_pressed(KeyCode::Backspace) && *cursor_pos > 0 {
            let byte_idx = char_to_byte(input, *cursor_pos - 1);
            input.remove(byte_idx);
            *cursor_pos -= 1;
        }
        if is_key_pressed(KeyCode::Left) && *cursor_pos > 0 {
            *cursor_pos -= 1;
        }
        if is_key_pressed(KeyCode::Right) && *cursor_pos < input.chars().count() {
            *cursor_pos += 1;
        }
        if is_key_pressed(KeyCode::Home) {
            *cursor_pos = 0;
        }
        if is_key_pressed(KeyCode::End) {
            *cursor_pos = input.chars().count();
        }

        if is_key_pressed(KeyCode::Enter) {
            let cmd = input.trim().to_lowercase();
            if !cmd.is_empty() {
                history.push(format!("> {cmd}"));
                input.clear();
                *cursor_pos = 0;
                Some(cmd)
            } else {
                None
            }
        } else {
            None
        }
    };

    // Phase 2: command execution. `history` borrows `game.state` while the
    // command mutates disjoint fields of `game`, so the state is re-matched
    // here.
    let mut new_state = None;
    if let Some(cmd) = pending {
        if let GameState::Console { history, .. } = &mut game.state {
            new_state = execute_console_command(
                &cmd,
                &mut game.debug_hitbox_visible,
                &mut game.time_scale,
                world,
                history,
            );
        }
    }

    if is_key_pressed(KeyCode::Slash) {
        new_state = new_state.or(Some(GameState::Playing));
    }

    if let Some(state) = new_state {
        if matches!(state, GameState::Playing) {
            show_mouse(false);
        }
        game.state = state;
    }

    if let GameState::Console {
        input,
        history,
        cursor_pos,
    } = &game.state
    {
        menu::draw_console(input, history, *cursor_pos);
    }
}

#[allow(clippy::too_many_arguments)]
fn execute_console_command(
    cmd: &str,
    debug_hitbox_visible: &mut bool,
    time_scale: &mut f32,
    world: &mut World,
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
            history.push("  powerup <type>          - Give powerup (rapid, spread, pierce, shield, bomb, life, score)".to_string());
            history.push("  hitbox                  - Toggle hitbox overlay".to_string());
            history.push(
                "  time <scale>            - Set time scale (0.x slow, 2.x fast)".to_string(),
            );
            history.push("  state                   - Print current game state".to_string());
            history.push("  damage <n>              - Damage current boss by n".to_string());
            history.push("  reload                  - Reload content JSON from disk".to_string());
            history.push("  quit                    - Quit to main menu".to_string());
        }
        "god" => {
            let mgr = &mut world.wave_director.powerup_manager;
            mgr.has_shield = !mgr.has_shield;
            history.push(if mgr.has_shield {
                "God mode ENABLED".to_string()
            } else {
                "God mode DISABLED".to_string()
            });
        }
        "heal" => {
            let mgr = &mut world.wave_director.powerup_manager;
            mgr.rapid_fire_timer = 30.0;
            mgr.spread_shot_timer = 30.0;
            mgr.pierce_timer = 30.0;
            mgr.has_shield = true;
            history.push("Ship healed and powered up!".to_string());
        }
        "wave" => {
            if let Some(wave_arg) = parts.get(1) {
                if let Ok(wave_num) = wave_arg.parse::<u32>() {
                    if (1..=15).contains(&wave_num) {
                        world.enemies.clear();
                        world.bullets.clear();
                        world.pirates.clear();
                        world.wave_director.start_wave(wave_num);
                        world.score = wave_num as i32 * 1000;
                        history.push(format!("Jumped to wave {wave_num}"));
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
            if let Some(score) = parts.get(1).and_then(|s| s.parse::<i32>().ok()) {
                world.score = score;
                history.push(format!("Score set to {score}"));
            } else {
                history.push("Usage: score <n>".to_string());
            }
        }
        "lives" => {
            if let Some(l) = parts.get(1).and_then(|s| s.parse::<i32>().ok()) {
                world.lives = l.clamp(0, 9);
                history.push(format!("Lives set to {}", world.lives));
            } else {
                history.push("Usage: lives <n>".to_string());
            }
        }
        "spawn" => {
            if let Some(entity) = parts.get(1) {
                if let Some(enemy_type) = EnemyType::from_key(entity) {
                    world.enemies.push(Enemy::new(
                        enemy_type,
                        screen_width() / 2.0,
                        -50.0,
                        world.wave_director.current_wave,
                    ));
                    history.push(format!("Spawned {entity}"));
                } else if let Some(boss_type) = BossType::from_key(entity) {
                    world.enemies.clear();
                    world.bullets.clear();
                    world.pirates.clear();
                    world.wave_director.current_boss = Some(Boss::new(boss_type));
                    world.wave_director.wave_state = WaveState::BossFight;
                    world.wave_director.is_boss_wave = true;
                    world.wave_director.current_wave = 99;
                    history.push(format!("Spawned boss: {}", boss_type.name()));
                    return Some(GameState::Playing);
                } else {
                    history.push(
                        "Unknown entity. Use: scout, fighter, bomber, interceptor, elite, or a boss name"
                            .to_string(),
                    );
                }
            } else {
                history.push("Usage: spawn <entity>".to_string());
            }
        }
        "killall" => {
            for enemy in world.enemies.iter_mut() {
                enemy.is_dead = true;
            }
            for pirate in world.pirates.iter_mut() {
                pirate.is_dead = true;
            }
            world.bullets.clear();
            history.push("All enemies killed".to_string());
        }
        "powerup" => {
            if let Some(kind) = parts.get(1) {
                let effect = match *kind {
                    "rapid" => config::PowerupType::RapidFire,
                    "spread" => config::PowerupType::SpreadShot,
                    "pierce" => config::PowerupType::Pierce,
                    "shield" => config::PowerupType::Shield,
                    "bomb" => config::PowerupType::Bomb,
                    "life" => config::PowerupType::Life,
                    "score" => config::PowerupType::Score,
                    _ => {
                        history.push(
                            "Unknown powerup. Use: rapid, spread, pierce, shield, bomb, life, score"
                                .to_string(),
                        );
                        return None;
                    }
                };
                world.wave_director.powerup_manager.apply_powerup(
                    effect,
                    &mut world.pirates,
                    &mut world.score,
                    &mut world.lives,
                );
                history.push(format!("Gave powerup: {effect:?}"));
            } else {
                history.push("Usage: powerup <type>".to_string());
            }
        }
        "hitbox" => {
            *debug_hitbox_visible = !*debug_hitbox_visible;
            history.push(if *debug_hitbox_visible {
                "Hitbox overlay ON".to_string()
            } else {
                "Hitbox overlay OFF".to_string()
            });
        }
        "time" => {
            if let Some(scale) = parts.get(1).and_then(|s| s.parse::<f32>().ok()) {
                *time_scale = scale.clamp(0.01, 10.0);
                history.push(format!("Time scale set to {time_scale}"));
            } else {
                history.push(format!("Current time scale: {time_scale}"));
            }
        }
        "state" => {
            let wd = &world.wave_director;
            history.push(format!(
                "Wave: {} | State: {:?} | Boss wave: {}",
                wd.current_wave, wd.wave_state, wd.is_boss_wave
            ));
            history.push(format!("Score: {} | Lives: {}", world.score, world.lives));
            history.push(format!(
                "Ship: ({:.0}, {:.0}) | Invuln: {:.1}s",
                world.ship.x, world.ship.y, world.ship.invuln_timer
            ));
            if let Some(boss) = &wd.current_boss {
                history.push(format!(
                    "Boss: {} | HP: {}/{} | Phase: {:?} | Invuln: {}",
                    boss.boss_type.name(),
                    boss.health,
                    boss.max_health,
                    boss.phase,
                    boss.invulnerable
                ));
            }
            let mgr = &wd.powerup_manager;
            history.push(format!(
                "Powerups: rapid_fire={:.1}s spread={:.1}s pierce={:.1}s shield={}",
                mgr.rapid_fire_timer, mgr.spread_shot_timer, mgr.pierce_timer, mgr.has_shield,
            ));
        }
        "damage" => {
            if let Some(boss) = &mut world.wave_director.current_boss {
                if let Some(dmg) = parts.get(1).and_then(|s| s.parse::<i32>().ok()) {
                    if dmg > 0 && boss.take_damage(dmg) {
                        world.score += 1000;
                        world.wave_director.wave_state = WaveState::BossDefeated;
                        history.push("Boss defeated!".to_string());
                    } else {
                        history.push(format!(
                            "Boss took {dmg} damage. HP: {}/{}",
                            boss.health, boss.max_health
                        ));
                    }
                } else {
                    history.push("Usage: damage <n>".to_string());
                }
            } else {
                history.push("No boss active".to_string());
            }
        }
        "reload" => match content::reload() {
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
        },
        "quit" => {
            history.push("Returning to main menu...".to_string());
            return Some(GameState::MainMenu);
        }
        _ => {
            history.push(format!("Unknown command: {cmd}. Type 'help' for list."));
        }
    }
    None
}

// ---------------------------------------------------------------------------
// State: boss intro / end screens
// ---------------------------------------------------------------------------

async fn update_boss_intro(
    game: &mut Game,
    world: &mut World,
    boss_type: BossType,
    portraits: &PortraitManager,
) {
    let title = format!("BOSS APPROACHING: {}", boss_type.name());
    let title_size = 60.0;
    let title_x =
        screen_width() * 0.5 - measure_text(&title, None, title_size as u16, 1.0).width * 0.5;
    draw_text(&title, title_x, screen_height() * 0.3, title_size, RED);

    let subtitle = "PRESS SPACE TO ENGAGE";
    let sub_size = 30.0;
    let sub_x =
        screen_width() * 0.5 - measure_text(subtitle, None, sub_size as u16, 1.0).width * 0.5;
    draw_text(subtitle, sub_x, screen_height() * 0.5, sub_size, YELLOW);

    let character = match boss_type {
        BossType::Blowfish => Character::Blowfish,
        BossType::Twofish => Character::Twofish,
        BossType::RufusReverse => Character::RufusReverse,
        BossType::MollyHashpass => Character::MollyHashpass,
        BossType::CaptainDavey => Character::CaptainDavey,
        BossType::Deadbeef => Character::Deadbeef,
    };
    if let Some(portrait) = portraits.get_character(&character) {
        let size = 200.0;
        draw_texture_ex(
            portrait,
            screen_width() * 0.5 - size * 0.5,
            screen_height() * 0.55,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(size, size)),
                ..Default::default()
            },
        );
    }

    if is_key_pressed(KeyCode::Space) {
        let mut boss = Boss::new(boss_type);
        boss.load_assets().await;
        world.wave_director.current_boss = Some(boss);
        world.wave_director.wave_state = WaveState::BossFight;
        game.state = GameState::Playing;
        audio::play_music("boss_music");
    }
}

fn update_end_screen(game: &mut Game, world: &mut World) {
    if is_key_pressed(KeyCode::Space) {
        world.reset();
        game.narrative = NarrativeProgress::new();
        begin_wave(1, game, world);
        audio::play_music("gameplay_music");
    }
    if is_key_pressed(KeyCode::Escape) {
        game.state = GameState::MainMenu;
        audio::play_music("menu_music");
    }
}

// ---------------------------------------------------------------------------
// Collisions
// ---------------------------------------------------------------------------

fn check_collisions(world: &mut World) {
    let World {
        ship,
        enemies,
        bullets,
        pirates,
        wave_director,
        score,
        lives,
        ..
    } = world;

    // Player bullets vs enemies. hit_flash debounces damage so piercing
    // bullets pass through instead of draining one target per frame.
    for bullet in bullets.iter_mut() {
        if bullet.is_dead || !bullet.bullet_type.is_player() || !bullet.can_damage() {
            continue;
        }
        for enemy in enemies.iter_mut() {
            if enemy.is_dead {
                continue;
            }
            if rects_overlap(bullet.get_rect(), enemy.get_rect()) {
                if enemy.take_damage(bullet.damage) {
                    *score += enemy.score_value;
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

    // Player bullets vs pirate minions.
    for bullet in bullets.iter_mut() {
        if bullet.is_dead || !bullet.bullet_type.is_player() || !bullet.can_damage() {
            continue;
        }
        for pirate in pirates.iter_mut() {
            if pirate.is_dead {
                continue;
            }
            if rects_overlap(bullet.get_rect(), pirate.get_rect()) {
                pirate.is_dead = true;
                *score += 25;
                if !bullet.pierce() {
                    bullet.is_dead = true;
                }
                bullet.hit_flash = 0.1;
                break;
            }
        }
    }

    // Enemy bullets vs ship. Player bullets are structurally excluded —
    // never rely on spawn geometry for that.
    let mut ship_hit = false;
    if ship.invuln_timer <= 0.0 {
        for bullet in bullets.iter_mut() {
            if bullet.is_dead || bullet.bullet_type.is_player() {
                continue;
            }
            if rects_overlap(bullet.get_rect(), ship.get_rect()) {
                bullet.is_dead = true;
                ship_hit = true;
                break;
            }
        }
    }

    // Pirate minions vs ship: the pirate crashes and burns, and the player
    // gets invulnerability frames instead of losing a life on every
    // overlapping frame.
    if !ship_hit && ship.invuln_timer <= 0.0 {
        for pirate in pirates.iter_mut() {
            if pirate.is_dead {
                continue;
            }
            if rects_overlap(pirate.get_rect(), ship.get_rect()) {
                pirate.is_dead = true;
                ship_hit = true;
                break;
            }
        }
    }

    if ship_hit {
        if wave_director.powerup_manager.has_shield {
            wave_director.powerup_manager.consume_shield();
            ship.invuln_timer = 1.0;
        } else if *lives > 0 {
            *lives -= 1;
            ship.invuln_timer = 2.0;
        } else {
            ship.gameover = true;
        }
    }

    enemies.retain(|e| !e.is_dead);
    pirates.retain(|p| !p.is_dead);
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

/// Draws the (possibly frozen) game world. Used by every in-game state so
/// the previously copy-pasted render blocks exist exactly once.
fn draw_world(world: &World, game: &Game, assets: &Assets) {
    draw_texture(&assets.background, 0.0, 0.0, WHITE);

    for pirate in &world.pirates {
        pirate.draw(&assets.pirate);
    }
    for enemy in &world.enemies {
        let tex = assets.enemies.get(enemy.enemy_type.sprite_name());
        enemy.draw(tex);
    }
    for bullet in &world.bullets {
        let tex = assets.bullets.get(bullet.bullet_type.sprite_name());
        bullet.draw(tex);
    }

    world.wave_director.powerup_manager.draw(&assets.powerups);

    if let Some(boss) = world.wave_director.get_current_boss() {
        boss.draw();
    }

    world
        .ship
        .draw(&assets.ship, world.wave_director.powerup_manager.has_shield);

    if game.debug_hitbox_visible {
        draw_debug_hitboxes(world);
    }

    draw_ui(world, game);
    world.wave_director.draw_wave_info();

    if world.wave_director.wave_state == WaveState::BossFight {
        if let Some(boss) = world.wave_director.get_current_boss() {
            draw_boss_health_bar(boss);
        }
    }

    world.wave_director.powerup_manager.draw_effect_indicators();

    if cfg!(debug_assertions) {
        draw_text(
            "DEBUG MODE",
            screen_width() - 120.0,
            screen_height() - 10.0,
            16.0,
            Color::new(0.5, 0.5, 0.5, 1.0),
        );
    }
}

fn draw_debug_hitboxes(world: &World) {
    for enemy in &world.enemies {
        if !enemy.is_dead {
            let (x, y, w, h) = enemy.get_rect();
            draw_rectangle_lines(x, y, w, h, 1.0, RED);
        }
    }
    for bullet in &world.bullets {
        if !bullet.is_dead {
            let (x, y, w, h) = bullet.get_rect();
            draw_rectangle_lines(x, y, w, h, 1.0, YELLOW);
        }
    }
    for pirate in &world.pirates {
        if !pirate.is_dead {
            let (x, y, w, h) = pirate.get_rect();
            draw_rectangle_lines(x, y, w, h, 1.0, RED);
        }
    }
    let (sx, sy, sw, sh) = world.ship.get_rect();
    draw_rectangle_lines(sx, sy, sw, sh, 1.0, GREEN);
    if let Some(boss) = world.wave_director.get_current_boss() {
        let (bx, by, bw, bh) = boss.get_rect();
        draw_rectangle_lines(bx, by, bw, bh, 1.0, RED);
        for p in boss.get_projectiles() {
            draw_circle_lines(p.x, p.y, p.size / 2.0, 1.0, RED);
        }
    }
}

fn draw_ui(world: &World, game: &Game) {
    draw_text(format!("Score: {}", world.score), 25.0, 25.0, 25.0, WHITE);
    draw_text(format!("Lives: {}", world.lives), 25.0, 55.0, 25.0, WHITE);
    draw_text(
        format!("Wave: {}", world.wave_director.current_wave),
        25.0,
        85.0,
        25.0,
        WHITE,
    );

    if game.narrative.current_chapter > 0 {
        draw_text(
            narrative::get_chapter_name(game.narrative.current_chapter),
            25.0,
            115.0,
            20.0,
            GOLD,
        );
    }
}

fn draw_game_over_screen(world: &World) {
    draw_text(
        "GAME OVER",
        screen_width() * 0.5 - 150.0,
        screen_height() * 0.4,
        100.0,
        RED,
    );
    draw_text(
        format!("Final Score: {}", world.score),
        screen_width() * 0.5 - 90.0,
        screen_height() * 0.5,
        36.0,
        YELLOW,
    );
    draw_text(
        "Press SPACE to play again",
        screen_width() * 0.5 - 150.0,
        screen_height() * 0.6,
        28.0,
        WHITE,
    );
    draw_text(
        "Press ESC for Main Menu",
        screen_width() * 0.5 - 140.0,
        screen_height() * 0.68,
        28.0,
        GRAY,
    );
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

fn draw_victory_screen(world: &World) {
    draw_text(
        "VICTORY!",
        screen_width() * 0.5 - 120.0,
        screen_height() * 0.3,
        100.0,
        GOLD,
    );
    draw_text(
        format!("Final Score: {}", world.score),
        screen_width() * 0.5 - 90.0,
        screen_height() * 0.45,
        36.0,
        YELLOW,
    );
    draw_text(
        "You defeated Captain Davey Portscan!",
        screen_width() * 0.5 - 200.0,
        screen_height() * 0.55,
        28.0,
        WHITE,
    );
    draw_text(
        "Press SPACE to play again",
        screen_width() * 0.5 - 150.0,
        screen_height() * 0.65,
        28.0,
        WHITE,
    );
    draw_text(
        "Press ESC for Main Menu",
        screen_width() * 0.5 - 140.0,
        screen_height() * 0.72,
        28.0,
        GRAY,
    );
}

fn boss_for_wave(wave: u32) -> BossType {
    content::boss_for_wave(wave)
        .and_then(BossType::from_key)
        .unwrap_or(BossType::Blowfish)
}
