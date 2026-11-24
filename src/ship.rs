use crate::config::CONFIG;
use macroquad::prelude::*;

/// Player ship.
///
/// Powerup effect state (rapid fire / spread / pierce / shield) deliberately
/// does NOT live here: `PowerUpManager` is the single source of truth. The
/// ship only owns position, motion, and damage state.
pub struct Ship {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    /// Pixels per second.
    pub speed: f32,
    pub color: Color,
    pub gameover: bool,
    pub invuln_timer: f32,
}

impl Ship {
    pub fn new() -> Self {
        Self {
            x: screen_width() * 0.5 - CONFIG.ship_width * 0.5,
            y: screen_height() - CONFIG.ship_start_y_offset,
            w: CONFIG.ship_width,
            h: CONFIG.ship_height,
            speed: CONFIG.ship_speed,
            color: GRAY,
            gameover: false,
            invuln_timer: 0.0,
        }
    }

    /// `reversed` inverts the controls (Rufus Reverse's signature attack).
    /// The ship navigates in both dimensions.
    pub fn update_input(&mut self, dt: f32, reversed: bool) {
        let left = is_key_down(KeyCode::Left);
        let right = is_key_down(KeyCode::Right);
        let up = is_key_down(KeyCode::Up);
        let down = is_key_down(KeyCode::Down);
        let (left, right) = if reversed {
            (right, left)
        } else {
            (left, right)
        };
        let (up, down) = if reversed { (down, up) } else { (up, down) };

        if left {
            self.x -= self.speed * dt;
        }
        if right {
            self.x += self.speed * dt;
        }
        if up {
            self.y -= self.speed * dt;
        }
        if down {
            self.y += self.speed * dt;
        }
        self.x = self.x.clamp(0.0, screen_width() - self.w);
        // Keep the player below the boss patrol band (bosses live around
        // y 50-160); tune the margin if that band ever changes.
        self.y = self.y.clamp(120.0, screen_height() - self.h);
    }

    pub fn update(&mut self, dt: f32) {
        if self.invuln_timer > 0.0 {
            self.invuln_timer -= dt;
        }
    }

    pub fn draw(&self, texture: &Texture2D, has_shield: bool) {
        // Blink while invulnerable.
        if self.invuln_timer > 0.0 && (self.invuln_timer * 8.0).sin() > 0.0 {
            return;
        }
        // Draw scaled to the collision rect (w x h), NOT at the texture's
        // native size — otherwise the sprite (32x32) sits in the corner of
        // the rect (60x64) and the shield circles the wrong center.
        draw_texture_ex(
            texture,
            self.x,
            self.y,
            self.color,
            DrawTextureParams {
                dest_size: Some(Vec2::new(self.w, self.h)),
                ..Default::default()
            },
        );
        if has_shield {
            draw_circle_lines(
                self.x + self.w / 2.0,
                self.y + self.h / 2.0,
                self.w.max(self.h) * 0.7,
                2.0,
                Color::new(0.5, 0.8, 1.0, 0.8),
            );
        }
    }

    pub fn get_rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.w, self.h)
    }

    pub fn reset_position(&mut self) {
        self.x = screen_width() * 0.5 - self.w * 0.5;
        self.y = screen_height() - CONFIG.ship_start_y_offset;
        self.gameover = false;
        self.invuln_timer = 0.0;
    }
}
