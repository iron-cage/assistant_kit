# Pitfall: Model Override Pitfalls

### Pattern

Session model management in `settings.json` has three independent failure modes: absent-tier vs. exhausted confusion, one-way ratchet (write only in one direction), and missing initialization.

### Pitfall 1 — `map_or(0.0, ...)` conflates absent tier with exhaustion (BUG-300)

Using `seven_day_sonnet.map_or(0.0, |s| 100.0 - s.utilization)` to compute Sonnet remaining capacity returns `0.0` when `seven_day_sonnet = None`. `0.0 < threshold` triggers Opus override — even for accounts with no Sonnet tier at all.

**Fix:** `if let Some(ref sonnet)` guard before any comparison. When `seven_day_sonnet = None`, restore to Sonnet conservatively (absent ≠ exhausted — BUG-311 fix).

**Rule:** `seven_day_sonnet = None` means "no Sonnet tier present" — NOT "Sonnet is exhausted". These are operational opposites.

### Pitfall 2 — One-way ratchet: only writing Opus, never restoring Sonnet (BUG-311)

`apply_model_override()` had code to downgrade to Opus (when Sonnet < 15%) but no code to restore Sonnet when quota recovered. Once an account crossed the 15% threshold (causing Opus override), it stayed on Opus indefinitely — even after the 7d window reset and quota was full.

**Fix:** Added bidirectional logic: when `seven_day_sonnet` is absent OR ≥ 15%, call `override_session_model_to_sonnet()` if the current model is an Opus form.

**Rule:** All bidirectional state machines need BOTH transition directions. A write-only-in-one-direction gate will drift into a permanent degraded state.

### Pitfall 3 — `effortLevel` never initialized, footer omits effort (BUG-312)

`set_session_effort()` carries forward the pre-rotation `effortLevel` when `session_effort = Some(...)`. But on a fresh install (or after clearing `settings.json`), `effortLevel` is absent, `get_session_effort()` returns `None`, and the footer never shows an effort level — even after repeated rotations.

**Fix:** `apply_model_override()` guards `get_session_effort().is_none()` and writes `"low"` when `effortLevel` is absent. This is now a **fallback** for the edge case where no model change occurs. When a model change fires, effort is set bidirectionally by Pitfall 4 (Fix BUG-322).

**Rule:** For fields that have sensible defaults, write the default on first use rather than waiting for an explicit set operation that may never happen.

### Pitfall 4 — Effort decoupled from model override (BUG-322)

`apply_model_override()` switched the model to Opus when Sonnet < 15% but never set effort to match. BUG-312 init wrote `"low"` when absent — the Opus branch had no effort logic. Result: `opus/low` in the footer instead of `opus/high`. Switching to Opus without upgrading effort wastes the model's extended thinking capability.

**Fix:** Effort now tracks model bidirectionally. When `override_session_model_to_opus` fires (`overrode = true`), `set_session_effort(paths, "high")` is called. When `override_session_model_to_sonnet` fires (`overrode = true`, from either sufficient-quota or absent-tier path), `set_session_effort(paths, "low")` is called. BUG-312 init is retained as fallback for the no-model-change edge case.

**Rule:** Paired settings (model + effort) must be written together at the same decision point. A write that changes one half without updating the other creates an inconsistent state that defaults and fallbacks cannot detect.

### Cross-References

| File | Relationship |
|------|-------------|
| [algorithm/002](../algorithm/002_session_model_override.md) | Session model override algorithm |
| [feature/062_unified_session_config.md](../feature/062_unified_session_config.md) | `recommended_model()`, `set_session_effort()` |
| [schema/006](../schema/006_settings_json.md) | `model` and `effortLevel` fields |
