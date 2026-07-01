# Algorithm: Session Model Override

### Purpose

Bidirectionally manage the interactive session model in `~/.claude/settings.json` based on Sonnet quota utilization. Upgrades from Opus back to Sonnet when quota recovers; downgrades to Opus when near-exhausted.

### Entry Points

- `src/usage/api.rs` — `apply_model_override(quota, paths, trace, label, name)` (mutation; `quota: &OauthUsageData`)
- `src/usage/format.rs` — `recommended_model(aq)` (read-only, for footer recommendation; `aq: &AccountQuota`)

### Decision Table

Effort is written **unconditionally** on every call to `apply_model_override()`, regardless of whether the model actually changed (`overrode` value). This ensures effort always matches the model state even for stable sessions (TSK-335 — Fix H2).

| `seven_day_sonnet` | Sonnet remaining (`100 - utilization`) | Current model | Action | Effort (unconditional write) |
|---|---|---|---|---|
| `None` | — | Opus form | **→ Sonnet** (absent tier ≠ exhausted; restore conservatively — Fix BUG-311) | → `"high"` |
| `None` | — | Sonnet form | No-op | → `"high"` |
| `Some` | ≥ 10% | Opus form | **→ Sonnet** (sufficient capacity — Fix BUG-311) | → `"high"` |
| `Some` | ≥ 10% | Sonnet form | No-op | → `"high"` |
| `Some` | < 10% | Sonnet form | **→ Opus** (near-exhausted — preserve remaining tokens) | → `"max"` |
| `Some` | < 10% | Opus form | No-op | → `"max"` |

"Opus form" = model string matches `claude-opus-*` or `"opus"`.
"Sonnet form" = model string matches `claude-sonnet-*` or `"sonnet"`.

### Threshold

10.0 from `OPUS_OVERRIDE_THRESHOLD` constant in `format.rs:385` (canonical) — actual gate: `100.0 - seven_day_sonnet.utilization < OPUS_OVERRIDE_THRESHOLD` (i.e., < 10% remaining).

### Bug History

- **BUG-300 (Fix TSK-302):** `map_or(0.0, ...)` on `seven_day_sonnet = None` returned 0.0 < threshold → Opus override fired unconditionally for accounts without Sonnet tier. Fix: `if let Some(ref sonnet)` guard.
- **BUG-311 (Fix 2026-06-23):** one-way ratchet — only wrote "opus" (exhaustion), never restored "sonnet" (recovery). Fix: added `else`-branch calling `override_session_model_to_sonnet()`. Tier-absent path also writes "sonnet" conservatively.
- **BUG-322 (Fix 2026-06-28):** effort decoupled from model — BUG-312 init wrote `"low"` when absent but never matched effort to model. Opus override produced `opus/low`. Fix: when model overrides to Opus (`overrode = true`), `set_session_effort(paths, "high")`; when model reverts to Sonnet or absent-tier fallback (`overrode = true`), `set_session_effort(paths, "low")`. BUG-312 init retained as fallback for no-model-change edge case.
- **TSK-335 (Fix H2 + H3, 2026-06-29):** Three related regressions fixed together:
  - **H2 — effort stale in stable sessions:** effort write was inside `if overrode` gate — only fired when model changed. Accounts already at the correct model never got their effort synced. Fix: move all effort writes outside `if overrode` — write unconditionally on every call. BUG-312 fallback becomes effectively unreachable but retained for safety (value updated: `"low"` → `"high"`).
  - **Effort values updated:** Opus effort `"high"` → `"max"`; Sonnet effort `"low"` → `"high"`. BUG-322 fix had the right structure but wrong values.
  - **H3 — render.rs Next line used carry-forward session_effort instead of model-derived effort:** `rec_display` was `session_effort` (the current account's effort read from settings.json), not derived from the recommended account's model. Fix: compute `rec_effort = if rec_model == "opus" { "max" } else { "high" }` inside `render.rs` — always show model-derived effort in the Next line.
  - **Carry-forward removal:** `api.rs` rotation dispatcher removed `if let Some(se) = session_effort { set_session_effort(paths, se) }` — carry-forward was overwriting model-derived effort from `apply_model_override()` with stale pre-rotation effort.

### Relationship to `recommended_model()`

`recommended_model(aq)` in `format.rs` returns the recommended model string without writing to disk, using `OPUS_OVERRIDE_THRESHOLD` directly. `apply_model_override()` also uses `OPUS_OVERRIDE_THRESHOLD` directly (not via `recommended_model()`) — the two functions share the same constant but are independent entry points. The footer `Next` line uses `recommended_model()` directly.

### API Change Impact (2026-06-25)

The Anthropic API restructured `GET /api/oauth/usage` between 2026-06-24T22:06Z and 2026-06-25T01:24Z. The `seven_day_sonnet` field is now always `null`. As a result:

- `apply_model_override()`: always takes the `None` path (row 1 in the table) — writes `"sonnet"` conservatively. The `→ Opus` path (row 5) can no longer fire, leaving sessions in Sonnet even when Sonnet quota is exhausted.
- `recommended_model()`: always returns `"sonnet"` (100% remaining assumed when `None`). The footer `Next` line never recommends Opus.

This is a **temporary blind spot** until Feature 066 (dual-source parsing) populates `seven_day_sonnet` from the new `limits` array when Anthropic re-enables per-model entries. See [algorithm/009](009_oauth_usage_response_migration.md).

### Cross-References

| File | Relationship |
|------|-------------|
| [feature/009_token_usage.md](../feature/009_token_usage.md) | AC-32: bidirectional override semantics |
| [feature/034_explicit_session_model_override.md](../feature/034_explicit_session_model_override.md) | `set_session_model()` / `get_session_model()` |
| [feature/062_unified_session_config.md](../feature/062_unified_session_config.md) | `recommended_model()` canonical entry point |
| [feature/039_decision_algorithms.md](../feature/039_decision_algorithms.md) | Table 2 (legacy reference) |
| [algorithm/009](009_oauth_usage_response_migration.md) | API response format change — why `seven_day_sonnet` is currently always `None`; dual-source parsing recovery path |
| [schema/006](../schema/006_settings_json.md) | `model` and `effortLevel` fields in `settings.json` |
| [pitfall/006](../pitfall/006_model_override_pitfalls.md) | Known pitfalls — absent-tier confusion, one-way ratchet, effort gate, carry-forward overwrite |
