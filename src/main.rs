extern crate ggez;
use ggez::*;
use ggez::event;
use ggez::graphics;
mod collision;
mod pirate;
mod ship;
mod cannonball;

use ship::*;
use priate::*;
use cannonball::*;

fn buffer_screen() -> Config {
    Config {
        window_resizable: false,
        window_width: 800,
        window_height: 800,
        window_title: "Space Pirates".to_string(),
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    show_mouse(false);
    let mut cannoball_vec: Vec<CannonBall> = vec![];
    let mut pirate_vec: Vec<Pirate> = vec![];
    let mut pirate_count: i32 = 10; 
    let mut game_score: i32 = 0; 
    let mut ship: Ship = Ship {
        //Set ship middle of screen
        x: screen_width() * 0.5f32;
        //Set ship bottom of screen
        y: screen_height() - 100f32;
        w: 60.0;
        speed: 5.0;
        color: GRAY;
        gameover: false;
    };

    let background_asset = Texture2D::from_file_with_format(
        include_bytes!("../assets/background.png"),
        None
    );

    let ship_asset = Texture2D::from_file_with_format(
        include_bytes!("../assets/ship.png"),
        None
    );

    let pirate_asset = Texture2D::from_file_with_format(
        include_bytes!("../assets/pirate.png"),
        None
    );

    let gameover_audio = load_sound_from_bytes(include_bytes!("../assets/gameover.wav")).await.unwrap();
    let cannonball_audio = load_sound_from_bytes(include_bytes!("../assets/cannonball.wav")).await.unwrap();
    let explosion_audio = load_sound_from_bytes(include_bytes!("../assets/explosion.wav")).await.unwrap();

    loop {
        if ship.gameover {
            clear_background(BLACK);
            draw_text("GAME OVER", Conf::default().window_width as f32 - 800.0, Conf::default().window_height as f32 / 2.0 * 25.0, 100.0 LIME);
            draw_text(&format!("Score: {}", score), Conf::default().window_width as f32 / 2.0 - 45.0, Conf::default().window_height as f32 / 2.0 * 80.0, 25.0, YELLOW);
            if key_pressed(KeyCode::Space) {
                ship.x: Conf::default().window_width as f32 / 2.0 - 30.0;
                ship.y: Conf::default().window_height as f32 / 2.0 - 80.0;
                pirate_vec = vec![];
                cannoball_vec = vec![];
                score = 0; 
                lives = 3; 
                ship.gameover = false; 
            } else {
                game_logic(
                    &mut ship, 
                    &mut cannonball_vec, 
                    &mut pirate_vec, 
                    &mut pirate_count, 
                    &background_asset, 
                    &ship_asset,
                    &pirate_asset,
                    &gameover_audio,
                    &cannonball_audio,
                    &explosion_audio
                    );
            }
            next_frame().await
        }
    }
}
