use macroquad::prelude::*;
use crate::content::{self, FormationSpec, DetailedFormation};
use crate::enemy::{Enemy, EnemyType, BehaviorState};
use ::rand::Rng;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FormationType {
    Random,
    Vee { count: u32, spacing: f32 },
    Line { count: u32, spacing: f32 },
    Circle { count: u32, radius: f32 },
    Escort { leader_type: EnemyType, follower_count: u32 },
    Grid { rows: u32, cols: u32, spacing: f32 },
    Chaos,
}

impl FormationType {
    pub fn from_spec(spec: &FormationSpec) -> FormationType {
        match spec {
            FormationSpec::Simple(name) => match name.as_str() {
                "vee" => FormationType::Vee { count: 5, spacing: 60.0 },
                "line" => FormationType::Line { count: 6, spacing: 80.0 },
                "circle" => FormationType::Circle { count: 8, radius: 120.0 },
                "escort" => FormationType::Escort { leader_type: EnemyType::Bomber, follower_count: 4 },
                "grid" => FormationType::Grid { rows: 3, cols: 5, spacing: 70.0 },
                "chaos" => FormationType::Chaos,
                _ => FormationType::Random,
            },
            FormationSpec::Detailed(d) => match d {
                DetailedFormation::Vee { count, spacing } => FormationType::Vee { count: *count, spacing: *spacing },
                DetailedFormation::Line { count, spacing } => FormationType::Line { count: *count, spacing: *spacing },
                DetailedFormation::Circle { count, radius } => FormationType::Circle { count: *count, radius: *radius },
                DetailedFormation::Escort { leader, followers } => FormationType::Escort {
                    leader_type: EnemyType::from_key(leader).unwrap_or(EnemyType::Bomber),
                    follower_count: *followers,
                },
                DetailedFormation::Grid { rows, cols, spacing } => FormationType::Grid { rows: *rows, cols: *cols, spacing: *spacing },
                DetailedFormation::Chaos => FormationType::Chaos,
            },
        }
    }

    pub fn default_params(&self) -> Self {
        match self {
            FormationType::Random => FormationType::Random,
            FormationType::Vee { .. } => FormationType::Vee { count: 5, spacing: 60.0 },
            FormationType::Line { .. } => FormationType::Line { count: 6, spacing: 80.0 },
            FormationType::Circle { .. } => FormationType::Circle { count: 8, radius: 120.0 },
            FormationType::Escort { .. } => FormationType::Escort { leader_type: EnemyType::Bomber, follower_count: 4 },
            FormationType::Grid { .. } => FormationType::Grid { rows: 3, cols: 5, spacing: 70.0 },
            FormationType::Chaos => FormationType::Chaos,
        }
    }

    pub fn get_positions(&self, center_x: f32, center_y: f32) -> Vec<(f32, f32, EnemyType)> {
        match self {
            FormationType::Random => vec![],
            FormationType::Vee { count, spacing } => self.vee_positions(center_x, center_y, *count, *spacing),
            FormationType::Line { count, spacing } => self.line_positions(center_x, center_y, *count, *spacing),
            FormationType::Circle { count, radius } => self.circle_positions(center_x, center_y, *count, *radius),
            FormationType::Escort { leader_type, follower_count } => self.escort_positions(center_x, center_y, *leader_type, *follower_count),
            FormationType::Grid { rows, cols, spacing } => self.grid_positions(center_x, center_y, *rows, *cols, *spacing),
            FormationType::Chaos => vec![],
        }
    }

    fn vee_positions(&self, cx: f32, cy: f32, count: u32, spacing: f32) -> Vec<(f32, f32, EnemyType)> {
        let mut positions = Vec::new();
        let half = count / 2;
        
        for i in 0..count {
            let row = (i / 2) as f32;
            let side = if i % 2 == 0 { -1.0 } else { 1.0 };
            let offset = side * (row + 1.0) * spacing * 0.5;
            let y = cy + row * spacing * 0.7;
            
            let enemy_type = if row == 0.0 { EnemyType::Fighter } 
                           else if row <= 1.0 { EnemyType::Interceptor }
                           else { EnemyType::Scout };
            
            positions.push((cx + offset, y, enemy_type));
        }
        positions
    }

    fn line_positions(&self, cx: f32, cy: f32, count: u32, spacing: f32) -> Vec<(f32, f32, EnemyType)> {
        let mut positions = Vec::new();
        let start_x = cx - (count - 1) as f32 * spacing * 0.5;
        
        for i in 0..count {
            let x = start_x + i as f32 * spacing;
            let y = cy + (i as f32 * 0.3).sin() * 30.0;
            
            let enemy_type = if i % 3 == 0 { EnemyType::Bomber }
                           else if i % 2 == 0 { EnemyType::Fighter }
                           else { EnemyType::Scout };
            
            positions.push((x, y, enemy_type));
        }
        positions
    }

    fn circle_positions(&self, cx: f32, cy: f32, count: u32, radius: f32) -> Vec<(f32, f32, EnemyType)> {
        let mut positions = Vec::new();
        let angle_step = std::f32::consts::TAU / count as f32;
        
        for i in 0..count {
            let angle = i as f32 * angle_step;
            let x = cx + angle.cos() * radius;
            let y = cy + angle.sin() * radius * 0.5;
            
            let enemy_type = match i % 4 {
                0 => EnemyType::Interceptor,
                1 => EnemyType::Fighter,
                2 => EnemyType::Bomber,
                _ => EnemyType::Scout,
            };
            
            positions.push((x, y, enemy_type));
        }
        positions
    }

    fn escort_positions(&self, cx: f32, cy: f32, leader_type: EnemyType, follower_count: u32) -> Vec<(f32, f32, EnemyType)> {
        let mut positions = Vec::new();
        
        positions.push((cx, cy, leader_type));
        
        let angle_step = std::f32::consts::TAU / follower_count as f32;
        for i in 0..follower_count {
            let angle = i as f32 * angle_step;
            let x = cx + angle.cos() * 80.0;
            let y = cy + angle.sin() * 40.0 + 50.0;
            
            positions.push((x, y, EnemyType::Fighter));
        }
        positions
    }

    fn grid_positions(&self, cx: f32, cy: f32, rows: u32, cols: u32, spacing: f32) -> Vec<(f32, f32, EnemyType)> {
        let mut positions = Vec::new();
        let start_x = cx - (cols - 1) as f32 * spacing * 0.5;
        let start_y = cy - (rows - 1) as f32 * spacing * 0.5;
        
        for row in 0..rows {
            for col in 0..cols {
                let x = start_x + col as f32 * spacing;
                let y = start_y + row as f32 * spacing;
                
                let enemy_type = match row {
                    0 => EnemyType::Bomber,
                    1 => EnemyType::Fighter,
                    _ => EnemyType::Scout,
                };
                
                positions.push((x, y, enemy_type));
            }
        }
        positions
    }

    pub fn apply_to_enemies(&self, enemies: &mut Vec<Enemy>, center_x: f32, center_y: f32) {
        let positions = self.get_positions(center_x, center_y);
        
        for (enemy, (x, y, enemy_type)) in enemies.iter_mut().zip(positions) {
            enemy.x = x;
            enemy.y = y - 200.0;
            enemy.entry_target_y = y;
            enemy.entry_anim = true;
            enemy.behavior_state = BehaviorState::Entering;
            
            if enemy_type != enemy.enemy_type {
                let wave = 5;
                *enemy = Enemy::new(enemy_type, x, y, wave);
            }
        }
    }
}

pub fn create_random_formation(wave: u32) -> FormationType {
    let formations = get_wave_formations(wave);
    if formations.is_empty() {
        return FormationType::Random;
    }
    let mut rng = ::rand::thread_rng();
    formations[rng.gen_range(0..formations.len())]
}

pub fn get_wave_formations(wave: u32) -> Vec<FormationType> {
    content::wave(wave)
        .formations
        .iter()
        .map(FormationType::from_spec)
        .collect()
}

pub struct FormationManager {
    pub is_active: bool,
    pub formation_type: Option<FormationType>,
    pub spawn_timer: f64,
    pub spawn_interval: f64,
    pub enemies_to_spawn: Vec<(f32, f32, EnemyType)>,
    pub current_index: usize,
}

impl FormationManager {
    pub fn new() -> Self {
        Self {
            is_active: false,
            formation_type: None,
            spawn_timer: 0.0,
            spawn_interval: 0.0,
            enemies_to_spawn: Vec::new(),
            current_index: 0,
        }
    }

    pub fn start_formation(&mut self, formation_type: FormationType, wave: u32, spawn_interval: f32) {
        self.formation_type = Some(formation_type);
        self.spawn_interval = spawn_interval as f64;
        self.spawn_timer = 0.0;
        self.is_active = true;
        self.current_index = 0;
        
        let center_x = screen_width() / 2.0;
        let center_y = 150.0;
        
        if let Some(ft) = &self.formation_type {
            self.enemies_to_spawn = ft.get_positions(center_x, center_y);
        }
    }

    pub fn update(&mut self, dt: f64, enemy_vec: &mut Vec<crate::enemy::Enemy>, wave: u32) -> bool {
        if !self.is_active || self.enemies_to_spawn.is_empty() {
            return true;
        }

        self.spawn_timer += dt;
        
        while self.current_index < self.enemies_to_spawn.len() && self.spawn_timer >= self.spawn_interval {
            let (x, y, enemy_type) = self.enemies_to_spawn[self.current_index];
            let enemy = crate::enemy::Enemy::new(enemy_type, x, y, wave);
            enemy_vec.push(enemy);
            self.current_index += 1;
        }

        if self.current_index >= self.enemies_to_spawn.len() {
            self.is_active = false;
            return true;
        }
        false
    }
}