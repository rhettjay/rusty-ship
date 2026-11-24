use crate::config::CONFIG;
use ::rand::Rng;
use macroquad::prelude::*;

pub const PIRATE_ROTATION: f32 = std::f32::consts::PI;

/// Pirate minions, spawned by boss attacks (Captain Davey's crew, Molly's
/// revived minions). They drift down, bounce off the screen edges, and hurt
/// the player on contact.
pub struct Pirate {
    pub x: f32,
    pub y: f32,
    pub speed_x: f32,
    pub speed_y: f32,
    pub color: Color,
    pub is_dead: bool,
    pub width: f32,
    pub height: f32,
}

impl Pirate {
    pub fn new(x: f32, y: f32, speed_x: f32) -> Self {
        let mut rng = ::rand::thread_rng();
        Self {
            x,
            y,
            speed_x,
            speed_y: rng.gen_range(60.0..180.0),
            color: WHITE,
            is_dead: false,
            width: CONFIG.pirate_width,
            height: CONFIG.pirate_height,
        }
    }

    /// `dt` in seconds; speeds are pixels per second.
    pub fn update(&mut self, dt: f32) {
        self.x += self.speed_x * dt;
        self.y += self.speed_y * dt;

        // Bounce off the side walls.
        if self.x > screen_width() - self.width {
            self.speed_x = -self.speed_x.abs();
        }
        if self.x < 0.0 {
            self.speed_x = self.speed_x.abs();
        }
    }

    pub fn draw(&self, texture: &Texture2D) {
        draw_texture_ex(
            texture,
            self.x,
            self.y,
            self.color,
            DrawTextureParams {
                dest_size: Some(Vec2::new(self.width, self.height)),
                rotation: PIRATE_ROTATION,
                ..Default::default()
            },
        );
    }

    pub fn get_rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.width, self.height)
    }
}
