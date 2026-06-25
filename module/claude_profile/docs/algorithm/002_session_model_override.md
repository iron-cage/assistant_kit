# Algorithm: Session Model Override

### Purpose

Bidirectionally manage the interactive session model in `~/.claude/settings.json` based on Sonnet quota utilization. Upgrades from Opus back to Sonnet when quota recovers; downgrades to Opus when near-exhausted.

### Entry Points

- `src/usage/api.rs` — `apply_model_override(quota, paths, trace, label, name)` (mutation; `quota: &OauthUsageData`)
- `src/usage/format.rs` — `recommended_model(aq)` (read-only, for footer recommendation; `aq: &AccountQuota`)

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

15.0 from `OPUS_OVERRIDE_THRESHOLD` constant in `format.rs:385` (canonical) — actual gate: `100.0 - seven_day_sonnet.utilization < OPUS_OVERRIDE_THRESHOLD` (i.e., < 15% remaining).

### Bug History

- **BUG-300 (Fix TSK-302):** `map_or(0.0, ...)` on `seven_day_sonnet = None` returned 0.0 < threshold → Opus override fired unconditionally for accounts without Sonnet tier. Fix: `if let Some(ref sonnet)` guard.
- **BUG-311 (Fix 2026-06-23):** one-way ratchet — only wrote "opus" (exhaustion), never restored "sonnet" (recovery). Fix: added `else`-branch calling `override_session_model_to_sonnet()`. Tier-absent path also writes "sonnet" conservatively.

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
