use macroquad::audio;
use macroquad::audio::play_sound_once;
use macroquad::prelude::*;
use crate::ship::*;
use crate::cannonball::*;
use crate::pirate::*;


fn score_show(score: &i32) {
    draw_text(
        &format!("Score: {}", score)[..],
        screen_height(),
        25.0,
        25.0,
        WHITE
    );
}

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
    pirate_death_audio: &audio::Sound,
    gameover_audio: &audio::Sound
    ) {
    bg_draw(&background_asset);
    ship.draw(&ship_sprite);
    score_show(&score);
    if is_key_down(KeyCode::LeftArrow) {
        ship.left();
    }
    if is_key_down(KeyCode::RightArrow) {
        ship.right();
    }
    if is_key_down(KeyCode::Space) {
        cannonball_vec.append(&mut vec![Cannonball::new(ship.x + ship.w * 0.5, ship.y - 15.0, 10.0, WHITE, true)]);
        play_sound_once(*cannon_audio);
    }
    if rand::rand() as i32 % 25 == 0 {
        if pirate_count > &mut (pirate_vec.len() as i32) {
         pirate_vec.append(&mut vec![Pirate::new(rand::gen_range(0.0 + ship.w, Conf::default().screen_width as f32 - ship.w), 20.0, rand::gen_range(1.0, 8.0))])
         ++*pirate_count;
        }
    }

    //Keep ship from going off the board
    if ship.x > screen_width() - ship.w {
        ship.x = screen_width() - ship.w
    }

    //Keep ship from going out of bounds
    if ship.x < 0.0 {
        ship.x = 0.0
    }
    
    for cannonball in cannonball_vec.iter_mut() {
        if cannonball.is_ready {
            //TODO set a timeout here so you can't just pound the cannon balls
            cannonball.fire();
        }
        cannonball.update();
        cannonball.draw();
        if cannonball.y < 0.0 {
            cannonball.ready();
        }
    }

    for pirate in pirate_vec.iter_mut() {
        pirate.update();
        pirate.draw(&pirate_sprite);
        if pirate.x > screen_height() - 15.0 {
            pirate.speed_x -= pirate_speed_x;
        }
        if pirate.x < 0.0 {
            pirate.speed_x -= pirate.speed_x;
        }
    }

    for cannonball in cannonball_vec.iter_mut() {
        for pirate in pirate_vec.iter_mut() {
            if pirate.x < cannonball.x + 10.0 && pirate.x + 15.0 > cannonball.x && pirate.y < cannonball.y + 10.0 && pirate.y + 15.0 > cannonball.y {
                play_sound_once(*pirate_death_audio);
                pirate.is_dead = true;
                ++*score;
            }
        }
    }

    for pirate in  pirate_vec.iter_mut() {
        if ship.x < pirate.x + 15.0 && player.x + 64.0 > pirate.x && ship.y < pirate.y + 15.0 && ship.y + 64.0 > pirate.y {
            if (lives > 0) {
                --*lives;
            } else {
                ship.gameover = true;
                play_sound_once(*gameover_audio);
                break;
            }
        }
    }
    cannonball_vec.retain(|x| x.y > 0.0);
    pirate_vec.retain(|x| x.y < Conf::default().window_height as f32);
    pirate_vec.retain(|x| !x.is_dead);
}
