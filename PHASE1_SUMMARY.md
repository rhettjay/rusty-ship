# Phase 1: Enemy/Wave System Rearchitecture - Complete

## Overview
Successfully rearchitected the baseline wave system from a single pirate type to a 4-type enemy system with wave progression, formations, power-ups, and intentional difficulty curve. All visuals use pixel art sprites (currently colored placeholders).

---

## Files Created/Modified

### New Modules
| File | Lines | Purpose |
|------|-------|---------|
| `src/enemy.rs` | ~480 | Enemy types, HP/armor scaling, behavior states |
| `src/bullet.rs` | ~390 | Bullet types (player/enemy), procedural rendering |
| `src/powerup.rs` | ~410 | Power-up types, effects, drop system |
| `src/formation.rs` | ~290 | Formation math (Vee, Line, Circle, Escort, Grid, Chaos) |
| `src/wave_director.rs` | ~240 | Wave scheduling, spawning, boss transitions |

### Modified Files
| File | Changes |
|------|---------|
| `src/config.rs` | Added WaveConfig, EnemyConfig, PowerupConfig, PowerupType |
| `src/ship.rs` | Added power-up timers, shield, hitbox, power-up application |
| `src/cannonball.rs` | Added damage, active/dead state, hitbox |
| `src/pirate.rs` | Added width/height, get_rect() |
| `src/boss.rs` | Added get_rect(), BossProjectile::get_rect() |
| `src/main.rs` | Full integration with wave_director, collision system, power-up pickups |

---

## Enemy System

### 4 Baseline Types + Elite Variants

| Type | Base HP | Armor | Speed | Size | Behavior | Sprite |
|------|---------|-------|-------|------|----------|--------|
| **Scout** | 1 | 0 | Fast | 24×24 | No shoot, erratic horizontal | `scout.png` |
| **Fighter** | 2 | 0 | Med | 32×32 | Straight shots, sine wave | `fighter.png` |
| **Bomber** | 3 | 1 | Slow | 40×40 | Arcing bombs, straight line | `bomber.png` |
| **Interceptor** | 2 | 0 | Fast | 28×28 | Aimed shots, dive attacks | `interceptor.png` |
| **Elite** (variant) | +2 HP | +2 | Same | Same | Enhanced patterns | Palette swap + glow |

**Wave Scaling:**
```
HP = base_hp + floor(wave / 3)
Armor = base_armor + floor(wave / 5)
```

---

## Wave Structure (15 Waves + Endless)

| Wave | Duration | Types | Formation | Spawn Interval | Notes |
|------|----------|-------|-----------|----------------|-------|
| 1 | 20s | Scout | Random | 2.0s | Tutorial |
| 2 | 25s | Scout | Random | 1.5s | Density up |
| 3 | 30s | Scout+Fighter | Vee(3) | 1.8s | First formation |
| 4 | 30s | Scout+Fighter | Line(4) | 1.5s | Horizontal sweep |
| **5** | **BOSS** | **Blowfish** | — | — | **Boss wave** |
| 6 | 35s | All 4 | Line(5) | 1.2s | Full roster |
| 7 | 35s | All 4 | Circle(6) | 1.0s | Surround player |
| 8 | 40s | +Interceptor | Escort | 1.0s | Protected bomber |
| 9 | 40s | Heavy Fighter | Vee(5) | 0.8s | Pressure |
| **10** | **BOSS** | **Twofish** | — | — | **Boss wave** |
| 11 | 45s | +Elite | Grid/Circle | 0.9s | Elites 10% |
| 12 | 45s | Elite heavy | Circle(8) | 0.7s | Elites 20% |
| 13 | 50s | All + Elite | Chaos | 0.6s | Survival |
| 14 | 50s | Elite swarm | Random | 0.5s | Gauntlet |
| **15** | **BOSS** | **Captain Davey** | — | — | **Final boss** |
| 16+ | ∞ | Endless | All | 0.4s | Score attack |

---

## Difficulty Levers (Intentional Design)

| Lever | Early (1-4) | Mid (6-9) | Late (11-14) | Purpose |
|-------|-------------|-----------|--------------|---------|
| **Time pressure** | 20-30s | 35-40s | 45-50s | Prevents camping |
| **Spawn interval** | 2.0→1.5s | 1.2→0.8s | 0.6→0.4s | Density ramp |
| **Type diversity** | 1→2 | 3→4 | 4 + Elite | Cognitive load |
| **Formation complexity** | None → V/Line | Circle/Escort | Mixed/Chaos | Pattern recognition |
| **Power-up frequency** | 8% | 5% | 3% | Resource management |
| **Armor/HP scaling** | 0-1 | 1-2 | 2-4 | Damage checks |

---

## Power-Up System (Temporary Effects)

| Power-Up | Duration | Effect | Weight | Drop Rate |
|----------|----------|--------|--------|-----------|
| **Rapid Fire** | 10s | Cooldown ×0.3 | 25% | Base 5% + wave bonus |
| **Spread Shot** | 10s | 3-way (-20°/+20°) | 20% | Elite: 3× base |
| **Pierce** | 10s | Pierces 3 enemies | 15% | Bomber: 2× base |
| **Shield** | 1 hit | Blocks 1 hit | 15% | |
| **Bomb** | Instant | Clear projectiles + damage all | 10% | |
| **Life** | Permanent | +1 life (max 9) | 10% | |
| **Score** | Instant | +500 pts | 5% | |

---

## Formation System

| Formation | Waves | Description |
|-----------|-------|-------------|
| Random | 1-2 | Pure random spawns |
| Vee | 3, 9 | Arrow formation, fighters at tips |
| Line | 4, 6 | Horizontal sweep, alternating types |
| Circle | 7, 12 | Surround player, interceptors |
| Escort | 8 | Bomber leader + fighter followers |
| Grid | 11, 12 | Structured rows/cols by type |
| Chaos | 13, 14 | Mixed types, high density |

**Progression:**
- Waves 1-2: 100% Random
- Waves 3-4: 50% Scripted (V/Line), 50% Random
- Waves 6-9: 70% Scripted, 30% Random
- Waves 11-14: 80% Scripted, 20% Random
- Wave 16+: 50/50 (endless variety)

---

## Bullet System

### Player Bullets
| Type | Speed | Damage | Pierce | Visual |
|------|-------|--------|--------|--------|
| Standard | 12 | 1 | 0 | Yellow rectangle + white core |
| Rapid | 14 | 1 | 0 | Orange (fast ROF via power-up) |
| Spread | 10 | 1 | 0 | Cyan triangle (3-way) |
| Pierce | 11 | 1 | 3 | Green rectangle + white outline |
| Laser | ∞ | 2/s | 999 | Pink beam |

### Enemy Bullets
| Type | Speed | Pattern | Source |
|------|-------|---------|--------|
| Straight | 5 | ↓ | Fighter |
| Aimed | 6 | →player | Interceptor |
| Bomb | 3 | Arc ↓ | Bomber |
| Spread | 5 | 3-way | Elite Fighter |

---

## Asset Structure (Created)

```
assets/
├── enemies/
│   ├── scout.png           # 24×24 (LIGHTGRAY)
│   ├── fighter.png         # 32×32 (WHITE)
│   ├── bomber.png          # 40×40 (ORANGE)
│   └── interceptor.png     # 28×28 (SKYBLUE)
├── bullets/
│   ├── player_standard.png  # 8×16 (YELLOW)
│   ├── player_spread.png    # 8×16 (CYAN)
│   ├── player_pierce.png    # 8×18 (LIME)
│   ├── player_laser.png     # 4×32 (PINK)
│   ├── enemy_straight.png   # 10×10 (RED)
│   ├── enemy_aimed.png      # 10×10 (ORANGE)
│   ├── enemy_bomb.png       # 16×16 (PURPLE)
│   ├── enemy_spread.png     # 10×10 (PINK)
│   ├── enemy_fast.png       # 8×8 (WHITE)
│   └── enemy_laser.png      # 4×32 (RED)
├── powerups/
│   ├── rapid_fire.png       # 24×24 (YELLOW)
│   ├── spread_shot.png      # 24×24 (CYAN)
│   ├── pierce.png           # 24×24 (LIME)
│   ├── shield.png           # 24×24 (WHITE)
│   ├── bomb.png             # 24×24 (RED)
│   ├── life.png             # 24×24 (PINK)
│   └── score.png            # 24×24 (GOLD)
└── effects/
    ├── explosion_small.png  # 32×32
    ├── explosion_medium.png # 48×48
    ├── hit_spark.png        # 16×16
    └── shield_break.png     # 32×32
```

*Currently all colored rectangles - replace with actual pixel art*

---

## Integration Status

✅ **Compiles and runs** (`cargo run --release`)

### Working Systems
- Wave progression (1-15 + endless)
- Enemy spawning with formation support
- 4 enemy types + elite variants
- HP/armor scaling per wave
- Power-up drops and pickup
- Temporary power-up effects (Rapid Fire, Spread, Pierce, Shield)
- Collision detection (bullets ↔ enemies, enemies ↔ player)
- Boss wave transitions (waves 5, 10, 15)
- Dialogue system integration
- Score/lives tracking

### Known Limitations (Placeholders)
- All sprites are colored rectangles
- No audio files (menu_music.ogg, gameplay_music.ogg, etc.)
- No particle effects / screen shake
- Some dead code warnings from legacy systems

---

## Next Steps (Future Phases)

| Phase | Focus | Est. Effort |
|-------|-------|-------------|
| **2** | Pixel art creation (enemies, bullets, powerups) | Medium |
| **3** | Audio implementation (music tracks, SFX) | Medium |
| **4** | Boss pattern polish + balance | Medium |
| **5** | Visual effects (particles, screen shake, hit flash) | Low |
| **6** | Playtesting + difficulty tuning | Medium |
| **7** | New Game+ / endless mode scoring | Low |

---

## Testing Checklist

- [x] Wave 1-2: Scout only, random spawn
- [x] Wave 3-4: Vee/Line formations with Fighters
- [x] Wave 5: Blowfish boss intro → fight → defeat dialogue
- [x] Wave 6-9: Full roster, Circle/Escort formations
- [x] Wave 10: Twofish boss
- [x] Wave 11-14: Elites, Grid/Chaos formations
- [x] Wave 15: Captain Davey final boss → Victory screen
- [x] Power-up spawns and pickup
- [x] Power-up effects apply correctly
- [x] Shield blocks 1 hit
- [x] Bomb clears screen
- [x] Score/lives persist across waves
- [x] Game Over → restart works
- [x] Victory → restart works

---

## Build Commands

```bash
# Development
cd /Users/bucklup/rusty-ship && cargo run

# Release (optimized)
cd /Users/bucklup/rusty-ship && cargo run --release

# Check only
cd /Users/bucklup/rusty-ship && cargo check
```

---

*Phase 1 Complete - Ready for art/audio integration*
*Date: $(date)*
*Branch: main*