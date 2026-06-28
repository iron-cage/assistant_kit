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

## FT-05 — Footer Next line shows `model/effort` when effort present

**AC-03** | **Source**: `render.rs` footer generation

| Field | Value |
|-------|-------|
| Input | `session_effort = Some("high")`, rec account has Sonnet left >= 15% |
| Expected | Footer Next line contains `sonnet/high` in the third column |
| Status | ✅ |

## FT-06 — Footer Next line shows only model when effort absent

**AC-03** | **Source**: `render.rs` footer generation

| Field | Value |
|-------|-------|
| Input | `session_effort = None`, rec account has Sonnet left >= 15% |
| Expected | Footer Next line contains `sonnet` (no slash, no effort) in third column |
| Status | ✅ |

## FT-07 — Footer Next line shows `opus/effort` when Sonnet exhausted and effort present

**AC-03** | **Source**: `render.rs` footer generation

| Field | Value |
|-------|-------|
| Input | `session_effort = Some("max")`, rec account has `seven_day_sonnet.utilization = 90.0` |
| Expected | Footer Next line contains `opus/max` in third column |
| Status | ✅ |

## FT-08 — Column alignment: `·` delimiters align across Current and Next lines

**AC-03** | **Source**: `render.rs` footer generation

| Field | Value |
|-------|-------|
| Input | Current account with `session_model = "sonnet"`, `session_effort = "high"`; rec account with `sonnet` model |
| Expected | Third `·` in both lines falls at the same column position |
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

## FT-12 — Rotation writes carry-forward effort to settings.json

**AC-06** | **Source**: `api.rs` rotation dispatcher

| Field | Value |
|-------|-------|
| Input | `.usage rotate::1`; `session_effort = "high"` in settings.json before rotation |
| Expected | After switch, `settings.json` `effortLevel` = `"high"` |
| Status | ✅ |

## FT-13 — Rotation initializes effort to "low" when none set (Fix BUG-312)

**AC-07** | **Source**: `api.rs` rotation dispatcher + `apply_model_override`

| Field | Value |
|-------|-------|
| Input | `.usage rotate::1`; settings.json has no `effortLevel` key before rotation |
| Expected | After switch, `settings.json` contains `"effortLevel": "low"` — initialized by `apply_model_override()` BUG-312 guard before carry-forward runs |
| Status | ✅ (updated post-BUG-312; was: "no effortLevel key written") |

## FT-14 — `apply_model_override()` initializes `effortLevel: "low"` when absent (BUG-312 MRE)

**Fix BUG-312** | **Source**: `src/usage/api.rs` (`apply_model_override`)

| Field | Value |
|-------|-------|
| Input | Temp dir as `~/.claude/`; no `settings.json` present; `apply_model_override` called with any quota data |
| Expected | `settings.json` created; contains `"effortLevel": "low"`. The init guard (`get_session_effort().is_none()` → `set_session_effort(paths, "low")`) fires. |
| Status | ✅ `mre_bug312_effort_initialized_to_low_when_absent` in `src/usage/api_tests.rs` |

## FT-15 — Pre-configured `effortLevel` is not overwritten by `apply_model_override()` (BUG-312 preservation guard)

**Fix BUG-312** | **Source**: `src/usage/api.rs` (`apply_model_override`)

| Field | Value |
|-------|-------|
| Input | `settings.json` pre-seeded with `"effortLevel": "high"`; `apply_model_override` called |
| Expected | `settings.json` still contains `"effortLevel": "high"` — `is_none()` guard is false; init does not overwrite user-configured effort |
| Status | ✅ `t10_effort_preserved_when_already_configured` in `src/usage/api_tests.rs` |

## FT-16 — Opus override sets effort to `"high"` (BUG-322 MRE)

**AC-09** | **Source**: `src/usage/api_tests.rs::mre_bug322_opus_override_sets_effort_high`

| Field | Value |
|-------|-------|
| Input | No `settings.json`; `seven_day_sonnet.utilization = 90.0` (10% left, < 15% threshold) |
| Expected | `settings.json` contains `"opus"` AND `"high"` — effort paired with Opus model override |
| Status | ✅ |

## FT-17 — Sonnet override resets effort to `"low"` (BUG-322 reverse)

**AC-10** | **Source**: `src/usage/api_tests.rs::t11_opus_to_sonnet_resets_effort_to_low`

| Field | Value |
|-------|-------|
| Input | `settings.json` pre-seeded with `"opus"` + `"high"`; `seven_day_sonnet.utilization = 4.0` (96% left) |
| Expected | `settings.json` contains `"sonnet"` AND `"low"` — effort resets when model reverts |
| Status | ✅ |

## FT-18 — Absent-tier + Opus→Sonnet resets effort to `"low"` (BUG-322 absent-tier)

**AC-10** | **Source**: `src/usage/api_tests.rs::t12_absent_tier_with_opus_resets_effort_to_low`

| Field | Value |
|-------|-------|
| Input | `settings.json` pre-seeded with `"opus"` + `"high"`; `seven_day_sonnet = None` |
| Expected | `settings.json` contains `"sonnet"` AND `"low"` — absent tier forces Sonnet + effort reset |
| Status | ✅ |

## EC-01 — `recommended_model()` boundary: utilization = 84.999 returns sonnet (above threshold)

**AC-01** | **Source**: `format.rs::recommended_model`

| Field | Value |
|-------|-------|
| Input | `seven_day_sonnet.utilization = 84.999` (15.001% left) |
| Expected | Returns `"sonnet"` (still above threshold) |
| Status | ✅ |
