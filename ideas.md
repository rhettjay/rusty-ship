Building a retro space shooter (often called a "shmup" or shoot 'em up) with a pirate theme is a fantastic project. It’s highly modular, meaning you can get a basic version running quickly and add complexity over time.

Here is a step-by-step roadmap on how to go about building it, from architecture to the "retro" feel.

---

### Phase 1: Choose Your Engine
For a 2D retro game, you want an engine that handles pixel art, sprite batching, and simple collisions effortlessly.
*   **Godot (Highly Recommended):** Free, lightweight, and incredible for 2D. Its GDScript language is very beginner-friendly, and it has built-in tools for tilemaps and pixel-perfect rendering.
*   **Unity:** The industry standard. Great 2D capabilities, C# is a powerful language, and there are thousands of shmup tutorials on YouTube.
*   **GameMaker:** Built specifically for 2D. It uses a drag-and-drop system with optional coding (GML). Many classic indie retro games (like *Hotline Miami*) were made here.

### Phase 2: The Core Gameplay (The "Grey Box" Phase)
Do not worry about art or pirates yet. Use colored squares and rectangles.
1.  **Player Movement:** Implement 8-way movement or strict 4-way. Make sure the player is **clamped** to the screen boundaries.
2.  **Shooting & Object Pooling:** *Crucial step.* Do not "instantiate" (create) and "destroy" bullets every time you shoot. This will cause lag. Instead, create an **Object Pool**—a pre-made array of, say, 50 bullet objects that turn invisible when off-screen and get recycled.
3.  **Hitboxes:** In retro shmups, the player's *visual* ship is much larger than their *actual* hitbox (usually a 3x3 pixel dot in the center). This makes the game feel challenging but fair.

### Phase 3: Designing the Pirates (Enemy Architecture)
You want variety, so design enemies as "base" objects with interchangeable behaviors.
*   **The Grunt:** Flies straight down. Maybe fires a single shot forward.
*   **The Strafer:** Enters from the side, moves in a sine-wave pattern.
*   **The Heavy:** Moves slowly, takes 5 hits, fires a 3-bullet spread.
*   **The Ace:** Fast, aims directly at the player (requires a simple math function to calculate the angle between the enemy and player).
*   *Tip:* Use a State Machine (`Idle`, `Entering_Screen`, `Attacking`, `Dying`) for enemies so their code doesn't become a messy web of `if/else` statements.

### Phase 4: The Wave System
Don't hardcode waves. Use a Data Structure (like a JSON file, a Dictionary, or an Array) to define them. A wave manager should operate on a simple State Machine:
1.  **WAVE_INTRO:** Show text "Wave 1". (Wait 2 seconds).
2.  **SPAWNING:** Read from your data: *"Spawn 5 Grunts, wait 1 second, spawn 2 Heavies."*
3.  **ACTIVE:** The player fights. The wave does not end until all enemies are dead *and* all enemy bullets are off-screen.
4.  **WAVE_CLEARED:** Brief pause, maybe drop a power-up, trigger `WAVE_INTRO` for Wave 2.

*Example Wave Data structure:*
```json
{
  "wave_number": 1,
  "spawns": [
    { "enemy_type": "grunt", "delay": 0.0 },
    { "enemy_type": "grunt", "delay": 0.5 },
    { "enemy_type": "heavy", "delay": 2.0 }
  ]
}
```

### Phase 5: Power-Ups & Progression
To keep wave progression interesting, the player needs to get stronger while the enemies get harder.
*   **Weapon Upgrades:** Faster fire rate, double shot, triple shot, rear-firing shot.
*   **Pirate Theme Twist:** Instead of generic power-ups, collect "Plunder." Spend Plunder between waves at a "Space Tavern" to upgrade your ship's shields, speed, or weapons.

### Phase 6: Nailing the "Retro" Feel
This is where your game stops feeling like a tech demo and starts feeling like an arcade cabinet.
1.  **Low Resolution Rendering:** Render your game at a low internal resolution (e.g., 320x240 or 384x216), then scale it up to fit modern monitors. This gives automatic, crispy pixels without manually drawing huge sprites.
2.  **Screen Shake:** Add a quick screen shake (translating the camera randomly for 0.1 seconds) when an enemy explodes or the player takes damage.
3.  **Hit Pause (Frame Freeze):** When the player lands a killing blow on a heavy enemy, freeze the game for exactly 2-3 frames. It adds immense weight to the shots.
4.  **CRT Shader:** Apply a simple post-processing shader to add scanlines and very slight screen curvature.
5.  **Audio:** Use square waves and triangle waves for sound effects. Keep the music fast-paced (chiptune or synthwave).

### Phase 7: Boss Fights
Every 5th or 10th wave should be a Boss. 
*   Bosses should have multiple **Phases**. 
*   *Phase 1:* Fires spread shots. 
*   *Phase 2 (at 50% health):* Spawns smaller pirate minions while firing lasers. 
*   *Phase 3 (at 10% health):* Goes berserk, fires faster, but bullets might be slower.

### Your Immediate Next Steps (Homework):
1.  Download **Godot** or **Unity**.
2.  Look up a 15-minute tutorial on *"Top-down 2D player movement"* and *"Shooting projectiles"*.
3.  Get a square moving on screen and shooting other squares. 
4.  Once that feels fun, you have your foundation. Everything else (art, waves, pirates) is just decorating that foundation.

*Search terms to help you find specific tutorials:* "Shmup tutorial [Engine Name]", "Object pooling bullets [Engine Name]", "Pixel perfect rendering 2D [Engine Name]".

---

## Issues found during session 2026-07-30

### Enemy spawning
- Enemies spawned at y=-50 with entry_target_y set to spawn y (also -50), so entry animation instantly finished off-screen
- Formation drift was `speed_y * dt * 0.3` = ~0.005 px/frame — took ~3 minutes to become visible
- **Fix:** entry_target_y randomized 80–200, drift changed to sinusoidal bob

### Wave progression
- State edge-detection for wave completion captured state *after* `wave_director.update()`, so `prev == current` on transition frames — never detected
- After wave reached Complete, no code called `start_wave(next)` — game sat idle
- **Fix:** capture state before update; auto-advance to next wave when no dialogue triggers

### Boss waves
- `boss_waves` in config was `[3, 6, 9, 12, 15]` but `wave_director.rs` used `matches!(wave, 5 | 10 | 15)` — mismatch
- Auto-advancing into a boss wave left `wave_state = BossIntro` with `game.state = Playing` — softlock
- **Fix:** config aligned to `[5, 10, 15]`; auto-advance transitions to `GameState::BossIntro`; boss defeat triggers `BossDefeated` narrative → defeat dialogue → `NextChapter`

### Powerups
- `vel_y` was `rng.gen_range(0.5..1.5)` then multiplied by `dt` — ~0.008 px/s, 28 min to reach ship
- **Fix:** changed to `60.0..120.0` — crosses screen in 6–12 seconds

### Miscellaneous bugs fixed
- `formation_offset` had `target_y - self.x` instead of `self.y`
- Menu navigation used `% 4` hardcoded but "No save" showed 3 items
- Console `heal`/`powerup` set ship timer fields but shooting code checked manager timers
- Double cannonball draw in Dialogue state
- Chapter number incremented twice on boss defeat (narrative trigger + callback)

---

## Ideas for future beef-ups

### Visual feel (high leverage, low code)
- **Screen shake:** translate draw offset randomly for 0.1s on enemy death / player hit (~20 lines)
- **Hitstop / freeze frame:** pause game for 2–3 frames on killing blow to add weight
- **Explosion particles:** spawn 5–10 fading circles at enemy death position
- **Score popups:** floating "+10"/"+50" text at enemy death with upward drift and fade
- **Player hit flash:** brief invulnerability + sprite flash on damage (1–2s invuln window)
- **Boss entrance animation:** slow dramatic float-in with pause, not instant `y += 2`

### Gameplay
- **Enemy dive timer:** in `update_regular_wave`, periodically set random enemy to `BehaviorState::Diving` — dive code already exists, just needs a trigger
- **Enemy variety through existing types:** Scouts move fast, Bombers drop gravity-affected bombs, Interceptors aim at player — all already implemented, just need tuning
- **Wave transition animation:** brief flash / zoom between waves

### Code hygiene
- **Delete `state.rs`:** dead ggez-era file with typos (`prelud`). Will break build if `mod state;` is added
- **Wire up `FormationManager`:** exists in `formation.rs` with Vee/Line/Circle/Grid/Chaos patterns but `WaveDirector` spawns at random x instead
- **Simplify enemy spawning:** `FormationManager.start_formation` → `update` gives positioned formations. Could replace the inline random spawns
- **Sprite loading:** currently all entities draw as colored rectangles. Add `include_bytes!` sprite loading at init for enemies, bullets, powerups (same pattern as `background.png` and `ship.png`)
