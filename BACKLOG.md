# Rusty Ship — Backlog

Unplanned work (features, bugs) queued for future releases. Planned milestones
and the long-term roadmap live in [ROADMAP.md](./ROADMAP.md); shipped
console/tooling notes live in [DEVNOTES.md](./DEVNOTES.md).

Statuses: `open` → `in progress` → done (strikethrough when shipped).

---

## Features

### F-01: Vertical player movement

- **Status:** `open`
- **Raised:** 2026-07-30

Allow the player ship to move **up and down** as well as left/right. Today the
ship is clamped to a fixed `y` (only `Ship::left` / `Ship::right` exist in
`src/ship.rs`), which makes dodging boss patterns that aim at the player's lane
nearly impossible once bullets have a real spread.

**Acceptance criteria:**
- [ ] Up/Down (and diagonals) move the ship; position clamped to screen bounds.
- [ ] Vertical range respects the HUD area (no flying into the score bar).

**Future / follow-on:** directional movement needs vector graphics — a ship
sprite that tilts (pitch/roll) with movement direction instead of the single
static texture used today. That is a separate visual-work item once the art
assets exist.

---

### F-02: Boss bullets do not threaten the player

- **Status:** `open` (bug, scheduled for a future release)
- **Raised:** 2026-07-30

Boss attacks look busy but are not dangerous: bullets cluster near the boss and
fail to spread out far enough to reach the player, so the player can sit in one
spot and never take a hit.

**Diagnosis (from `assets/content/bosses.json` + `src/boss.rs`):**
- Bullet `speed` is 4–5 px/frame (≈240–300 px/s at 60fps) across most attacks
  on an 800px-tall screen; with `lifetime` 3–8s most bullets still reach the
  bottom, but the *cluster* of aimed/scatter bullets is narrow.
- `ProjectilePattern::Spiral` (boss.rs `update_projectiles`) recomputes velocity
  from `sin`/`cos` of the previous velocity each frame, which damps and
  oscillates the bullet around ~4 px/frame instead of producing a real outward
  spiral.
- `Bounce` only gains gravity (`vel_y += 0.3`), so it drifts instead of
  spreading.
- Burst/ring patterns start from the boss center, so at low speed the bullets
  spend most of their lifetime near the top of the screen.

**Fix direction (when scheduled):** tune `speed`/`spread`/`lifetime` per attack
in `bosses.json` (data-first, no recompile), and repair the Spiral and Bounce
patterns in `src/boss.rs` so they produce genuinely wider, faster bullet fans.
Re-verify against the player's new vertical mobility (F-01).

---
