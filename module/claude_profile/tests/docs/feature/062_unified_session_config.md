# Feature 062 Tests — Unified Session Config Recommendation

### Scope

- **Purpose**: Test cases for model/effort recommendation logic and its application during rendering and rotation.
- **Source**: `docs/feature/062_unified_session_config.md`
- **Covers**: AC-01 through AC-11

Feature doc: [docs/feature/062_unified_session_config.md](../../docs/feature/062_unified_session_config.md)

**Behavioral Divergence Pair:** FT-02 ↔ FT-03 — `seven_day_sonnet.utilization = 90.0` (exactly 10% left) returns `"sonnet"` (boundary is NOT opus); `utilization = 91.0` (9% left) returns `"opus"` — demonstrating the strict `< 10.0` threshold.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FT-01 | `recommended_model()` returns sonnet when Sonnet tier absent | Model Selection |
| FT-02 | `recommended_model()` returns sonnet when Sonnet left >= 10% | Model Selection |
| FT-03 | `recommended_model()` returns opus when Sonnet left < 10% | Model Selection |
| FT-04 | `recommended_model()` returns sonnet on Err result | Model Selection |
| FT-05 | Footer Next line shows `sonnet/high` with model-derived effort | Footer |
| FT-07 | Footer Next line shows `opus/max` when Sonnet exhausted | Footer |
| FT-08 | Column alignment: `·` delimiters align across Current and Next lines | Footer |
| FT-09 | `set_session_effort()` writes effortLevel to settings.json | Effort Write |
| FT-10 | `set_session_effort()` creates `~/.claude/` when absent | Effort Write |
| FT-11 | Rotation applies model override for winner (Sonnet exhausted) | Rotation |
| FT-13 | Rotation writes model-derived effort when none set | Rotation |
| FT-14 | `apply_model_override()` always writes model-derived `effortLevel` (BUG-312 MRE) | Override |
| FT-15 | `apply_model_override()` overwrites `effortLevel` with model-derived value | Override |
| FT-16 | Opus branch sets effort to `"max"` (BUG-322 MRE) | Override |
| FT-17 | Sonnet branch sets effort to `"high"` (BUG-322 reverse) | Override |
| FT-18 | Absent-tier + Opus→Sonnet sets effort to `"high"` | Override |
| FT-19 | Effort always synced even when model already at target | Override |
| FT-20 | Footer Next line always shows model-derived effort regardless of session_effort | Footer |
| EC-01 | `recommended_model()` boundary: utilization = 89.999 returns sonnet | Edge Case |

**Total:** 18 FT cases + 1 EC case (ID gaps: FT-06, FT-12 removed — see gap annotations below)

---

### FT-01: `recommended_model()` returns sonnet when Sonnet tier absent

- **Given:** `AccountQuota` with `result = Ok(data)`, `data.seven_day_sonnet = None`.
- **When:** `recommended_model()` is called.
- **Then:** Returns `"sonnet"`.
- **Source fn:** `format.rs::recommended_model` | AC-01 ✅

---

### FT-02: `recommended_model()` returns sonnet when Sonnet left >= 10%

- **Given:** `AccountQuota` with `seven_day_sonnet.utilization = 90.0` (exactly 10% remaining).
- **When:** `recommended_model()` is called.
- **Then:** Returns `"sonnet"` — strict `< 10.0` guard means 10% exactly is NOT opus.
- **Source fn:** `format.rs::recommended_model` | AC-01 ✅

---

### FT-03: `recommended_model()` returns opus when Sonnet left < 10%

- **Given:** `AccountQuota` with `seven_day_sonnet.utilization = 91.0` (9% remaining).
- **When:** `recommended_model()` is called.
- **Then:** Returns `"opus"`.
- **Source fn:** `format.rs::recommended_model` | AC-01 ✅

---

### FT-04: `recommended_model()` returns sonnet on Err result

- **Given:** `AccountQuota` with `result = Err("HTTP 429".into())`.
- **When:** `recommended_model()` is called.
- **Then:** Returns `"sonnet"`.
- **Source fn:** `format.rs::recommended_model` | AC-01 ✅

---

### FT-05: Footer Next line shows `sonnet/high` with model-derived effort

- **Given:** Recommended account with `seven_day_sonnet.utilization <= 90.0` (Sonnet left >= 10%).
- **When:** Usage table rendered with footer.
- **Then:** Footer Next line contains `sonnet/high` in the third column — `"high"` is model-derived (Sonnet branch), not from `session_effort`.
- **Source fn:** `render.rs` footer generation | AC-03 ✅

---

> **Note:** FT-06 removed — case not applicable; see feature doc history.

---

### FT-07: Footer Next line shows `opus/max` when Sonnet exhausted

- **Given:** Recommended account with `seven_day_sonnet.utilization = 91.0` (9% remaining, Opus branch).
- **When:** Usage table rendered with footer.
- **Then:** Footer Next line contains `opus/max` in the third column — `"max"` is model-derived (Opus branch), not from `session_effort`.
- **Source fn:** `render.rs` footer generation | AC-03 ✅

---

### FT-08: Column alignment — `·` delimiters align across Current and Next lines

- **Given:** Current account with `session_model = "sonnet"`, `session_effort = "high"`; recommended account with Sonnet left >= 10%.
- **When:** Usage table rendered with footer.
- **Then:** Third `·` in both Current and Next lines falls at the same column position; Next line shows `sonnet/high` (model-derived) — both lines always have `model/effort` format.
- **Source fn:** `render.rs` footer generation | AC-03 ✅

---

### FT-09: `set_session_effort()` writes effortLevel to settings.json

- **Given:** Temp dir used as `~/.claude/`; existing `settings.json` with other keys present.
- **When:** `set_session_effort(paths, "max")` is called.
- **Then:** `settings.json` contains `"effortLevel": "max"`; existing keys preserved.
- **Source fn:** `claude_profile_core::account::set_session_effort` | AC-04 ✅

---

### FT-10: `set_session_effort()` creates `~/.claude/` when absent

- **Given:** Non-existent `~/.claude/` directory; no `settings.json`.
- **When:** `set_session_effort(paths, "max")` is called.
- **Then:** `~/.claude/` directory created; `settings.json` written with `"effortLevel"` key.
- **Source fn:** `claude_profile_core::account::set_session_effort` | AC-04 ✅

---

### FT-11: Rotation applies model override for winner (Sonnet exhausted)

- **Given:** Winner account has `seven_day_sonnet.utilization = 91.0` (9% remaining, Opus branch).
- **When:** `.usage rotate::1` executed.
- **Then:** After switch, `settings.json` `model` field = `"opus"`.
- **Source fn:** `api.rs` rotation dispatcher | AC-05 ✅

---

> **Note:** FT-12 removed — case not applicable; see feature doc history.

---

### FT-13: Rotation writes model-derived effort when none set

- **Given:** `settings.json` has no `effortLevel` key before rotation; winner has Sonnet left >= 10%.
- **When:** `.usage rotate::1` executed.
- **Then:** After switch, `settings.json` contains `"effortLevel": "high"` — written by `apply_model_override()` Sonnet branch (unconditional write); no carry-forward call.
- **Source fn:** `api.rs` rotation dispatcher + `apply_model_override` | AC-07 ✅

---

### FT-14: `apply_model_override()` always writes model-derived `effortLevel` (BUG-312 MRE, updated TSK-335)

- **Given:** Temp dir as `~/.claude/`; no `settings.json` present; quota data with Sonnet left >= 10%.
- **When:** `apply_model_override()` is called.
- **Then:** `settings.json` created; contains `"effortLevel": "high"`. Written by the Sonnet branch unconditional write (TSK-335); the BUG-312 init guard is unreachable but retained.
- **Source fn:** `src/usage/api.rs` (`apply_model_override`) | Fix BUG-312, AC-07 ✅

---

### FT-15: `apply_model_override()` overwrites `effortLevel` with model-derived value (TSK-335)

- **Given:** `settings.json` pre-seeded with `"effortLevel": "high"`; quota data with Sonnet left >= 10%.
- **When:** `apply_model_override()` is called.
- **Then:** `settings.json` contains `"effortLevel": "high"` — written by unconditional Sonnet branch (TSK-335). Value matches pre-seeded value; mechanism has changed from preservation to model-derived overwrite.
- **Source fn:** `src/usage/api.rs` (`apply_model_override`) | AC-07, AC-11 ✅

---

### FT-16: Opus branch sets effort to `"max"` (BUG-322 MRE, updated TSK-335)

- **Given:** No `settings.json`; quota data with `seven_day_sonnet.utilization = 91.0` (9% remaining, Opus branch).
- **When:** `apply_model_override()` is called.
- **Then:** `settings.json` contains `"opus"` AND `"max"` — effort paired with Opus branch (TSK-335: was `"high"`).
- **Source fn:** `tests/usage/api_tests_a.rs::mre_bug322_opus_override_sets_effort_max` | AC-09 ✅

---

### FT-17: Sonnet branch sets effort to `"high"` (BUG-322 reverse, updated TSK-335)

- **Given:** `settings.json` pre-seeded with `"opus"` + `"max"`; quota data with `seven_day_sonnet.utilization = 4.0` (96% remaining).
- **When:** `apply_model_override()` is called.
- **Then:** `settings.json` contains `"sonnet"` AND `"high"` — effort set to Sonnet-derived value when model reverts (TSK-335: was `"low"`).
- **Source fn:** `tests/usage/api_tests_a.rs::t11_opus_to_sonnet_sets_effort_high` | AC-10 ✅

---

### FT-18: Absent-tier + Opus→Sonnet sets effort to `"high"` (BUG-322 absent-tier, updated TSK-335)

- **Given:** `settings.json` pre-seeded with `"opus"` + `"max"`; quota data with `seven_day_sonnet = None`.
- **When:** `apply_model_override()` is called.
- **Then:** `settings.json` contains `"sonnet"` AND `"high"` — absent tier forces Sonnet + Sonnet-derived effort (TSK-335: was `"low"`).
- **Source fn:** `tests/usage/api_tests_a.rs::t12_absent_tier_with_opus_sets_effort_high` | AC-10 ✅

---

### FT-19: Effort always synced even when model already at target (always-sync, TSK-335 H2)

- **Given:** `settings.json` pre-seeded with `"model": "sonnet"` (no `effortLevel`); quota data with Sonnet left >= 10% — `override_session_model_to_sonnet()` returns `false` (model unchanged).
- **When:** `apply_model_override()` is called.
- **Then:** `settings.json` contains `"effortLevel": "high"` — effort written unconditionally even though `overrode = false`.
- **Source fn:** `src/usage/api.rs` (`apply_model_override`) | AC-07, AC-11 ✅

---

### FT-20: Footer Next line always shows model-derived effort regardless of session_effort (TSK-335 H3)

- **Given:** `session_effort = None`; recommended account has Sonnet left >= 10%.
- **When:** Usage table rendered with footer.
- **Then:** Footer Next line contains `sonnet/high` — model-derived effort always shown; no conditional on `session_effort` being present.
- **Source fn:** `render.rs` footer generation | AC-03 ✅

---

### EC-01: `recommended_model()` boundary — utilization = 89.999 returns sonnet

- **Given:** `AccountQuota` with `seven_day_sonnet.utilization = 89.999` (10.001% remaining).
- **When:** `recommended_model()` is called.
- **Then:** Returns `"sonnet"` — still above threshold (10.001% > 10.0%).
- **Source fn:** `format.rs::recommended_model` | AC-01 ✅
