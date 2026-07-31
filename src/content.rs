use macroquad::prelude::Color;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicPtr, Ordering};

/// Runtime-loaded game content (enemy archetypes, wave tables, boss definitions).
///
/// Files live in `assets/content/` and are read at startup, so balance tweaks
/// take effect without recompiling. If a file is missing or malformed we fall
/// back to the embedded copy of the same file so the game always boots.

// ---- enemy archetypes ----------------------------------------------------

#[derive(Clone, Debug, Deserialize)]
pub struct EnemyArchetype {
    pub base_hp: i32,
    pub base_armor: i32,
    pub speed_range: (f32, f32),
    pub size: (f32, f32),
    pub shoot_pattern: String,
    pub shoot_interval: f64,
    pub score_value: i32,
    pub powerup_chance: f32,
    pub color: String,
    pub sprite: String,
}

// ---- waves ---------------------------------------------------------------

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum FormationSpec {
    Simple(String),
    Detailed(DetailedFormation),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DetailedFormation {
    Vee { count: u32, spacing: f32 },
    Line { count: u32, spacing: f32 },
    Circle { count: u32, radius: f32 },
    Escort { leader: String, followers: u32 },
    Grid { rows: u32, cols: u32, spacing: f32 },
    Chaos,
}

#[derive(Clone, Debug, Deserialize)]
pub struct WaveDef {
    pub duration: f32,
    pub max_enemies: u32,
    pub spawn_interval: f32,
    #[serde(default)]
    pub formations: Vec<FormationSpec>,
    #[serde(default)]
    pub powerup_chance: f32,
    #[serde(default)]
    pub boss: Option<String>,
    #[serde(default)]
    pub enemy_weights: BTreeMap<String, f32>,
}

impl Default for WaveDef {
    fn default() -> Self {
        Self {
            duration: 30.0,
            max_enemies: 24,
            spawn_interval: 1.0,
            formations: vec![],
            powerup_chance: 0.0,
            boss: None,
            enemy_weights: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
struct WavesFile {
    #[serde(default)]
    default: WaveDef,
    #[serde(flatten)]
    waves: BTreeMap<String, WaveDef>,
}

// ---- bosses --------------------------------------------------------------

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MovementDef {
    Patrol { speed_x: f32, base_y: f32, amp_y: f32, freq_y: f32 },
    CenteredSway { amp_x: f32, freq_x: f32, base_y: f32, amp_y: f32, freq_y: f32 },
    Teleport { cooldown: f32 },
}

#[derive(Clone, Debug, Deserialize)]
pub struct BulletDef {
    pub size: f32,
    #[serde(default)]
    pub damage: i32,
    pub color: String,
    #[serde(default = "default_bullet_pattern")]
    pub pattern: String,
    #[serde(default = "default_bullet_lifetime")]
    pub lifetime: f64,
}

fn default_bullet_pattern() -> String {
    "straight".to_string()
}

fn default_bullet_lifetime() -> f64 {
    5.0
}

fn default_beam_bullet() -> BulletDef {
    BulletDef {
        size: 8.0,
        damage: 2,
        color: "red".to_string(),
        pattern: "straight".to_string(),
        lifetime: 0.5,
    }
}

fn default_food_bullet() -> BulletDef {
    BulletDef {
        size: 20.0,
        damage: 1,
        color: "brown".to_string(),
        pattern: "bounce".to_string(),
        lifetime: 5.0,
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AttackDef {
    Burst {
        count: u32,
        speed: f32,
        bullet: BulletDef,
        #[serde(default = "one")]
        chance: f32,
    },
    Aimed {
        count: u32,
        speed: f32,
        #[serde(default)]
        spread: f32,
        #[serde(default)]
        mirror: bool,
        bullet: BulletDef,
        #[serde(default = "one")]
        chance: f32,
    },
    Scatter {
        count: u32,
        speed: f32,
        spread: f32,
        bullet: BulletDef,
        #[serde(default = "one")]
        chance: f32,
    },
    Ring {
        count: u32,
        speed: f32,
        bullet: BulletDef,
        #[serde(default = "one")]
        chance: f32,
    },
    Homing {
        count: u32,
        bullet: BulletDef,
        #[serde(default = "one")]
        chance: f32,
    },
    Charge {
        #[serde(default)]
        speed: f32,
        #[serde(default)]
        invuln: f32,
        #[serde(default = "one")]
        chance: f32,
    },
    Heal {
        amount: i32,
        #[serde(default = "one")]
        chance: f32,
    },
    HealBeam {
        amount: i32,
        #[serde(default)]
        invuln: f32,
        #[serde(default = "one")]
        chance: f32,
    },
    PoisonCloud {
        count: u32,
        bullet: BulletDef,
        #[serde(default = "one")]
        chance: f32,
    },
    ReviveMinions,
    SpawnMinions,
    ReverseZone {
        duration: f32,
        #[serde(default = "one")]
        chance: f32,
    },
    Wall {
        rows: u32,
        cols: u32,
        speed: f32,
        gap: u32,
        bullet: BulletDef,
        #[serde(default = "one")]
        chance: f32,
    },
    Beam {
        #[serde(default = "default_beam_bullet")]
        bullet: BulletDef,
        #[serde(default = "one")]
        chance: f32,
    },
    ThrowFood {
        #[serde(default = "default_food_bullet")]
        bullet: BulletDef,
        #[serde(default = "one")]
        chance: f32,
    },
    Feast {
        count: u32,
        #[serde(default = "one")]
        chance: f32,
    },
}

fn one() -> f32 {
    1.0
}

#[derive(Clone, Debug, Deserialize)]
pub struct BossPhaseDef {
    pub health_threshold: f32,
    pub attack_interval: f64,
    pub attacks: Vec<AttackDef>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BossDef {
    pub name: String,
    pub max_health: i32,
    pub size: (f32, f32),
    pub sprite: String,
    pub portrait: String,
    pub movement: MovementDef,
    pub phases: Vec<BossPhaseDef>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Content {
    pub enemies: BTreeMap<String, EnemyArchetype>,
    pub waves: BTreeMap<String, WaveDef>,
    pub default_wave: WaveDef,
    pub bosses: BTreeMap<String, BossDef>,
}

// ---- global content store -------------------------------------------------

static CONTENT: AtomicPtr<Content> = AtomicPtr::new(std::ptr::null_mut());

/// Access the content store, loading from disk (with embedded fallback) on first use.
///
/// The returned `&'static` handle points into the store, which is only ever
/// replaced wholesale by [`reload`]. All consumers copy the values they need
/// out at the point of use, so nothing outlives a reload and the raw-pointer
/// swap stays sound.
pub fn load() -> &'static Content {
    unsafe {
        let ptr = CONTENT.load(Ordering::Acquire);
        if ptr.is_null() {
            let fresh = Box::into_raw(Box::new(load_content()));
            match CONTENT.compare_exchange(std::ptr::null_mut(), fresh, Ordering::AcqRel, Ordering::Acquire) {
                Ok(_) => &*fresh,
                Err(prev) => {
                    drop(Box::from_raw(fresh));
                    &*prev
                }
            }
        } else {
            &*ptr
        }
    }
}

/// Re-read the content files from disk, replacing the active store on success.
/// If any file is missing or malformed the current content is left untouched.
///
/// The superseded store is intentionally leaked rather than dropped so that
/// `&'static` handles obtained from earlier [`load`] calls stay valid; each
/// reload leaks a few KB, which is fine for a dev/console tool.
pub fn reload() -> Result<(), String> {
    let fresh = Box::new(load_content_from_disk()?);
    let new = Box::into_raw(fresh);
    CONTENT.store(new, Ordering::Release);
    Ok(())
}

/// Cross-file reference checks: boss hooks, enemy weights, and formation
/// leaders in the wave tables must resolve to entries that actually exist.
pub fn validate() -> Result<(), String> {
    let c = load();
    let mut errors = Vec::new();

    for (wave_key, wave_def) in &c.waves {
        if let Some(boss_key) = &wave_def.boss {
            if !c.bosses.contains_key(boss_key) {
                errors.push(format!("wave {wave_key}: boss \"{boss_key}\" is not defined in bosses.json"));
            }
        }
        for enemy_key in wave_def.enemy_weights.keys() {
            if !c.enemies.contains_key(enemy_key) {
                errors.push(format!("wave {wave_key}: enemy weight \"{enemy_key}\" is not defined in enemies.json"));
            }
        }
        for spec in &wave_def.formations {
            if let FormationSpec::Detailed(DetailedFormation::Escort { leader, .. }) = spec {
                if !c.enemies.contains_key(leader) {
                    errors.push(format!("wave {wave_key}: formation leader \"{leader}\" is not defined in enemies.json"));
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

fn load_content() -> Content {
    let enemies: BTreeMap<String, EnemyArchetype> =
        load_file("assets/content/enemies.json", include_str!("../assets/content/enemies.json"));
    let waves: WavesFile =
        load_file("assets/content/waves.json", include_str!("../assets/content/waves.json"));
    let bosses: BTreeMap<String, BossDef> =
        load_file("assets/content/bosses.json", include_str!("../assets/content/bosses.json"));

    Content {
        enemies,
        waves: waves.waves,
        default_wave: waves.default,
        bosses,
    }
}

fn load_content_from_disk() -> Result<Content, String> {
    let enemies: BTreeMap<String, EnemyArchetype> = load_file_from_disk("assets/content/enemies.json")?;
    let waves: WavesFile = load_file_from_disk("assets/content/waves.json")?;
    let bosses: BTreeMap<String, BossDef> = load_file_from_disk("assets/content/bosses.json")?;

    Ok(Content {
        enemies,
        waves: waves.waves,
        default_wave: waves.default,
        bosses,
    })
}

fn load_file<T: serde::de::DeserializeOwned>(path: &str, embedded: &str) -> T {
    match std::fs::read_to_string(path) {
        Ok(raw) => match serde_json::from_str(&raw) {
            Ok(value) => value,
            Err(err) => {
                eprintln!("[content] failed to parse {path}: {err}; using embedded defaults");
                serde_json::from_str(embedded).expect("embedded content must be valid")
            }
        },
        Err(err) => {
            eprintln!("[content] could not read {path}: {err}; using embedded defaults");
            serde_json::from_str(embedded).expect("embedded content must be valid")
        }
    }
}

fn load_file_from_disk<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, String> {
    let raw = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    serde_json::from_str(&raw).map_err(|e| format!("{path}: {e}"))
}

// ---- lookups -------------------------------------------------------------

pub fn enemy(key: &str) -> Option<&'static EnemyArchetype> {
    load().enemies.get(key)
}

/// Wave definition for `wave`, falling back to the default (endless) table.
pub fn wave(wave: u32) -> &'static WaveDef {
    load().waves.get(&wave.to_string()).unwrap_or(&load().default_wave)
}

pub fn boss(key: &str) -> Option<&'static BossDef> {
    load().bosses.get(key)
}

/// Boss key for the given wave, if that wave is a boss wave.
pub fn boss_for_wave(wave: u32) -> Option<&'static str> {
    load().waves.get(&wave.to_string()).and_then(|w| w.boss.as_deref())
}

// ---- color parsing -------------------------------------------------------

pub fn parse_color(s: &str) -> Color {
    let s = s.trim();
    match s.to_lowercase().as_str() {
        "lightgray" => return macroquad::prelude::LIGHTGRAY,
        "white" => return macroquad::prelude::WHITE,
        "orange" => return macroquad::prelude::ORANGE,
        "skyblue" => return macroquad::prelude::SKYBLUE,
        "gold" => return macroquad::prelude::GOLD,
        "blue" => return macroquad::prelude::BLUE,
        "purple" => return macroquad::prelude::PURPLE,
        "yellow" => return macroquad::prelude::YELLOW,
        "pink" => return macroquad::prelude::PINK,
        "green" => return macroquad::prelude::GREEN,
        "red" => return macroquad::prelude::RED,
        "brown" => return macroquad::prelude::BROWN,
        _ => {}
    }

    if let Some(hex) = s.strip_prefix('#') {
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0);
            }
        }
    }

    if let Some(rest) = s.strip_prefix("rgba(").and_then(|x| x.strip_suffix(')')) {
        let parts: Vec<f32> = rest.split(',').filter_map(|p| p.trim().parse().ok()).collect();
        if parts.len() == 4 {
            return Color::new(parts[0], parts[1], parts[2], parts[3]);
        }
    }

    macroquad::prelude::WHITE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_enemy_archetypes() {
        let c = load();
        assert!(c.enemies.contains_key("scout"));
        assert!(c.enemies.contains_key("fighter"));
        assert!(c.enemies.contains_key("bomber"));
        assert!(c.enemies.contains_key("interceptor"));
        assert!(c.enemies.contains_key("elite"));
        assert_eq!(c.enemies["scout"].base_hp, 1);
        assert_eq!(c.enemies["bomber"].score_value, 50);
    }

    #[test]
    fn test_load_waves() {
        let c = load();
        assert_eq!(c.waves["1"].max_enemies, 6);
        assert_eq!(c.waves["1"].spawn_interval, 2.0);
        assert_eq!(c.waves["15"].boss.as_deref(), Some("captain_davey"));
        assert_eq!(c.waves["5"].boss.as_deref(), Some("blowfish"));
        assert!(c.waves["1"].boss.is_none());
    }

    #[test]
    fn test_wave_default_fallback() {
        assert_eq!(wave(1).max_enemies, 6);
        assert_eq!(wave(5).boss.as_deref(), Some("blowfish"));
        assert_eq!(wave(999).max_enemies, 24);
        assert_eq!(boss_for_wave(10), Some("twofish"));
        assert_eq!(boss_for_wave(4), None);
    }

    #[test]
    fn test_load_boss_defs() {
        let c = load();
        assert!(c.bosses.contains_key("blowfish"));
        assert_eq!(c.bosses["blowfish"].max_health, 150);
        assert_eq!(c.bosses["captain_davey"].max_health, 500);
        assert_eq!(c.bosses["blowfish"].phases.len(), 3);
        assert_eq!(c.bosses["blowfish"].phases[0].attacks.len(), 1);
        match &c.bosses["blowfish"].phases[0].attacks[0] {
            AttackDef::Burst { count, .. } => assert_eq!(*count, 8),
            other => panic!("expected Burst, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_colors() {
        let c = parse_color("orange");
        assert_eq!(c, macroquad::prelude::ORANGE);

        let c = parse_color("rgba(0.0, 0.8, 0.0, 0.5)");
        assert_eq!(c.g, 0.8);
        assert_eq!(c.a, 0.5);

        let c = parse_color("notacolor");
        assert_eq!(c, macroquad::prelude::WHITE);
    }

    #[test]
    fn test_validate_passes_on_committed_content() {
        assert!(validate().is_ok(), "validate() should pass: {}", validate().unwrap_err());
    }

    #[test]
    fn test_reload_reparses_disk_content() {
        assert!(reload().is_ok(), "reload() should succeed: {}", reload().unwrap_err());
        assert_eq!(load().waves["1"].max_enemies, 6);
        assert_eq!(load().bosses["blowfish"].max_health, 150);
        assert!(load().bosses.contains_key("captain_davey"));
    }
}
