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

## Test Harness

Run with: `cargo test`

| Module | What's tested |
|--------|---------------|
| `collision::tests` | Rect overlap detection (edge cases, containment, no collision) |
| `boss::tests` | Boss creation, take_damage, phase transitions, invulnerability |
| `wave_director::tests` | State machine transitions through wave lifecycle |
