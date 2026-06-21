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

## FT-13 — Rotation does not inject effort when none set

**AC-07** | **Source**: `api.rs` rotation dispatcher

| Field | Value |
|-------|-------|
| Input | `.usage rotate::1`; settings.json has no `effortLevel` key before rotation |
| Expected | After switch, `settings.json` still has no `effortLevel` key |
| Status | ✅ |

## EC-01 — `recommended_model()` boundary: utilization = 84.999 returns sonnet (above threshold)

**AC-01** | **Source**: `format.rs::recommended_model`

| Field | Value |
|-------|-------|
| Input | `seven_day_sonnet.utilization = 84.999` (15.001% left) |
| Expected | Returns `"sonnet"` (still above threshold) |
| Status | ✅ |
