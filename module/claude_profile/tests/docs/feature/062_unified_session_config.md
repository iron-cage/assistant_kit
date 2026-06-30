# Feature 062 Tests — Unified Session Config Recommendation

Feature doc: [docs/feature/062_unified_session_config.md](../../docs/feature/062_unified_session_config.md)

---

## FT-01 — `recommended_model()` returns sonnet when Sonnet tier absent

**AC-01** | **Source**: `format.rs::recommended_model`

| Field | Value |
|-------|-------|
| Input | `AccountQuota` with `result = Ok(data)`, `data.seven_day_sonnet = None` |
| Expected | Returns `"sonnet"` |
| Status | ✅ |

## FT-02 — `recommended_model()` returns sonnet when Sonnet left >= 15%

**AC-01** | **Source**: `format.rs::recommended_model`

| Field | Value |
|-------|-------|
| Input | `AccountQuota` with `seven_day_sonnet.utilization = 85.0` (exactly 15% left) |
| Expected | Returns `"sonnet"` (strict `< 15.0` guard — boundary is NOT opus) |
| Status | ✅ |

## FT-03 — `recommended_model()` returns opus when Sonnet left < 15%

**AC-01** | **Source**: `format.rs::recommended_model`

| Field | Value |
|-------|-------|
| Input | `AccountQuota` with `seven_day_sonnet.utilization = 86.0` (14% left) |
| Expected | Returns `"opus"` |
| Status | ✅ |

## FT-04 — `recommended_model()` returns sonnet on Err result

**AC-01** | **Source**: `format.rs::recommended_model`

| Field | Value |
|-------|-------|
| Input | `AccountQuota` with `result = Err("HTTP 429".into())` |
| Expected | Returns `"sonnet"` |
| Status | ✅ |

## FT-05 — Footer Next line shows `model/effort` with model-derived effort (sonnet)

**AC-03** | **Source**: `render.rs` footer generation

| Field | Value |
|-------|-------|
| Input | Rec account has Sonnet left >= 15% (`seven_day_sonnet.utilization <= 85.0`) |
| Expected | Footer Next line contains `sonnet/high` in the third column — `"high"` is model-derived (sonnet branch), not from `session_effort` |
| Status | ✅ |

## FT-07 — Footer Next line shows `opus/max` when Sonnet exhausted

**AC-03** | **Source**: `render.rs` footer generation

| Field | Value |
|-------|-------|
| Input | Rec account has `seven_day_sonnet.utilization = 90.0` (10% left, Opus branch) |
| Expected | Footer Next line contains `opus/max` in third column — `"max"` is model-derived (opus branch), not from `session_effort` |
| Status | ✅ |

## FT-08 — Column alignment: `·` delimiters align across Current and Next lines

**AC-03** | **Source**: `render.rs` footer generation

| Field | Value |
|-------|-------|
| Input | Current account with `session_model = "sonnet"`, `session_effort = "high"`; rec account with sonnet model (Sonnet left >= 15%) |
| Expected | Third `·` in both lines falls at the same column position; Next line shows `sonnet/high` (model-derived) — both lines always have `model/effort` format |
| Status | ✅ |

## FT-09 — `set_session_effort()` writes effortLevel to settings.json

**AC-04** | **Source**: `claude_profile_core::account::set_session_effort`

| Field | Value |
|-------|-------|
| Input | Temp dir as `~/.claude/`; call `set_session_effort(paths, "max")` |
| Expected | `settings.json` contains `"effortLevel": "max"`; existing keys preserved |
| Status | ✅ |

## FT-10 — `set_session_effort()` creates `~/.claude/` when absent

**AC-04** | **Source**: `claude_profile_core::account::set_session_effort`

| Field | Value |
|-------|-------|
| Input | Non-existent `~/.claude/` directory |
| Expected | Directory created; `settings.json` written with `"effortLevel"` key |
| Status | ✅ |

## FT-11 — Rotation applies model override for winner (Sonnet exhausted)

**AC-05** | **Source**: `api.rs` rotation dispatcher

| Field | Value |
|-------|-------|
| Input | `.usage rotate::1`; winner has `seven_day_sonnet.utilization = 90.0` |
| Expected | After switch, `settings.json` `model` = `"opus"` |
| Status | ✅ |

## FT-13 — Rotation writes model-derived effort when none set

**AC-07** | **Source**: `api.rs` rotation dispatcher + `apply_model_override`

| Field | Value |
|-------|-------|
| Input | `.usage rotate::1`; settings.json has no `effortLevel` key before rotation; winner has Sonnet left >= 15% |
| Expected | After switch, `settings.json` contains `"effortLevel": "high"` — written by `apply_model_override()` Sonnet branch (unconditional write); no carry-forward call |
| Status | ✅ |

## FT-14 — `apply_model_override()` always writes model-derived `effortLevel` (BUG-312 MRE, updated TSK-335)

**Fix BUG-312, AC-07** | **Source**: `src/usage/api.rs` (`apply_model_override`)

| Field | Value |
|-------|-------|
| Input | Temp dir as `~/.claude/`; no `settings.json` present; `apply_model_override` called with Sonnet left >= 15% quota data |
| Expected | `settings.json` created; contains `"effortLevel": "high"`. Written by the Sonnet branch unconditional write (TSK-335); the BUG-312 init guard is unreachable but retained. |
| Status | ✅ |

## FT-15 — `apply_model_override()` overwrites `effortLevel` with model-derived value (TSK-335)

**AC-07, AC-11** | **Source**: `src/usage/api.rs` (`apply_model_override`)

| Field | Value |
|-------|-------|
| Input | `settings.json` pre-seeded with `"effortLevel": "high"`; `apply_model_override` called with Sonnet left >= 15% |
| Expected | `settings.json` contains `"effortLevel": "high"` — written by unconditional Sonnet branch (TSK-335). Value happens to match pre-seeded value; mechanism has changed from preservation to model-derived overwrite. |
| Status | ✅ |

## FT-16 — Opus branch sets effort to `"max"` (BUG-322 MRE, updated TSK-335)

**AC-09** | **Source**: `src/usage/api_tests.rs::mre_bug322_opus_override_sets_effort_max`

| Field | Value |
|-------|-------|
| Input | No `settings.json`; `seven_day_sonnet.utilization = 90.0` (10% left, < 15% threshold) |
| Expected | `settings.json` contains `"opus"` AND `"max"` — effort paired with Opus branch (TSK-335: was `"high"`) |
| Status | ✅ |

## FT-17 — Sonnet branch sets effort to `"high"` (BUG-322 reverse, updated TSK-335)

**AC-10** | **Source**: `src/usage/api_tests.rs::t11_opus_to_sonnet_sets_effort_high`

| Field | Value |
|-------|-------|
| Input | `settings.json` pre-seeded with `"opus"` + `"max"`; `seven_day_sonnet.utilization = 4.0` (96% left) |
| Expected | `settings.json` contains `"sonnet"` AND `"high"` — effort set to Sonnet-derived value when model reverts (TSK-335: was `"low"`) |
| Status | ✅ |

## FT-18 — Absent-tier + Opus→Sonnet sets effort to `"high"` (BUG-322 absent-tier, updated TSK-335)

**AC-10** | **Source**: `src/usage/api_tests.rs::t12_absent_tier_with_opus_sets_effort_high`

| Field | Value |
|-------|-------|
| Input | `settings.json` pre-seeded with `"opus"` + `"max"`; `seven_day_sonnet = None` |
| Expected | `settings.json` contains `"sonnet"` AND `"high"` — absent tier forces Sonnet + Sonnet-derived effort (TSK-335: was `"low"`) |
| Status | ✅ |

## FT-19 — Effort always synced even when model is already at target (always-sync, TSK-335 H2)

**AC-07, AC-11** | **Source**: `src/usage/api.rs` (`apply_model_override`)

| Field | Value |
|-------|-------|
| Input | `settings.json` pre-seeded with `"model": "sonnet"` (no effortLevel); `apply_model_override` called with Sonnet left >= 15% — `override_session_model_to_sonnet()` returns `false` (already Sonnet, no model change) |
| Expected | `settings.json` contains `"effortLevel": "high"` — effort written unconditionally even though `overrode = false` |
| Status | ✅ |

## FT-20 — Footer Next line always shows model-derived effort regardless of session_effort (TSK-335 H3)

**AC-03** | **Source**: `render.rs` footer generation

| Field | Value |
|-------|-------|
| Input | `session_effort = None`; rec account has Sonnet left >= 15% |
| Expected | Footer Next line contains `sonnet/high` — model-derived effort always shown; no conditional on `session_effort` being present |
| Status | ✅ |

## EC-01 — `recommended_model()` boundary: utilization = 84.999 returns sonnet (above threshold)

**AC-01** | **Source**: `format.rs::recommended_model`

| Field | Value |
|-------|-------|
| Input | `seven_day_sonnet.utilization = 84.999` (15.001% left) |
| Expected | Returns `"sonnet"` (still above threshold) |
| Status | ✅ |
