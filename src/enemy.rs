use crate::content;
use ::rand::Rng;
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EnemyType {
    Scout,
    Fighter,
    Bomber,
    Interceptor,
    Elite,
}

impl EnemyType {
    pub fn key(&self) -> &'static str {
        match self {
            EnemyType::Scout => "scout",
            EnemyType::Fighter => "fighter",
            EnemyType::Bomber => "bomber",
            EnemyType::Interceptor => "interceptor",
            EnemyType::Elite => "elite",
        }
    }

    pub fn from_key(key: &str) -> Option<EnemyType> {
        match key {
            "scout" => Some(EnemyType::Scout),
            "fighter" => Some(EnemyType::Fighter),
            "bomber" => Some(EnemyType::Bomber),
            "interceptor" => Some(EnemyType::Interceptor),
            "elite" => Some(EnemyType::Elite),
            _ => None,
        }
    }

    pub fn archetype(&self) -> &'static content::EnemyArchetype {
        content::enemy(self.key())
            .unwrap_or_else(|| panic!("missing enemy archetype: {}", self.key()))
    }

    /// Asset file stem under assets/enemies/.
    pub fn sprite_name(&self) -> &'static str {
        match self.archetype().sprite.as_str() {
            "scout" => "scout",
            "bomber" => "bomber",
            "interceptor" => "interceptor",
            _ => "fighter",
        }
    }

    pub fn is_elite(&self) -> bool {
        matches!(self, EnemyType::Elite)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShootPattern {
    None,
    Straight,
    Aimed,
    Bomb,
    Spread,
}

impl ShootPattern {
    pub fn from_key(key: &str) -> ShootPattern {
        match key {
            "straight" => ShootPattern::Straight,
            "aimed" => ShootPattern::Aimed,
            "bomb" => ShootPattern::Bomb,
            "spread" => ShootPattern::Spread,
            _ => ShootPattern::None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BehaviorState {
    Entering,
    Formation,
}

pub struct Enemy {
    pub enemy_type: EnemyType,
    pub hp: i32,
    pub max_hp: i32,
    pub armor: i32,
    pub x: f32,
    pub y: f32,
    /// Pixels per second.
    pub speed_x: f32,
    pub speed_y: f32,
    pub color: Color,
    pub is_dead: bool,
    pub shoot_timer: f64,
    pub shoot_pattern: ShootPattern,
    pub shoot_interval: f64,
    pub score_value: i32,
    pub powerup_chance: f32,
    pub width: f32,
    pub height: f32,
    pub is_elite: bool,
    pub hit_flash: f64,
    pub behavior_state: BehaviorState,
    entry_target_y: f32,
}

impl Enemy {
    pub fn new(enemy_type: EnemyType, x: f32, y: f32, wave: u32) -> Self {
        let arch = enemy_type.archetype();
        let hp = arch.base_hp + (wave / 3) as i32;
        let armor = arch.base_armor + (wave / 5) as i32;
        let (width, height) = arch.size;
        let (min_speed, max_speed) = arch.speed_range;
        let mut rng = ::rand::thread_rng();
        // Content speeds are authored as pixels-per-frame at 60 fps.
        let speed_x =
            rng.gen_range(min_speed..max_speed) * 60.0 * if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
        let speed_y = rng.gen_range(30.0..90.0);

        Self {
            enemy_type,
            hp,
            max_hp: hp,
            armor,
            x,
            y,
            speed_x,
            speed_y,
            color: content::parse_color(&arch.color),
            is_dead: false,
            shoot_timer: 0.0,
            shoot_pattern: ShootPattern::from_key(&arch.shoot_pattern),
            shoot_interval: arch.shoot_interval,
            score_value: arch.score_value,
            powerup_chance: arch.powerup_chance,
            width,
            height,
            is_elite: enemy_type.is_elite(),
            hit_flash: 0.0,
            behavior_state: BehaviorState::Entering,
            entry_target_y: rng.gen_range(80.0..200.0),
        }
    }

    pub fn take_damage(&mut self, damage: i32) -> bool {
        let actual_damage = (damage - self.armor).max(1);
        self.hp -= actual_damage;
        self.hit_flash = 0.15;

        if self.hp <= 0 {
            self.hp = 0;
            self.is_dead = true;
            return true;
        }
        false
    }

    pub fn update(&mut self, dt: f64, _player_x: f32, _player_y: f32) {
        let dt_f32 = dt as f32;

        if self.hit_flash > 0.0 {
            self.hit_flash -= dt;
        }

        match self.behavior_state {
            BehaviorState::Entering => {
                self.y += 120.0 * dt_f32;
                if self.y >= self.entry_target_y {
                    self.y = self.entry_target_y;
                    self.behavior_state = BehaviorState::Formation;
                }
            }
            BehaviorState::Formation => {
                self.x += self.speed_x * dt_f32;
                self.y += (self.speed_y * dt_f32 * 0.5).sin() * 0.3;
                if self.x <= self.width / 2.0 || self.x >= screen_width() - self.width / 2.0 {
                    self.speed_x = -self.speed_x;
                }
            }
        }

        self.shoot_timer += dt;
        self.clamp_to_screen();
    }

    fn clamp_to_screen(&mut self) {
        if self.x < self.width / 2.0 {
            self.x = self.width / 2.0;
            self.speed_x = self.speed_x.abs();
        }
        if self.x > screen_width() - self.width / 2.0 {
            self.x = screen_width() - self.width / 2.0;
            self.speed_x = -self.speed_x.abs();
        }
    }

    pub fn can_shoot(&self) -> bool {
        self.shoot_timer >= self.shoot_interval && self.behavior_state == BehaviorState::Formation
    }

    pub fn reset_shoot_timer(&mut self) {
        self.shoot_timer = 0.0;
    }

    pub fn get_shoot_direction(&self, player_x: f32, player_y: f32) -> (f32, f32) {
        match self.shoot_pattern {
            ShootPattern::Aimed => {
                let dx = player_x - self.x;
                let dy = player_y - self.y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist > 0.0 {
                    (dx / dist, dy / dist)
                } else {
                    (0.0, 1.0)
                }
            }
            ShootPattern::None => (0.0, 0.0),
            _ => (0.0, 1.0),
        }
    }

    pub fn draw(&self, texture: Option<&Texture2D>) {
        let mut draw_color = self.color;

        if self.hit_flash > 0.0 {
            draw_color = WHITE;
        } else if self.is_elite {
            let pulse = ((get_time() * 5.0).sin() * 0.3 + 0.7) as f32;
            draw_color = Color::new(
                draw_color.r * pulse,
                draw_color.g * pulse,
                draw_color.b * pulse,
                draw_color.a,
            );
        }

        if let Some(tex) = texture {
            draw_texture_ex(
                tex,
                self.x - self.width / 2.0,
                self.y - self.height / 2.0,
                draw_color,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(self.width, self.height)),
                    ..Default::default()
                },
            );
        } else {
            draw_rectangle(
                self.x - self.width / 2.0,
                self.y - self.height / 2.0,
                self.width,
                self.height,
                draw_color,
            );
        }

        if self.max_hp > 3 {
            self.draw_health_bar();
        }
    }

    fn draw_health_bar(&self) {
        let bar_w = self.width;
        let bar_h = 4.0;
        let x = self.x - bar_w / 2.0;
        let y = self.y - self.height / 2.0 - 8.0;

        draw_rectangle(x, y, bar_w, bar_h, Color::new(0.2, 0.0, 0.0, 0.8));
        let pct = self.hp as f32 / self.max_hp as f32;
        draw_rectangle(x, y, bar_w * pct, bar_h, RED);
    }

    pub fn get_rect(&self) -> (f32, f32, f32, f32) {
        (
            self.x - self.width / 2.0,
            self.y - self.height / 2.0,
            self.width,
            self.height,
        )
    }
}

pub fn get_spawn_weights(wave: u32) -> Vec<(EnemyType, f32)> {
    let def = content::wave(wave);

    if def.enemy_weights.is_empty() {
        return default_spawn_weights();
    }

    let mut weights: Vec<(EnemyType, f32)> = def
        .enemy_weights
        .iter()
        .filter_map(|(key, weight)| EnemyType::from_key(key).map(|et| (et, *weight)))
        .collect();

    if weights.is_empty() {
        weights = default_spawn_weights();
    }

    weights.sort_by_key(|(et, _)| *et as u8);
    weights
}

fn default_spawn_weights() -> Vec<(EnemyType, f32)> {
    vec![
        (EnemyType::Scout, 0.3),
        (EnemyType::Fighter, 1.0),
        (EnemyType::Bomber, 1.0),
        (EnemyType::Interceptor, 1.0),
        (EnemyType::Elite, 0.25),
    ]
}

pub fn select_enemy_type(wave: u32, rng: &mut impl Rng) -> EnemyType {
    let weights = get_spawn_weights(wave);
    let total: f32 = weights.iter().map(|(_, w)| *w).sum();
    let mut roll = rng.gen_range(0.0..total);

    for (enemy_type, weight) in weights {
        if roll <= weight {
            return enemy_type;
        }
        roll -= weight;
    }
    EnemyType::Scout
}
