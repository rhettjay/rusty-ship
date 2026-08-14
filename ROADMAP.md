# Rusty Ship - Development Roadmap

```latex
hackerz4 G$\sqrt{oo}d
```

## Current State (v0.1.0)
- Basic space shooter with ship, cannonballs, pirates
- Main menu with Start/Settings/Quit
- Config system for all game parameters
- Pirate ships rotated 180°
- Game over / restart flow

---

## Phase 1: Core Mechanics Enhancement

### Multi-Cannon System
- [ ] Ship has array of cannon positions (left, center, right)
- [ ] Each cannon fires independently with offset
- [ ] Visual indication of active cannons on ship sprite

### Enemy Ship Variety
- [ ] **Scout** - Fast, low HP, zigzag pattern
- [ ] **Bomber** - Slow, high HP, drops mines
- [ ] **Interceptor** - Medium, shoots back
- [ ] **Capital Ship** - Boss, multiple hit zones

### Hacking Minigame Foundation
- [ ] Collision with disabled enemy triggers hack state
- [ ] Simple prompt: "Press [KEY] to hack"
- [ ] Success/fail callbacks

---

## Phase 2: Fleet Building System

### Fleet Mechanics
- [ ] Captured ships trail behind player
- [ ] Each fleet ship = +1 cannon auto-fire
- [ ] Fleet ships have own health (shared lives pool)
- [ ] Visual formation: wedge, line, circle

### Fleet Management
- [ ] Fleet UI showing ship icons + health
- [ ] Lose fleet ship when lives depleted
- [ ] Max fleet size configurable (config.rs)

### Auto-Fire System
- [ ] Fleet cannons fire at intervals
- [ ] Staggered timing for spread
- [ ] Target nearest enemy

---

## Phase 3: Hacking Minigame

### Minigame Types (rotating)
1. **Command Injection** - Type commands before timer expires
2. **Pattern Match** - Repeat arrow sequence (Simon Says)
3. **Memory Dump** - Find matching pairs in grid
4. **Port Scan** - Stop rotating dial on green zones

### Hacking UI
```
┌─ SECURITY BREACH ─┐
│ TARGET: CR-7 KESTREL │
│ FIREWALL: ████░░░░ 42% │
│                     │
│ > sudo inject       │
│   payload.bin       │
│                     │
│ [ENTER] EXECUTE     │
└─────────────────────┘
```

### Risk/Reward
- [ ] Success: Capture ship, add to fleet
- [ ] Partial: Steal tech (permanent upgrade)
- [ ] Fail: Enemy self-destructs, damage player
- [ ] Critical fail: Lose fleet ship

---

## Phase 4: Progression & Meta

### Wave System
- [ ] Waves increase enemy count/speed/types
- [ ] Every 5 waves = boss encounter
- [ ] Wave cleared = upgrade screen

### Upgrade Shop (between waves)
- [ ] **Hull** - +1 max lives
- [ ] **Engines** - +ship speed
- [ ] **Targeting** - +cannonball speed/pierce
- [ ] **Hack Suite** - +hack time, -difficulty
- [ ] **Fleet Bay** - +max fleet size

### Persistence
- [ ] Save fleet composition to disk
- [ ] Load on startup
- [ ] High score table

### Boss Ships
- [ ] Multi-phase encounters
- [ ] Require hacking specific subsystems
- [ ] Unique mechanics per boss

---

## Phase 5: Polish & Content

### Visual
- [ ] Particle effects (explosions, engine trails)
- [ ] Screen shake on impact
- [ ] Hacking terminal animations
- [ ] Fleet formation transitions

### Audio
- [ ] Enable macroquad audio feature
- [ ] Procedural synth for retro sounds
- [ ] Music tracks per wave tier

### Config Expansion
- [ ] Difficulty presets (Easy/Normal/Hard/Rogue)
- [ ] Accessibility options
- [ ] Key rebinding

---

## Technical Debt

- [ ] Collision system using Hitbox struct
- [ ] Entity-component system for scalability
- [ ] Asset hot-reload for development
- [ ] Unit tests for logic.rs

---

## Config Reference (`src/config.rs`)

```rust
pub struct GameConfig {
    // Display
    pub window_width: f32 = 800.0;
    pub window_height: f32 = 800.0;
    
    // Player Ship
    pub ship_speed: f32 = 5.0;
    pub ship_width: f32 = 60.0;
    pub ship_height: f32 = 64.0;
    pub ship_start_y_offset: f32 = 100.0;
    
    // Cannons
    pub cannonball_speed: f32 = 10.0;
    pub cannonball_width: f32 = 5.0;
    pub cannonball_height: f32 = 15.0;
    pub cannonball_cooldown: f64 = 0.2;
    
    // Pirates/Enemies
    pub pirate_base_speed_x_range: (f32, f32) = (1.0, 8.0);
    pub pirate_base_speed_y_range: (f32, f32) = (1.0, 3.0);
    pub pirate_spawn_chance: u32 = 25;  // 1 in N frames
    pub pirate_max_count: i32 = 10;
    pub pirate_width: f32 = 15.0;
    pub pirate_height: f32 = 15.0;
    
    // Game Balance
    pub starting_lives: i32 = 3;
    pub starting_pirate_count: i32 = 10;
}
```

---

## Build & Run

```bash
source ~/.cargo/env && cargo run
```

## Controls

| Key | Action |
|-----|--------|
| ←/→ | Move ship |
| SPACE | Fire cannon |
| ↑/↓ | Menu navigation |
| ENTER | Menu select |
| ESC | Quit / Return to menu |
