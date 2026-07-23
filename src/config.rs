pub struct GameConfig {
    pub window_width: f32,
    pub window_height: f32,
    pub window_title: &'static str,
    
    pub ship_speed: f32,
    pub ship_width: f32,
    pub ship_height: f32,
    pub ship_start_y_offset: f32,
    
    pub cannonball_speed: f32,
    pub cannonball_width: f32,
    pub cannonball_height: f32,
    pub cannonball_cooldown: f64,
    
    pub pirate_base_speed_x_range: (f32, f32),
    pub pirate_base_speed_y_range: (f32, f32),
    pub pirate_spawn_chance: u32,
    pub pirate_max_count: i32,
    pub pirate_width: f32,
    pub pirate_height: f32,
    pub pirate_rotation: f32,
    
    pub starting_lives: i32,
    pub starting_pirate_count: i32,
}

impl GameConfig {
    pub const fn new() -> Self {
        Self {
            window_width: 800.0,
            window_height: 800.0,
            window_title: "rusty-ship",
            
            ship_speed: 5.0,
            ship_width: 60.0,
            ship_height: 64.0,
            ship_start_y_offset: 100.0,
            
            cannonball_speed: 10.0,
            cannonball_width: 5.0,
            cannonball_height: 15.0,
            cannonball_cooldown: 0.2,
            
            pirate_base_speed_x_range: (1.0, 8.0),
            pirate_base_speed_y_range: (1.0, 3.0),
            pirate_spawn_chance: 25,
            pirate_max_count: 10,
            pirate_width: 15.0,
            pirate_height: 15.0,
            pirate_rotation: 3.14159265359,
            
            starting_lives: 3,
            starting_pirate_count: 10,
        }
    }
}

pub const CONFIG: GameConfig = GameConfig::new();