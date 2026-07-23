use macroquad::audio;
use macroquad::audio::play_sound_once;
use macroquad::prelude::*;
use ::rand::Rng;
use crate::ship::*;
use crate::cannonball::*;
use crate::pirate::*;
use crate::config::CONFIG;

pub fn run(
    ship: &mut Ship,
    cannonball_vec: &mut Vec<Cannonball>,
    pirate_vec: &mut Vec<Pirate>,
    pirate_count: &mut i32,
    score: &mut i32,
    lives: &mut i32,
    background_asset: &Texture2D,
    ship_sprite: &Texture2D,
    pirate_sprite: &Texture2D,
    cannon_audio: &audio::Sound,
    explosion_audio: &audio::Sound,
    pirate_death_audio: &audio::Sound,
    gameover_audio: &audio::Sound
) {
    draw_texture(background_asset, 0.0, 0.0, WHITE);
    ship.draw(ship_sprite);

    let mut rng = ::rand::thread_rng();
    if rng.gen_range(0..CONFIG.pirate_spawn_chance) == 0 {
        if *pirate_count > pirate_vec.len() as i32 {
            let x = rng.gen_range(CONFIG.ship_width..screen_width() - CONFIG.ship_width);
            let speed_x = rng.gen_range(CONFIG.pirate_base_speed_x_range.0..CONFIG.pirate_base_speed_x_range.1);
            pirate_vec.push(Pirate::new(x, 20.0, speed_x));
            *pirate_count += 1;
        }
    }

    if ship.x > screen_width() - ship.w {
        ship.x = screen_width() - ship.w;
    }
    if ship.x < 0.0 {
        ship.x = 0.0;
    }

    for cannonball in cannonball_vec.iter_mut() {
        cannonball.update();
        cannonball.draw();
    }
    cannonball_vec.retain(|c| c.y > 0.0);

    for pirate in pirate_vec.iter_mut() {
        pirate.update();
        pirate.draw(pirate_sprite);

        if pirate.x > screen_width() - CONFIG.pirate_width {
            pirate.speed_x = -pirate.speed_x.abs();
        }
        if pirate.x < 0.0 {
            pirate.speed_x = pirate.speed_x.abs();
        }
    }

    for cannonball in cannonball_vec.iter_mut() {
        for pirate in pirate_vec.iter_mut() {
            if pirate.x < cannonball.x + CONFIG.cannonball_width
                && pirate.x + CONFIG.pirate_width > cannonball.x
                && pirate.y < cannonball.y + CONFIG.cannonball_height
                && pirate.y + CONFIG.pirate_height > cannonball.y {
                play_sound_once(pirate_death_audio);
                pirate.is_dead = true;
                *score += 1;
            }
        }
    }

    for pirate in pirate_vec.iter_mut() {
        if ship.x < pirate.x + CONFIG.pirate_width
            && ship.x + ship.w > pirate.x
            && ship.y < pirate.y + CONFIG.pirate_height
            && ship.y + CONFIG.ship_height > pirate.y {
            if *lives > 0 {
                *lives -= 1;
            } else {
                ship.gameover = true;
                play_sound_once(gameover_audio);
                break;
            }
        }
    }

    pirate_vec.retain(|p| p.y < screen_height() && !p.is_dead);
}
