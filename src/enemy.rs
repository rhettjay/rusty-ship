use macroquad::prelude::*;
use crate::config::{ENEMY_CONFIG, WAVE_CONFIG};
use ::rand::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EnemyType {
    Scout,
    Fighter,
    Bomber,
    Interceptor,
    Elite,
}

impl EnemyType {
    pub fn base_hp(&self) -> i32 {
        match self {
            EnemyType::Scout => 1,
            EnemyType::Fighter => 2,
            EnemyType::Bomber => 3,
            EnemyType::Interceptor => 2,
            EnemyType::Elite => 4,
        }
    }

    pub fn base_armor(&self) -> i32 {
        match self {
            EnemyType::Scout => 0,
            EnemyType::Fighter => 0,
            EnemyType::Bomber => 1,
            EnemyType::Interceptor => 0,
            EnemyType::Elite => 2,
        }
    }

    pub fn speed_range(&self) -> (f32, f32) {
        match self {
            EnemyType::Scout => (3.0, 5.0),
            EnemyType::Fighter => (1.5, 3.0),
            EnemyType::Bomber => (0.8, 1.5),
            EnemyType::Interceptor => (2.5, 4.0),
            EnemyType::Elite => (2.0, 3.5),
        }
    }

    pub fn size(&self) -> (f32, f32) {
        match self {
            EnemyType::Scout => (24.0, 24.0),
            EnemyType::Fighter => (32.0, 32.0),
            EnemyType::Bomber => (40.0, 40.0),
            EnemyType::Interceptor => (28.0, 28.0),
            EnemyType::Elite => (32.0, 32.0),
        }
    }

    pub fn shoot_pattern(&self) -> ShootPattern {
        match self {
            EnemyType::Scout => ShootPattern::None,
            EnemyType::Fighter => ShootPattern::Straight,
            EnemyType::Bomber => ShootPattern::Bomb,
            EnemyType::Interceptor => ShootPattern::Aimed,
            EnemyType::Elite => ShootPattern::Spread,
        }
    }

    pub fn shoot_interval(&self) -> f64 {
        match self {
            EnemyType::Scout => 0.0,
            EnemyType::Fighter => 2.0,
            EnemyType::Bomber => 3.0,
            EnemyType::Interceptor => 1.5,
            EnemyType::Elite => 1.2,
        }
    }

    pub fn score_value(&self) -> i32 {
        match self {
            EnemyType::Scout => 10,
            EnemyType::Fighter => 25,
            EnemyType::Bomber => 50,
            EnemyType::Interceptor => 40,
            EnemyType::Elite => 100,
        }
    }

    pub fn powerup_chance(&self) -> f32 {
        match self {
            EnemyType::Scout => 0.03,
            EnemyType::Fighter => 0.05,
            EnemyType::Bomber => 0.10,
            EnemyType::Interceptor => 0.07,
            EnemyType::Elite => 0.25,
        }
    }

    pub fn sprite_name(&self) -> &'static str {
        match self {
            EnemyType::Scout => "scout",
            EnemyType::Fighter => "fighter",
            EnemyType::Bomber => "bomber",
            EnemyType::Interceptor => "interceptor",
            EnemyType::Elite => "fighter",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            EnemyType::Scout => LIGHTGRAY,
            EnemyType::Fighter => WHITE,
            EnemyType::Bomber => ORANGE,
            EnemyType::Interceptor => SKYBLUE,
            EnemyType::Elite => GOLD,
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

pub struct Enemy {
    pub enemy_type: EnemyType,
    pub hp: i32,
    pub max_hp: i32,
    pub armor: i32,
    pub x: f32,
    pub y: f32,
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
    pub formation_offset: Option<(f32, f32)>,
    pub entry_anim: bool,
    pub entry_target_y: f32,
    pub hit_flash: f64,
    pub behavior_state: BehaviorState,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BehaviorState {
    Entering,
    Formation,
    Diving,
    Returning,
    Idle,
}

impl Enemy {
    pub fn new(enemy_type: EnemyType, x: f32, y: f32, wave: u32) -> Self {
        let base_hp = enemy_type.base_hp();
        let base_armor = enemy_type.base_armor();
        let hp_scaling = wave / 3;
        let armor_scaling = wave / 5;

        let hp = base_hp + hp_scaling as i32;
        let armor = base_armor + armor_scaling as i32;
        let (width, height) = enemy_type.size();
        let (min_speed, max_speed) = enemy_type.speed_range();
        let mut rng = ::rand::thread_rng();
        let speed_x = rng.gen_range(min_speed..max_speed) * if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
        let speed_y = rng.gen_range(0.5..1.5);

        Self {
            enemy_type,
            hp,
            max_hp: hp,
            armor,
            x,
            y,
            speed_x,
            speed_y,
            color: enemy_type.color(),
            is_dead: false,
            shoot_timer: 0.0,
            shoot_pattern: enemy_type.shoot_pattern(),
            shoot_interval: enemy_type.shoot_interval(),
            score_value: enemy_type.score_value(),
            powerup_chance: enemy_type.powerup_chance(),
            width,
            height,
            is_elite: enemy_type.is_elite(),
            formation_offset: None,
            entry_anim: true,
            entry_target_y: rng.gen_range(80.0..200.0),
            hit_flash: 0.0,
            behavior_state: BehaviorState::Entering,
        }
    }

    pub fn new_elite(enemy_type: EnemyType, x: f32, y: f32, wave: u32) -> Self {
        let mut enemy = Self::new(enemy_type, x, y, wave);
        enemy.is_elite = true;
        enemy.color = GOLD;
        enemy.hp = (enemy.hp as f32 * 1.5) as i32;
        enemy.max_hp = enemy.hp;
        enemy.armor += 2;
        enemy.score_value = (enemy.score_value as f32 * 2.0) as i32;
        enemy.powerup_chance *= 3.0;
        
        match enemy_type {
            EnemyType::Fighter => enemy.shoot_pattern = ShootPattern::Spread,
            EnemyType::Interceptor => enemy.shoot_pattern = ShootPattern::Aimed,
            EnemyType::Bomber => enemy.shoot_pattern = ShootPattern::Bomb,
            _ => enemy.shoot_pattern = ShootPattern::Straight,
        }
        enemy.shoot_interval *= 0.7;
        enemy
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

    pub fn update(&mut self, dt: f64, player_x: f32, player_y: f32) {
        if self.hit_flash > 0.0 {
            self.hit_flash -= dt;
        }

        if self.entry_anim {
            self.y += 2.0;
            if self.y >= self.entry_target_y {
                self.y = self.entry_target_y;
                self.entry_anim = false;
                self.behavior_state = BehaviorState::Formation;
            }
            return;
        }

        match self.behavior_state {
            BehaviorState::Formation => {
                self.update_formation_movement(dt);
            }
            BehaviorState::Diving => {
                self.update_diving(dt, player_x, player_y);
            }
            BehaviorState::Returning => {
                self.update_returning(dt);
            }
            BehaviorState::Idle => {
                self.update_idle(dt);
            }
            _ => {}
        }

        self.shoot_timer += dt;
        
        if let Some(offset) = self.formation_offset {
            let target_x = screen_width() / 2.0 + offset.0 - self.width / 2.0;
            let target_y = 100.0 + offset.1;
            let dx = target_x - self.x;
            let dy = target_y - self.y;
            let dist = (dx * dx + dy * dy).sqrt();
            
            if dist > 5.0 {
                self.x += (dx / dist) * 2.0;
                self.y += (dy / dist) * 2.0;
            } else {
                self.formation_offset = None;
            }
        }

        self.clamp_to_screen();
    }

    fn update_formation_movement(&mut self, dt: f64) {
        self.x += self.speed_x * dt as f32;
        self.y += (self.speed_y * dt as f32 * 0.5).sin() * 0.3;

        if self.x <= self.width / 2.0 || self.x >= screen_width() - self.width / 2.0 {
            self.speed_x = -self.speed_x;
        }
    }

    fn update_diving(&mut self, dt: f64, player_x: f32, player_y: f32) {
        let dx = player_x - self.x;
        let dy = player_y - self.y;
        let dist = (dx * dx + dy * dy).sqrt();
        
        if dist > 0.0 {
            self.x += (dx / dist) * self.speed_x * dt as f32 * 1.5;
            self.y += (dy / dist) * self.speed_y * dt as f32 * 1.5;
        }

        if self.y > screen_height() + 50.0 {
            self.behavior_state = BehaviorState::Returning;
        }
    }

    fn update_returning(&mut self, dt: f64) {
        let target_y = self.entry_target_y;
        let dy = target_y - self.y;
        
        if dy.abs() > 5.0 {
            self.y += dy.signum() * self.speed_y * dt as f32;
        } else {
            self.y = target_y;
            self.behavior_state = BehaviorState::Formation;
        }
    }

    fn update_idle(&mut self, dt: f64) {
        self.y += self.speed_y * dt as f32 * 0.5;
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
        self.shoot_timer >= self.shoot_interval && !self.entry_anim
    }

    pub fn reset_shoot_timer(&mut self) {
        self.shoot_timer = 0.0;
    }

    pub fn get_shoot_direction(&self, player_x: f32, player_y: f32) -> (f32, f32) {
        match self.shoot_pattern {
            ShootPattern::Straight => (0.0, 1.0),
            ShootPattern::Aimed => {
                let dx = player_x - self.x;
                let dy = player_y - self.y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist > 0.0 { (dx / dist, dy / dist) } else { (0.0, 1.0) }
            }
            ShootPattern::Bomb => (0.0, 1.0),
            ShootPattern::Spread => (0.0, 1.0),
            ShootPattern::None => (0.0, 0.0),
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
                }
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
    let mut weights = vec![
        (EnemyType::Scout, 1.0),
        (EnemyType::Fighter, 0.5),
        (EnemyType::Bomber, 0.2),
        (EnemyType::Interceptor, 0.1),
        (EnemyType::Elite, 0.0),
    ];

    match wave {
        1..=2 => {}
        3..=4 => {
            weights[1].1 = 0.8;
            weights[2].1 = 0.3;
        }
        5..=6 => {
            weights[1].1 = 1.0;
            weights[2].1 = 0.5;
            weights[3].1 = 0.4;
        }
        7..=9 => {
            weights[1].1 = 1.0;
            weights[2].1 = 0.8;
            weights[3].1 = 0.6;
            weights[4].1 = 0.05;
        }
        10..=14 => {
            weights[0].1 = 0.5;
            weights[1].1 = 1.0;
            weights[2].1 = 1.0;
            weights[3].1 = 0.8;
            weights[4].1 = 0.15;
        }
        _ => {
            weights[0].1 = 0.3;
            weights[1].1 = 1.0;
            weights[2].1 = 1.0;
            weights[3].1 = 1.0;
            weights[4].1 = 0.25;
        }
    }

    weights
}

pub fn select_enemy_type(wave: u32, rng: &mut impl Rng) -> EnemyType {
    let weights = get_spawn_weights(wave);
    let total: f32 = weights.iter().map(|(_, w)| *w).sum();
    let mut roll = rng.gen_range(0.0..total);
    
    for (enemy_type, weight) in weights {
        if roll <= weight {
            if enemy_type == EnemyType::Elite && rng.gen_bool(0.3) {
                return EnemyType::Elite;
            }
            return enemy_type;
        }
        roll -= weight;
    }
    EnemyType::Scout
}