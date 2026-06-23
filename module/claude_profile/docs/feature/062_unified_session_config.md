# Feature 062 — Unified Session Config Recommendation

### Scope

- **Purpose**: Ensure session model and effort are computed from a single canonical source and applied consistently for both display (footer) and execution (rotation).
- **Responsibility**: Documents `recommended_model()` in `format.rs`; footer `Next` line showing `{model}/{effort}`; `set_session_effort()` in `claude_profile_core`; post-rotation model override and effort write for the winner.
- **In Scope**: `recommended_model(aq)` canonical function; footer Next line effort display; `set_session_effort()` API; rotation dispatcher applying model override and effort for winner.
- **Out of Scope**: Touch subprocess effort (`imodel::` / `effort::` params — Feature 026); changes to `effort::` parameter behavior; `live::` monitor loop footer.

### Design

Previously, the recommended session model was computed in two places using the same 15% Sonnet threshold:

1. `render.rs:249-257` — inline `match` for the footer `Next` line label
2. `api.rs:259-290` (`apply_model_override`) — settings.json write when the override fires

These were not linked: a threshold change or logic refinement required updating both. Feature 062 extracts the shared logic into `recommended_model(aq: &AccountQuota) -> &'static str` in `format.rs`, which both sites call.

**Footer Next line — adding effort display:**

The `Current` line already shows `{model}/{effort}`, reading both from `settings.json` via `parse_string_field`. The `Next` line previously showed only `{rec_model}`, omitting effort. Feature 062 adds the `session_effort` to the Next line in the same `{model}/{effort}` format when effort is present in settings.json. When `session_effort` is `None` (key absent from settings.json), the Next line shows only `{rec_model}` (no slash). Column widths are recomputed to align across both lines.

**Rotation dispatcher — model and effort write for winner:**

`.usage rotate::1` previously called `switch_account()` then `apply_touch()` but did not call `apply_model_override` for the winner. The pre-rotation model override (`api.rs:690-696`) updated settings.json for the **old** current account's quota, not the winner's. After Feature 062 and subsequent fixes, the rotation dispatcher additionally:

1. Calls `apply_model_override(winner_data, paths)` — bidirectional model correction (Fix BUG-311): writes `"opus"` when Sonnet left < 15%; writes `"sonnet"` when Sonnet left >= 15% or tier absent. Also initializes `effortLevel: "low"` in settings.json when the key is absent (Fix BUG-312 — first-use initialization guard).
2. Calls `set_session_effort(paths, session_effort)` — writes `effortLevel` to settings.json with carry-forward value when `session_effort` is `Some`. When `session_effort` is `None` and effortLevel was absent, step 1 already initialized it to `"low"` via the BUG-312 guard; this step is a no-op.

**`set_session_effort()` in `claude_profile_core`:**

Counterpart to `set_session_model()` with identical read-modify-write pattern: reads settings.json, inserts or updates the `effortLevel` key, writes back the full JSON. Accepts `effort_id: &str` (no Option; caller guards on `is_some`). Like `set_session_model()`, creates `~/.claude/` if absent.

### Acceptance Criteria

- **AC-01**: `recommended_model(aq: &AccountQuota) -> &'static str` is a `pub(crate)` function in `format.rs`. Returns `"opus"` when `aq.result` is `Ok(data)` and `data.seven_day_sonnet` is `Some(s)` and `100.0 - s.utilization < 15.0`. Returns `"sonnet"` in all other cases (tier absent, result is Err, or Sonnet left >= 15%).
- **AC-02**: `render.rs` footer `Next` line calls `recommended_model(rec)` instead of the previously-inline match. No inline threshold logic remains in `render.rs` for the recommendation model.
- **AC-03**: Footer `Next` line shows `{rec_model}/{session_effort}` when `session_effort` is `Some`; shows `{rec_model}` (no slash) when `session_effort` is `None`. Column width `col3_w` accounts for the slash-delimited effort when present, aligning `·` delimiters across both footer lines.
- **AC-04**: `set_session_effort(paths: &ClaudePaths, effort_id: &str)` exists in `claude_profile_core::account`. Reads `settings.json`, sets `"effortLevel"` key to `effort_id`, writes back. Creates `~/.claude/` if absent (same guard as `set_session_model`).
- **AC-05**: After `rotate::1` switch succeeds and winner `result = Ok(data)`, `apply_model_override(data, paths)` is called for the winner. When winner's Sonnet left < 15%, settings.json `model` becomes `"opus"`.
- **AC-06**: After `rotate::1` switch succeeds and `session_effort` is `Some(e)`, `set_session_effort(paths, e)` is called. Settings.json `effortLevel` key reflects the carry-forward effort value.
- **AC-07**: When `session_effort` is `None` (not set in settings.json at rotation time), the carry-forward `set_session_effort()` call is a no-op. However, `apply_model_override()` (called before carry-forward) initializes `effortLevel: "low"` when the key is absent in settings.json (Fix BUG-312). Effective result: rotation with no prior effortLevel produces `effortLevel: "low"` in settings.json.
- **AC-08**: `apply_model_override()` in `api.rs` calls `recommended_model(aq_data)` internally to determine whether the override fires, or continues to use its own equivalent logic. The threshold is authoritative in `recommended_model()`; no duplication remains.

### Features

| File | Relationship |
|------|--------------|
| [020_usage_sort_strategies.md](020_usage_sort_strategies.md) | Footer format — Next line now shows effort |
| [034_explicit_session_model_override.md](034_explicit_session_model_override.md) | `set_session_model()` counterpart for effort |
| [035_model_command.md](035_model_command.md) | `get_session_model()` / `set_session_model()` — sibling API |
| [038_usage_strategy_rotate.md](038_usage_strategy_rotate.md) | Rotation dispatcher — model+effort write added |
| [039_decision_algorithms.md](039_decision_algorithms.md) | Table 2 (Session Model Override) — `recommended_model()` canonical entry point |
| [026_subprocess_model_effort.md](026_subprocess_model_effort.md) | Touch subprocess effort — distinct from session effort |

### Sources

| File | Relationship |
|------|--------------|
| `src/usage/format.rs` | `recommended_model(aq)` — canonical session model recommendation function |
| `src/usage/render.rs` | Footer `Next` line generation — calls `recommended_model()`; adds `/{effort}` suffix when present |
| `src/usage/api.rs` | Rotation dispatcher — calls `apply_model_override()` + `set_session_effort()` for winner |
| `module/claude_profile_core/src/account.rs` | `set_session_effort(paths, effort_id)` — writes `effortLevel` to `settings.json` |

### Tests

| File | Relationship |
|------|--------------|
| `tests/docs/feature/62_unified_session_config.md` | FT-01..FT-15, EC-01 — FT-01..FT-13 implemented (TSK-315); FT-14..FT-15 added for BUG-312 MRE tests |

### Algorithm Docs

| File | Relationship |
|------|-------------|
| [algorithm/002_session_model_override.md](../algorithm/002_session_model_override.md) | `apply_model_override()` and `recommended_model()` — canonical entry point extracted by this feature |

### Schema

| File | Relationship |
|------|-------------|
| [schema/006_settings_json.md](../schema/006_settings_json.md) | `model` and `effortLevel` fields written by `set_session_model()` / `set_session_effort()` |
