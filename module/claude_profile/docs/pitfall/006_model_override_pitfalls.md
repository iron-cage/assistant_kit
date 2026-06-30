# Pitfall: Model Override Pitfalls

### Pattern

Session model management in `settings.json` has three independent failure modes: absent-tier vs. exhausted confusion, one-way ratchet (write only in one direction), and missing initialization.

### Pitfall 1 — `map_or(0.0, ...)` conflates absent tier with exhaustion (BUG-300)

Using `seven_day_sonnet.map_or(0.0, |s| 100.0 - s.utilization)` to compute Sonnet remaining capacity returns `0.0` when `seven_day_sonnet = None`. `0.0 < threshold` triggers Opus override — even for accounts with no Sonnet tier at all.

**Fix:** `if let Some(ref sonnet)` guard before any comparison. When `seven_day_sonnet = None`, restore to Sonnet conservatively (absent ≠ exhausted — BUG-311 fix).

**Rule:** `seven_day_sonnet = None` means "no Sonnet tier present" — NOT "Sonnet is exhausted". These are operational opposites.

### Pitfall 2 — One-way ratchet: only writing Opus, never restoring Sonnet (BUG-311)

`apply_model_override()` had code to downgrade to Opus (when Sonnet < 15%) but no code to restore Sonnet when quota recovered. Once an account crossed the 15% threshold (causing Opus override), it stayed on Opus indefinitely — even after the 7d window reset and quota was full. (Historical threshold was 15%; changed to 10% — see `OPUS_OVERRIDE_THRESHOLD`.)

**Fix:** Added bidirectional logic: when `seven_day_sonnet` is absent OR ≥ 10%, call `override_session_model_to_sonnet()` if the current model is an Opus form.

**Rule:** All bidirectional state machines need BOTH transition directions. A write-only-in-one-direction gate will drift into a permanent degraded state.

### Pitfall 3 — `effortLevel` never initialized, footer omits effort (BUG-312)

On a fresh install (or after clearing `settings.json`), `effortLevel` is absent, `get_session_effort()` returns `None`, and the footer never shows an effort level — even after repeated rotations. The original design relied on `set_session_effort()` carry-forward during rotation, but carry-forward was only triggered when `session_effort` was already set, so the field was never bootstrapped.

**Fix:** `apply_model_override()` writes effort unconditionally in every branch (TSK-335). A safety `get_session_effort().is_none()` fallback writes `"high"` when the field is absent — but this guard is now unreachable since all model branches write first. The carry-forward `set_session_effort()` call in the rotation dispatcher has been removed entirely.

**Rule:** Fields that always have a correct model-derived value should be written at the model decision point, not carried forward from stale pre-rotation state.

### Pitfall 4 — Effort decoupled from model override (BUG-322)

`apply_model_override()` switched the model to Opus when Sonnet < 15% (historical threshold; changed to 10%) but never set effort to match. BUG-312 init wrote `"low"` when absent — the Opus branch had no effort logic. Result: `opus/low` (or `opus/high` after BUG-322 fix but before TSK-335) in the footer. After TSK-335, correct values are `opus/max` and `sonnet/high`.

**Fix:** Effort is written unconditionally in each branch regardless of `overrode`. Opus branch: `set_session_effort(paths, "max")`. Sonnet branch (sufficient quota and absent-tier): `set_session_effort(paths, "high")`. BUG-312 init fallback retained as unreachable guard with value `"high"`.

**Rule:** Paired settings (model + effort) must be written together at the same decision point, unconditionally. Gating the write on `if overrode` leaves stable sessions with stale effort values that never self-correct.

### Pitfall 5 — Effort gate on `if overrode` leaves stable sessions stale (TSK-335 H2)

Gating `set_session_effort()` inside `if overrode { }` means effort is only updated when the model actually changes. Sessions that are already at the correct model state (Opus when Sonnet exhausted; Sonnet when sufficient) never get their effort updated. A session that starts at Sonnet with no `effortLevel` set will never see `"high"` written because `override_session_model_to_sonnet()` returns `false` (already Sonnet) and `overrode = false` gates out the effort write.

**Fix:** Move all effort writes outside the `if overrode` gate — they run unconditionally on every `apply_model_override()` call.

**Rule:** Effort sync must be unconditional. "Model didn't change" does not mean "effort is correct" — the effort field is independent and can be absent or stale even when the model is already right.

### Cross-References

| File | Relationship |
|------|-------------|
| [algorithm/002](../algorithm/002_session_model_override.md) | Session model override algorithm |
| [feature/062_unified_session_config.md](../feature/062_unified_session_config.md) | `recommended_model()`, `set_session_effort()` |
| [schema/006](../schema/006_settings_json.md) | `model` and `effortLevel` fields |
