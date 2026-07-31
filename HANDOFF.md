# Handoff — Rusty Ship

**Session:** content-driven data refactor + console input fixes
**Date:** 2026-07-30
**Head:** `be7dd1a` (main)

---

## What changed this session

1. **Data-driven game content** (`95df3ca`) — enemy archetypes, wave tables, and
   boss definitions moved out of hardcoded Rust tables into JSON:
   - New: `assets/content/{enemies,waves,bosses}.json`, `src/content.rs`
     (serde structs, `load()`/`wave()`/`enemy()`/`boss()`/`boss_for_wave()`,
     embedded fallback so the game always boots).
   - `src/enemy.rs`, `src/wave_director.rs`, `src/formation.rs`, `src/boss.rs`
     now hydrate from content; hardcoded `WaveConfig`/`EnemyConfig`/boss stats
     deleted from `src/config.rs`.
   - Boss attacks/movement/phases are fully parameterized (`AttackDef`,
     `MovementDef`, `BossPhaseDef`) and driven by `bosses.json`.
2. **Live content reload** (`95df3ca`) — console command `reload` re-reads the
   JSON from disk (keeps old content if parse fails) and runs cross-file
   validation. Boot also validates and eprints issues.
3. **Console input fixes** (`864ba4b`):
   - UTF-8 char-boundary panics (byte-index cursor + multi-byte chars) — cursor
     is now a char index converted via `menu::char_to_byte`.
   - Input buffer now clears after every command (previously only on `quit`).
4. **Backlog created** (`be7dd1a`) — see `BACKLOG.md`:
   - **F-01** Vertical player movement (feature; needs tilting ship art later).
   - **F-02** Boss bullets don't threaten the player (bug, future release;
     diagnosis inside: low bullet speeds in `bosses.json`, damped Spiral/Bounce
     math in `src/boss.rs::update_projectiles`).

## Current state

- `cargo build` clean, `cargo test` 35/35 passing.
- `make dev` = `cargo check && cargo test`; `make run` to play.
- Gameplay balance lives in `assets/content/*.json` — edit + `reload` in the
  debug console (`/` to open) — no recompile needed.
- Wave → boss mapping: 5→`blowfish`, 10→`twofish`, 15→`captain_davey`.

## Untracked / ignored (not committed, by design)

- `savegame.bin` (runtime save artifact)
- `session-ses_04ac.md` (session log; repo convention keeps these out of git)

## Next steps (when picking up)

1. **F-02 (boss bullet threat)** — tune `speed`/`spread`/`lifetime` per attack
   in `assets/content/bosses.json` (data-first) and repair
   `ProjectilePattern::Spiral`/`Bounce` in `src/boss.rs`. Re-verify against F-01
   once vertical movement exists.
2. **F-01 (vertical movement)** — add up/down to `src/ship.rs` (only
   `left()`/`right()` exist today), clamp to screen/HUD, update controls docs.
3. Pre-existing repo debt (see `ROADMAP.md` Technical Debt): dead
   `logic.rs`/`state.rs` modules, unwired `FormationManager` (formation data in
   JSON is validated but not consumed by the live game), ~70 warnings largely
   from those dead paths.
