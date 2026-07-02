# Pitfall Tests: Model Override Pitfalls

Test cases verifying that each guard documented in `docs/pitfall/006_model_override_pitfalls.md`
is in place and prevents the described session model management failure mode.

**Source:** [docs/pitfall/006_model_override_pitfalls.md](../../../docs/pitfall/006_model_override_pitfalls.md)
**Case prefix:** `PP-` (Pitfall Protection)

### Pitfall Guard Index

| ID | Pitfall | Bug | Guard Verified By |
|----|---------|-----|-------------------|
| PP-1 | `map_or(0.0, ...)` conflates absent Sonnet tier with exhaustion | BUG-300 | `mre_bug300_model_override_absent_sonnet_no_override` |
| PP-2 | One-way ratchet: only writing Opus, never restoring Sonnet | BUG-311 | `mre_bug311_model_restored_to_sonnet_when_opus_and_quota_sufficient` |
| PP-3 | `effortLevel` never initialized — footer omits effort | BUG-312 | `mre_bug312_effort_initialized_to_high_when_absent` |
| PP-4 | `set_model::` overrides `.account.use` restore — not suppressed | — | `ft05_explicit_set_model_wins_over_switch_restore` in `set_model_test.rs` |

---

### PP-1: `if let Some(ref sonnet)` guard prevents absent-tier Opus override

- **Given:** `OauthUsageData { seven_day_sonnet: None }` — account has no Sonnet tier.
- **When:** `apply_model_override()` evaluates Sonnet remaining capacity.
- **Then:** The `None` arm restores/keeps Sonnet — it does NOT compute `0.0 < threshold`
  and override to Opus. Fix BUG-300 / BUG-311.
- **Rule:** `seven_day_sonnet = None` means "no Sonnet tier present" — NOT "Sonnet is
  exhausted at 0%". These are operationally opposite states. Always guard with
  `if let Some(ref sonnet)` before any threshold comparison.
- **Source fn:** `mre_bug300_model_override_absent_sonnet_no_override` in
  `tests/usage/api_tests_a.rs`
- **Source:** [pitfall/006_model_override_pitfalls.md §P1](../../../docs/pitfall/006_model_override_pitfalls.md)

---

### PP-2: Bidirectional model override — Sonnet restore path exists

- **Given:** An account with `seven_day_sonnet.utilization = 80%` (20% remaining, above 10%
  threshold) and `settings.json` model = `"claude-opus-4-8"` (previously overridden to Opus).
- **When:** `apply_model_override()` runs.
- **Then:** `settings.json` model is restored to `"claude-sonnet-5"` — the recovery path
  (Opus → Sonnet when quota recovers) executes. Fix BUG-311: the pre-fix implementation
  only had an Opus-write branch; once an account was overridden to Opus, it stayed on Opus
  indefinitely even after the 7d window reset.
- **Rule:** All bidirectional state machines need BOTH transition directions. A write-only-
  in-one-direction gate drifts into a permanent degraded state.
- **Source fn:** `mre_bug311_model_restored_to_sonnet_when_opus_and_quota_sufficient` in
  `tests/usage/api_tests_a.rs`; `ac3_sufficient_quota_with_opus_session_restores_sonnet` in
  `tests/usage/api_tests_b.rs`
- **Source:** [pitfall/006_model_override_pitfalls.md §P2](../../../docs/pitfall/006_model_override_pitfalls.md)

---

### PP-3: `effortLevel` initialized unconditionally by every model override branch

- **Given:** `settings.json` has no `effortLevel` field (fresh install, or field deleted).
- **When:** `apply_model_override()` runs and writes a model change.
- **Then:** `effortLevel` is written to `"high"` (Sonnet) or `"max"` (Opus) by the branch
  that executes — the field is ALWAYS initialized, never left absent. Fix BUG-312: the
  original design only propagated `effortLevel` via carry-forward when it was already present;
  a fresh `settings.json` never received an initial value, so the footer never showed an
  effort level.
- **Source fn:** `mre_bug312_effort_initialized_to_high_when_absent` in
  `tests/usage/api_tests_a.rs`
- **Source:** [pitfall/006_model_override_pitfalls.md §P3](../../../docs/pitfall/006_model_override_pitfalls.md)

---

### PP-4: `set_model::` overrides `.account.use` post-switch model restore

- **Given:** An account switch (`clp .account.use name::X`) would normally restore the model
  from `{X}.json`. The user also specifies `set_model::sonnet` on the command.
- **When:** `.account.use name::X set_model::sonnet` executes.
- **Then:** The explicit `set_model::` value (`sonnet`) takes effect — it is NOT overridden
  by the stored model restoration logic. The `set_model::` parameter has higher priority than
  the account-level model carry-forward.
- **Source fn:** `ft05_explicit_set_model_wins_over_switch_restore`,
  `ec6_account_use_set_model_wins_over_switch_restore` in
  `tests/cli/set_model_test.rs`
- **Source:** [pitfall/006_model_override_pitfalls.md §P4](../../../docs/pitfall/006_model_override_pitfalls.md)
