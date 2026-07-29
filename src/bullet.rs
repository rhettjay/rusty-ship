use macroquad::prelude::*;
use ::rand::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BulletType {
    PlayerStandard,
    PlayerRapid,
    PlayerSpread,
    PlayerPierce,
    PlayerLaser,
    EnemyStraight,
    EnemyAimed,
    EnemyBomb,
    EnemySpread,
    EnemyFast,
    EnemyLaser,
}

impl BulletType {
    pub fn color(&self) -> Color {
        match self {
            BulletType::PlayerStandard => YELLOW,
            BulletType::PlayerRapid => GOLD,
            BulletType::PlayerSpread => SKYBLUE,
            BulletType::PlayerPierce => LIME,
            BulletType::PlayerLaser => RED,
            BulletType::EnemyStraight => ORANGE,
            BulletType::EnemyAimed => RED,
            BulletType::EnemyBomb => PURPLE,
            BulletType::EnemySpread => PINK,
            BulletType::EnemyFast => WHITE,
            BulletType::EnemyLaser => Color::new(1.0, 0.0, 0.5, 1.0),
        }
    }

    pub fn size(&self) -> (f32, f32) {
        match self {
            BulletType::PlayerStandard => (6.0, 14.0),
            BulletType::PlayerRapid => (5.0, 12.0),
            BulletType::PlayerSpread => (7.0, 14.0),
            BulletType::PlayerPierce => (6.0, 16.0),
            BulletType::PlayerLaser => (4.0, 32.0),
            BulletType::EnemyStraight => (10.0, 10.0),
            BulletType::EnemyAimed => (10.0, 10.0),
            BulletType::EnemyBomb => (16.0, 16.0),
            BulletType::EnemySpread => (10.0, 10.0),
            BulletType::EnemyFast => (8.0, 8.0),
            BulletType::EnemyLaser => (4.0, 32.0),
        }
    }

    pub fn speed(&self) -> f32 {
        // Speed in pixels per frame (original values)
        match self {
            BulletType::PlayerStandard => 12.0,
            BulletType::PlayerRapid => 14.0,
            BulletType::PlayerSpread => 10.0,
            BulletType::PlayerPierce => 11.0,
            BulletType::PlayerLaser => 0.0,
            BulletType::EnemyStraight => 5.0,
            BulletType::EnemyAimed => 6.0,
            BulletType::EnemyBomb => 3.0,
            BulletType::EnemySpread => 5.0,
            BulletType::EnemyFast => 8.0,
            BulletType::EnemyLaser => 0.0,
        }
    }

    pub fn damage(&self) -> i32 {
        match self {
            BulletType::PlayerStandard => 1,
            BulletType::PlayerRapid => 1,
            BulletType::PlayerSpread => 1,
            BulletType::PlayerPierce => 1,
            BulletType::PlayerLaser => 2,
            BulletType::EnemyStraight => 1,
            BulletType::EnemyAimed => 1,
            BulletType::EnemyBomb => 2,
            BulletType::EnemySpread => 1,
            BulletType::EnemyFast => 1,
            BulletType::EnemyLaser => 3,
        }
    }

    pub fn is_player(&self) -> bool {
        matches!(self,
            BulletType::PlayerStandard
            | BulletType::PlayerRapid
            | BulletType::PlayerSpread
            | BulletType::PlayerPierce
            | BulletType::PlayerLaser
        )
    }

    pub fn is_laser(&self) -> bool {
        matches!(self, BulletType::PlayerLaser | BulletType::EnemyLaser)
    }

    pub fn pierce_count(&self) -> i32 {
        match self {
            BulletType::PlayerPierce => 3,
            BulletType::PlayerLaser => 999,
            BulletType::EnemyLaser => 999,
            _ => 0,
        }
    }

    pub fn sprite_name(&self) -> &'static str {
        match self {
            BulletType::PlayerStandard => "bullet_player_1",
            BulletType::PlayerRapid => "bullet_player_1",
            BulletType::PlayerSpread => "bullet_player_2",
            BulletType::PlayerPierce => "bullet_player_3",
            BulletType::PlayerLaser => "bullet_laser",
            BulletType::EnemyStraight => "bullet_enemy_1",
            BulletType::EnemyAimed => "bullet_enemy_2",
            BulletType::EnemyBomb => "bullet_bomb",
            BulletType::EnemySpread => "bullet_enemy_3",
            BulletType::EnemyFast => "bullet_enemy_4",
            BulletType::EnemyLaser => "bullet_laser_enemy",
        }
    }
}

pub struct Bullet {
    pub bullet_type: BulletType,
    pub x: f32,
    pub y: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub width: f32,
    pub height: f32,
    pub damage: i32,
    pub pierce_remaining: i32,
    pub lifetime: f64,
    pub is_dead: bool,
    pub is_laser: bool,
    pub laser_target_y: f32,
    pub laser_warmup: f64,
    pub rotation: f32,
    pub hit_flash: f64,
}

impl Bullet {
    pub fn new(bullet_type: BulletType, x: f32, y: f32, vel_x: f32, vel_y: f32) -> Self {
        let (width, height) = bullet_type.size();
        let is_laser = bullet_type.is_laser();
        
        Self {
            bullet_type,
            x,
            y,
            vel_x,
            vel_y,
            width,
            height,
            damage: bullet_type.damage(),
            pierce_remaining: bullet_type.pierce_count(),
            lifetime: if is_laser { 1.0 } else { 5.0 },
            is_dead: false,
            is_laser,
            laser_target_y: if bullet_type.is_player() { 0.0 } else { screen_height() },
            laser_warmup: if is_laser { 0.3 } else { 0.0 },
            rotation: vel_y.atan2(vel_x) + std::f32::consts::FRAC_PI_2,
            hit_flash: 0.0,
        }
    }

    pub fn new_laser(bullet_type: BulletType, x: f32, y: f32, target_y: f32, is_player: bool) -> Self {
        let (width, _height) = bullet_type.size();
        Self {
            bullet_type,
            x,
            y,
            vel_x: 0.0,
            vel_y: 0.0,
            width,
            height: if is_player { y - target_y } else { target_y - y },
            damage: bullet_type.damage(),
            pierce_remaining: bullet_type.pierce_count(),
            lifetime: 0.5,
            is_dead: false,
            is_laser: true,
            laser_target_y: target_y,
            laser_warmup: 0.2,
            rotation: 0.0,
            hit_flash: 0.0,
        }
    }

    pub fn update(&mut self, dt: f64) {
        if self.hit_flash > 0.0 {
            self.hit_flash -= dt;
        }

        if self.is_laser {
            self.laser_warmup -= dt;
            self.lifetime -= dt;
            if self.lifetime <= 0.0 {
                self.is_dead = true;
            }
            return;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;
        self.lifetime -= dt;

        if self.lifetime <= 0.0
            || self.y < -50.0
            || self.y > screen_height() + 50.0
            || self.x < -50.0
            || self.x > screen_width() + 50.0 {
            self.is_dead = true;
        }

        if self.bullet_type == BulletType::EnemyBomb {
            self.vel_y += 0.15 * dt as f32;
        }
    }

    pub fn draw(&self, texture: Option<&Texture2D>) {
        let color = self.bullet_type.color();
        
        if self.hit_flash > 0.0 {
            let flash_color = WHITE;
            self.draw_shape(flash_color, texture);
        } else {
            self.draw_shape(color, texture);
        }
    }

    fn draw_shape(&self, color: Color, texture: Option<&Texture2D>) {
        if self.is_laser {
            self.draw_laser(color, texture);
            return;
        }

        if let Some(tex) = texture {
            draw_texture_ex(
                tex,
                self.x - self.width / 2.0,
                self.y - self.height / 2.0,
                color,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(self.width, self.height)),
                    rotation: self.rotation,
                    ..Default::default()
                }
            );
        } else {
            self.draw_procedural(color);
        }
    }

    fn draw_procedural(&self, color: Color) {
        match self.bullet_type {
            BulletType::PlayerStandard | BulletType::PlayerRapid | BulletType::PlayerPierce => {
                draw_rectangle(
                    self.x - self.width / 2.0,
                    self.y - self.height / 2.0,
                    self.width,
                    self.height,
                    color,
                );
                draw_rectangle(
                    self.x - 1.0,
                    self.y - self.height / 2.0,
                    2.0,
                    self.height,
                    WHITE,
                );
            }
            BulletType::PlayerSpread => {
                draw_rectangle(
                    self.x - self.width / 2.0,
                    self.y - self.height / 2.0,
                    self.width,
                    self.height,
                    color,
                );
            }
            BulletType::EnemyStraight | BulletType::EnemyAimed | BulletType::EnemyFast | BulletType::EnemySpread => {
                draw_circle(self.x, self.y, self.width / 2.0, color);
                draw_circle(self.x, self.y, self.width / 3.0, WHITE);
            }
            BulletType::EnemyBomb => {
                draw_circle(self.x, self.y, self.width / 2.0, color);
                draw_circle(self.x, self.y, self.width / 2.0 - 3.0, Color::new(0.5, 0.0, 0.5, 1.0));
            }
            _ => {
                draw_rectangle(
                    self.x - self.width / 2.0,
                    self.y - self.height / 2.0,
                    self.width,
                    self.height,
                    color,
                );
            }
        }
    }

    fn draw_laser(&self, color: Color, _texture: Option<&Texture2D>) {
        let laser_color = Color::new(color.r, color.g, color.b, 0.7);
        
        if self.bullet_type.is_player() {
            draw_rectangle(
                self.x - self.width / 2.0,
                self.laser_target_y,
                self.width,
                self.y - self.laser_target_y,
                laser_color,
            );
            draw_rectangle(
                self.x - 1.0,
                self.laser_target_y,
                2.0,
                self.y - self.laser_target_y,
                WHITE,
            );
        } else {
            draw_rectangle(
                self.x - self.width / 2.0,
                self.y,
                self.width,
                self.laser_target_y - self.y,
                laser_color,
            );
            draw_rectangle(
                self.x - 1.0,
                self.y,
                2.0,
                self.laser_target_y - self.y,
                WHITE,
            );
        }
    }

    pub fn get_rect(&self) -> (f32, f32, f32, f32) {
        if self.is_laser {
            if self.bullet_type.is_player() {
                return (
                    self.x - self.width / 2.0,
                    self.laser_target_y,
                    self.width,
                    self.y - self.laser_target_y,
                );
            } else {
                return (
                    self.x - self.width / 2.0,
                    self.y,
                    self.width,
                    self.laser_target_y - self.y,
                );
            }
        }
        (
            self.x - self.width / 2.0,
            self.y - self.height / 2.0,
            self.width,
            self.height,
        )
    }

    pub fn pierce(&mut self) -> bool {
        if self.pierce_remaining > 0 {
            self.pierce_remaining -= 1;
            return self.pierce_remaining >= 0;
        }
        false
    }
}

pub fn create_spread_bullets(bullet_type: BulletType, x: f32, y: f32, base_vel_x: f32, base_vel_y: f32, count: i32, spread_angle: f32) -> Vec<Bullet> {
    let mut bullets = Vec::new();
    let angle_step = spread_angle / (count - 1).max(1) as f32;
    let start_angle = -spread_angle / 2.0;
    
    for i in 0..count {
        let angle = start_angle + i as f32 * angle_step;
        let vel_x = base_vel_x * angle.cos() - base_vel_y * angle.sin();
        let vel_y = base_vel_x * angle.sin() + base_vel_y * angle.cos();
        bullets.push(Bullet::new(bullet_type, x, y, vel_x, vel_y));
    }
    bullets
}

pub fn create_bomb_bullet(x: f32, y: f32, target_x: f32) -> Bullet {
    let dx = target_x - x;
    let dist = dx.abs().max(1.0);
    let vel_x = dx / dist * 1.5;
    Bullet::new(BulletType::EnemyBomb, x, y, vel_x, 2.0)
}