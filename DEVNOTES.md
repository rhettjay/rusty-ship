# Dev Notes

## Debug Console

Open with `/` while playing. Type `help` for list.

### Planned commands

| Command | Status | Description |
|---------|--------|-------------|
| `help` | Done | Show available commands |
| `god` | Done | Toggle invincibility |
| `heal` | Done | Full heal + powerups |
| `wave <n>` | Done | Jump to wave n (1-15) |
| `score <n>` | Done | Set score |
| `lives <n>` | Done | Set lives |
| `spawn <enemy>` | Done | Spawn enemy (scout/fighter/bomber/interceptor/elite) |
| `killall` | Done | Kill all enemies |
| `powerup <type>` | Done | Give powerup (rapid/spread/pierce/shield) |
| `quit` | Done | Quit to main menu |
| `spawn <boss>` | Added | Spawn any boss by name |
| `hitbox` | Added | Toggle hitbox overlay |
| `time <scale>` | Added | Set time scale (0.x slow-mo, 2.x fast) |
| `state` | Added | Print current game state |
| `damage <boss> <n>` | Added | Damage current boss by n |
| `reload` | Added | Reload `assets/content/*.json` from disk + validate |

## Test Harness

Run with: `cargo test`

| Module | What's tested |
|--------|---------------|
| `collision::tests` | Rect overlap detection (edge cases, containment, no collision) |
| `boss::tests` | Boss creation, take_damage, phase transitions, invulnerability |
| `wave_director::tests` | State machine transitions through wave lifecycle |
| `content::tests` | JSON loading (enemies/waves/bosses), wave fallback, color parsing, validation, reload |

## Content Data

Gameplay balance lives in `assets/content/` as JSON, loaded at startup (no recompile needed):

- `enemies.json` — enemy archetypes (hp, armor, speed, size, shoot pattern, score, powerup chance, color, sprite)
- `waves.json` — per-wave tables (`default` = endless mode): duration, max enemies, spawn interval, formations, enemy weights, optional `boss` key (waves 5, 10, 15)
- `bosses.json` — boss definitions: health, size, sprites, movement kind, and health-threshold phases with attack lists

Use the in-game console command `reload` to re-read these files while playing. If a file fails to parse, the game keeps the last good content and prints the error.
