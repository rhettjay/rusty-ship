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
    
    pub waves_per_chapter: u32,
    pub boss_waves: &'static [u32],
    pub boss_types: &'static [&'static str],
    pub bonus_score_threshold: i32,
    
    pub dialogue_typewriter_speed: f32,
    pub dialogue_box_padding: f32,
    pub dialogue_portrait_size: f32,
    
    pub music_volume: f32,
    pub sfx_volume: f32,
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
            
            waves_per_chapter: 3,
            boss_waves: &[3, 6, 9, 12, 15],
            boss_types: &["Blowfish", "Twofish", "RufusReverse", "MollyHashpass", "CaptainDavey"],
            bonus_score_threshold: 50000,
            
            dialogue_typewriter_speed: 30.0,
            dialogue_box_padding: 20.0,
            dialogue_portrait_size: 128.0,
            
            music_volume: 0.5,
            sfx_volume: 0.7,
        }
    }
}

pub const CONFIG: GameConfig = GameConfig::new();