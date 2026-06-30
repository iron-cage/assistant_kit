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

The `Current` line shows `{model}/{effort}`, reading both from `settings.json` via `parse_string_field`. The `Next` line previously showed only `{rec_model}`, omitting effort. Feature 062 adds effort to the Next line in `{model}/{effort}` format using model-derived effort: `"max"` when `rec_model = "opus"`, `"high"` when `rec_model = "sonnet"` (TSK-335 — Fix H3: was incorrectly using `session_effort` carry-forward from current account instead of model-derived effort). The effort is always shown — no conditional on `session_effort`. Column widths are recomputed to align across both lines.

**Rotation dispatcher — model and effort write for winner:**

`.usage rotate::1` previously called `switch_account()` then `apply_touch()` but did not call `apply_model_override` for the winner. The pre-rotation model override (`api.rs:690-696`) updated settings.json for the **old** current account's quota, not the winner's. After Feature 062 and subsequent fixes, the rotation dispatcher additionally:

1. Calls `apply_model_override(winner_data, paths)` — bidirectional model AND effort correction: writes `"opus"` + `effortLevel: "max"` when Sonnet left < 15% (Fix BUG-322, values updated TSK-335); writes `"sonnet"` + `effortLevel: "high"` when Sonnet left >= 15% or tier absent (Fix BUG-311, values updated TSK-335). Effort is written unconditionally (TSK-335 — Fix H2) — the BUG-312 init guard is now effectively unreachable but retained as safety fallback (writes `"high"` if absent).

The carry-forward `set_session_effort(paths, session_effort)` call that previously followed step 1 has been removed (TSK-335): it overwrote model-derived effort with stale pre-rotation effort when `session_effort` was `Some`.

**`set_session_effort()` in `claude_profile_core`:**

Counterpart to `set_session_model()` with identical read-modify-write pattern: reads settings.json, inserts or updates the `effortLevel` key, writes back the full JSON. Accepts `effort_id: &str` (no Option; caller guards on `is_some`). Like `set_session_model()`, creates `~/.claude/` if absent.

### Acceptance Criteria

- **AC-01**: `recommended_model(aq: &AccountQuota) -> &'static str` is a `pub(crate)` function in `format.rs`. Returns `"opus"` when `aq.result` is `Ok(data)` and `data.seven_day_sonnet` is `Some(s)` and `100.0 - s.utilization < 15.0`. Returns `"sonnet"` in all other cases (tier absent, result is Err, or Sonnet left >= 15%).
- **AC-02**: `render.rs` footer `Next` line calls `recommended_model(rec)` instead of the previously-inline match. No inline threshold logic remains in `render.rs` for the recommendation model.
- **AC-03**: Footer `Next` line always shows `{rec_model}/{model-derived-effort}` where model-derived effort is `"max"` when `rec_model = "opus"` and `"high"` when `rec_model = "sonnet"`. Effort is shown unconditionally — not conditional on `session_effort`. Column width `col3_w` is always computed to include the slash-delimited effort, aligning `·` delimiters across both footer lines. (TSK-335 — Fix H3)
- **AC-04**: `set_session_effort(paths: &ClaudePaths, effort_id: &str)` exists in `claude_profile_core::account`. Reads `settings.json`, sets `"effortLevel"` key to `effort_id`, writes back. Creates `~/.claude/` if absent (same guard as `set_session_model`).
- **AC-05**: After `rotate::1` switch succeeds and winner `result = Ok(data)`, `apply_model_override(data, paths)` is called for the winner. When winner's Sonnet left < 15%, settings.json `model` becomes `"opus"`.
- **AC-06**: After `rotate::1` switch succeeds, `apply_model_override(winner_data, paths)` sets `effortLevel` to the model-derived value (`"max"` for Opus, `"high"` for Sonnet or absent-tier). No carry-forward `set_session_effort()` call follows — the carry-forward mechanism was removed (TSK-335) because it overwrote the model-derived effort with stale pre-rotation effort.
- **AC-07**: `apply_model_override()` writes effort unconditionally on every call regardless of whether the model changed (`overrode` flag). When model is already at the target state (no-op), effort is still written: `"max"` (Opus) or `"high"` (Sonnet/absent-tier). Effective result: any call to `apply_model_override()` always produces a fully-synced `effortLevel` in settings.json. (TSK-335 — Fix H2)
- **AC-08**: `apply_model_override()` in `api.rs` calls `recommended_model(aq_data)` internally to determine whether the override fires, or continues to use its own equivalent logic. The threshold is authoritative in `recommended_model()`; no duplication remains.
- **AC-09**: When `apply_model_override()` is called and `seven_day_sonnet` left < 15% (Opus branch), `set_session_effort(paths, "max")` is called unconditionally regardless of `overrode`. (Fix BUG-322; value updated TSK-335: `"high"` → `"max"`)
- **AC-10**: When `apply_model_override()` is called and `seven_day_sonnet` left ≥ 15% or tier absent (Sonnet branch), `set_session_effort(paths, "high")` is called unconditionally regardless of `overrode`. (Fix BUG-322; value updated TSK-335: `"low"` → `"high"`)
- **AC-11**: `apply_model_override()` never leaves `effortLevel` unset after a call when a Sonnet or Opus branch fires. The BUG-312 init guard (`get_session_effort().is_none() → set_session_effort("high")`) is retained as a safety fallback but is unreachable when AC-09 or AC-10 fire — those branches always write first. (TSK-335)

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
| `src/usage/render.rs` | Footer `Next` line generation — calls `recommended_model()`; always shows `/{model-derived-effort}` (`"max"` for opus, `"high"` for sonnet) |
| `src/usage/api.rs` | Rotation dispatcher — calls `apply_model_override()` for winner; carry-forward `set_session_effort()` removed (TSK-335) |
| `module/claude_profile_core/src/account.rs` | `set_session_effort(paths, effort_id)` — writes `effortLevel` to `settings.json` |

### Tests

| File | Relationship |
|------|--------------|
| `tests/docs/feature/062_unified_session_config.md` | FT-01..FT-20, EC-01 — FT-01..FT-13 implemented (TSK-315); FT-14..FT-15 for BUG-312; FT-16..FT-18 for BUG-322; FT-19..FT-20 for TSK-335 (H2 always-sync + H3 render) |

### Algorithm Docs

| File | Relationship |
|------|-------------|
| [algorithm/002_session_model_override.md](../algorithm/002_session_model_override.md) | `apply_model_override()` and `recommended_model()` — canonical entry point extracted by this feature |

### Schema

| File | Relationship |
|------|-------------|
| [schema/006_settings_json.md](../schema/006_settings_json.md) | `model` and `effortLevel` fields written by `set_session_model()` / `set_session_effort()` |
