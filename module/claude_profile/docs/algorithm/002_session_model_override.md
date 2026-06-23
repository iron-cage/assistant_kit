# Algorithm: Session Model Override

### Purpose

Bidirectionally manage the interactive session model in `~/.claude/settings.json` based on Sonnet quota utilization. Upgrades from Opus back to Sonnet when quota recovers; downgrades to Opus when near-exhausted.

### Entry Points

- `src/usage/api.rs` — `apply_model_override(aq)` (mutation)
- `src/usage/format.rs` — `recommended_model(aq)` (read-only, for footer recommendation)

### Decision Table

| `seven_day_sonnet` | Sonnet remaining (`100 - utilization`) | Current model | Action |
|---|---|---|---|
| `None` | — | Opus form | **→ Sonnet** (absent tier ≠ exhausted; restore conservatively — Fix BUG-311) |
| `None` | — | Sonnet form | No-op |
| `Some` | ≥ 15% | Opus form | **→ Sonnet** (sufficient capacity — Fix BUG-311) |
| `Some` | ≥ 15% | Sonnet form | No-op |
| `Some` | < 15% | Sonnet form | **→ Opus** (near-exhausted — preserve remaining tokens) |
| `Some` | < 15% | Opus form | No-op |

"Opus form" = model string matches `claude-opus-*` or `"opus"`.
"Sonnet form" = model string matches `claude-sonnet-*` or `"sonnet"`.

### Threshold

5% from `recommended_model()` in `format.rs` (canonical) — actual gate: `seven_day_sonnet.utilization >= 85.0` (i.e., ≤ 15% remaining).

### Bug History

- **BUG-300 (Fix TSK-302):** `map_or(0.0, ...)` on `seven_day_sonnet = None` returned 0.0 < threshold → Opus override fired unconditionally for accounts without Sonnet tier. Fix: `if let Some(ref sonnet)` guard.
- **BUG-311 (Fix 2026-06-23):** one-way ratchet — only wrote "opus" (exhaustion), never restored "sonnet" (recovery). Fix: added `else`-branch calling `override_session_model_to_sonnet()`. Tier-absent path also writes "sonnet" conservatively.

### Relationship to `recommended_model()`

`recommended_model(aq)` in `format.rs` is the **canonical threshold function** — it returns the recommended model string without writing to disk. `apply_model_override()` calls this function to determine direction, then calls `set_session_model()` or `override_session_model_to_sonnet()`. The footer `Next` line uses `recommended_model()` directly.

### Cross-References

| File | Relationship |
|------|-------------|
| [feature/009_token_usage.md](../feature/009_token_usage.md) | AC-32: bidirectional override semantics |
| [feature/034_explicit_session_model_override.md](../feature/034_explicit_session_model_override.md) | `set_session_model()` / `get_session_model()` |
| [feature/062_unified_session_config.md](../feature/062_unified_session_config.md) | `recommended_model()` canonical entry point |
| [feature/039_decision_algorithms.md](../feature/039_decision_algorithms.md) | Table 2 (legacy reference) |
| [schema/006](../schema/006_settings_json.md) | `model` and `effortLevel` fields in `settings.json` |
