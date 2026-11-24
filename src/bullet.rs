use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BulletType {
    PlayerStandard,
    PlayerSpread,
    PlayerPierce,
    EnemyStraight,
    EnemyAimed,
    EnemyBomb,
    EnemySpread,
}

impl BulletType {
    pub fn color(&self) -> Color {
        match self {
            BulletType::PlayerStandard => YELLOW,
            BulletType::PlayerSpread => SKYBLUE,
            BulletType::PlayerPierce => LIME,
            BulletType::EnemyStraight => ORANGE,
            BulletType::EnemyAimed => RED,
            BulletType::EnemyBomb => PURPLE,
            BulletType::EnemySpread => PINK,
        }
    }

    pub fn size(&self) -> (f32, f32) {
        match self {
            BulletType::PlayerStandard => (6.0, 14.0),
            BulletType::PlayerSpread => (7.0, 14.0),
            BulletType::PlayerPierce => (6.0, 16.0),
            BulletType::EnemyStraight => (10.0, 10.0),
            BulletType::EnemyAimed => (10.0, 10.0),
            BulletType::EnemyBomb => (16.0, 16.0),
            BulletType::EnemySpread => (10.0, 10.0),
        }
    }

    /// Pixels per second.
    pub fn speed(&self) -> f32 {
        match self {
            BulletType::PlayerStandard => 720.0,
            BulletType::PlayerSpread => 600.0,
            BulletType::PlayerPierce => 660.0,
            BulletType::EnemyStraight => 300.0,
            BulletType::EnemyAimed => 360.0,
            BulletType::EnemyBomb => 180.0,
            BulletType::EnemySpread => 300.0,
        }
    }

    pub fn damage(&self) -> i32 {
        match self {
            BulletType::PlayerStandard => 1,
            BulletType::PlayerSpread => 1,
            BulletType::PlayerPierce => 1,
            BulletType::EnemyStraight => 1,
            BulletType::EnemyAimed => 1,
            BulletType::EnemyBomb => 2,
            BulletType::EnemySpread => 1,
        }
    }

    pub fn is_player(&self) -> bool {
        matches!(
            self,
            BulletType::PlayerStandard | BulletType::PlayerSpread | BulletType::PlayerPierce
        )
    }

    /// How many enemies this bullet can pass through after the first hit.
    pub fn pierce_count(&self) -> i32 {
        match self {
            BulletType::PlayerPierce => 3,
            _ => 0,
        }
    }

    /// Asset file stem under assets/bullets/.
    pub fn sprite_name(&self) -> &'static str {
        match self {
            BulletType::PlayerStandard => "player_standard",
            BulletType::PlayerSpread => "player_spread",
            BulletType::PlayerPierce => "player_pierce",
            BulletType::EnemyStraight => "enemy_straight",
            BulletType::EnemyAimed => "enemy_aimed",
            BulletType::EnemyBomb => "enemy_bomb",
            BulletType::EnemySpread => "enemy_spread",
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
    pub rotation: f32,
    /// Positive for a short window after a hit. Doubles as the render flash
    /// and the damage-debounce gate so piercing bullets cannot damage the
    /// same target every frame they overlap it.
    pub hit_flash: f64,
}

impl Bullet {
    pub fn new(bullet_type: BulletType, x: f32, y: f32, vel_x: f32, vel_y: f32) -> Self {
        let (width, height) = bullet_type.size();

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
            lifetime: 5.0,
            is_dead: false,
            rotation: vel_y.atan2(vel_x) + std::f32::consts::FRAC_PI_2,
            hit_flash: 0.0,
        }
    }

    pub fn update(&mut self, dt: f64) {
        if self.hit_flash > 0.0 {
            self.hit_flash -= dt;
        }

        self.x += self.vel_x * dt as f32;
        self.y += self.vel_y * dt as f32;
        self.lifetime -= dt;

        // Bombs arc under gravity.
        if self.bullet_type == BulletType::EnemyBomb {
            self.vel_y += 540.0 * dt as f32;
        }

        if self.lifetime <= 0.0
            || self.y < -50.0
            || self.y > screen_height() + 50.0
            || self.x < -50.0
            || self.x > screen_width() + 50.0
        {
            self.is_dead = true;
        }
    }

    pub fn draw(&self, texture: Option<&Texture2D>) {
        let color = if self.hit_flash > 0.0 {
            WHITE
        } else {
            self.bullet_type.color()
        };
        self.draw_shape(color, texture);
    }

    fn draw_shape(&self, color: Color, texture: Option<&Texture2D>) {
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
                },
            );
        } else {
            self.draw_procedural(color);
        }
    }

    fn draw_procedural(&self, color: Color) {
        match self.bullet_type {
            BulletType::PlayerStandard | BulletType::PlayerPierce => {
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
            BulletType::EnemyStraight | BulletType::EnemyAimed | BulletType::EnemySpread => {
                draw_circle(self.x, self.y, self.width / 2.0, color);
                draw_circle(self.x, self.y, self.width / 3.0, WHITE);
            }
            BulletType::EnemyBomb => {
                draw_circle(self.x, self.y, self.width / 2.0, color);
                draw_circle(
                    self.x,
                    self.y,
                    self.width / 2.0 - 3.0,
                    Color::new(0.5, 0.0, 0.5, 1.0),
                );
            }
        }
    }

    pub fn get_rect(&self) -> (f32, f32, f32, f32) {
        (
            self.x - self.width / 2.0,
            self.y - self.height / 2.0,
            self.width,
            self.height,
        )
    }

    /// Called when the bullet damages something. Returns true if the bullet
    /// survives to hit another target.
    pub fn pierce(&mut self) -> bool {
        if self.pierce_remaining > 0 {
            self.pierce_remaining -= 1;
            return self.pierce_remaining >= 0;
        }
        false
    }

    /// A bullet that recently hit something is in its damage debounce
    /// window and cannot damage again yet.
    pub fn can_damage(&self) -> bool {
        self.hit_flash <= 0.0
    }
}

/// Fan of `count` bullets around the base velocity, spanning `spread_angle`
/// radians total.
pub fn create_spread_bullets(
    bullet_type: BulletType,
    x: f32,
    y: f32,
    base_vel_x: f32,
    base_vel_y: f32,
    count: i32,
    spread_angle: f32,
) -> Vec<Bullet> {
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
