# Algorithm 008: Subprocess Effort Resolution

AC test cases for `docs/algorithm/008_subprocess_effort_resolution.md`. Tests
`resolve_effort(resolved_model, effort_param)` in `src/usage/subprocess.rs` and the
effort initialization path in `apply_model_override()` in `src/usage/api.rs`.

### AC Case Index

| AC | Short Name | Category | Status |
|----|------------|----------|--------|
| AC-1 | Explicit `effort::low` → `low` injected for any model | Nominal | ✅ |
| AC-2 | Explicit `effort::normal` → `normal` injected | Nominal | ✅ |
| AC-3 | Explicit `effort::high` → `high` injected | Nominal | ✅ |
| AC-4 | Explicit `effort::max` → `max` injected | Nominal | ✅ |
| AC-5 | `effort::auto` + Opus → `low` | Nominal | ✅ |
| AC-6 | `effort::auto` + Sonnet → `low` | Nominal | ✅ |
| AC-7 | `effort::auto` + Haiku → no `--effort` flag | Nominal | ✅ |
| AC-8 | `effort::auto` + `KeepCurrent` → no `--effort` flag | Nominal | ✅ |
| AC-9 | Model override to Opus sets `effortLevel` to `"max"` (BUG-322 fix) | Regression | ✅ |
| AC-10 | Model override to Sonnet (sufficient quota) sets `effortLevel` to `"high"` | Regression | ✅ |
| AC-11 | Absent-tier path sets `effortLevel` to `"high"` | Regression | ✅ |
| AC-12 | `effortLevel` initialized when absent (BUG-312 fix) | Regression | ✅ |
| AC-13 | Effort synced even when model is already at target (no-op model path) | TSK-335 | ✅ |
| AC-14 | `effort::` absent on fetch failure — no spurious effort write | Regression | ✅ |

---

### AC-1: Explicit `effort::low` → `low` injected

- **Given:** `effort_param = "low"`; any `resolved_model`.
- **When:** `resolve_effort(resolved_model, "low")` is called.
- **Then:** Returns `Some("low")` — the explicit effort value is passed through unconditionally.
- **Source fn:** Covered by effort integration tests in `tests/usage/api_tests_a.rs`
- **Source:** [algorithm/008_subprocess_effort_resolution.md](../../../docs/algorithm/008_subprocess_effort_resolution.md)

---

### AC-2–4: Explicit effort values passed through for any model

- **Given:** `effort_param ∈ {"low", "normal", "high", "max"}`; any model.
- **When:** `resolve_effort` is called.
- **Then:** Returns `Some(effort_param)` — explicit values are never rewritten.
- **Source fn:** Covered by effort override tests in `tests/usage/api_tests_a.rs`
- **Source:** [algorithm/008_subprocess_effort_resolution.md](../../../docs/algorithm/008_subprocess_effort_resolution.md)

---

### AC-5: `effort::auto` + Opus → `low`

- **Given:** `effort_param = "auto"`; `resolved_model = Specific("claude-opus-4-6")`.
- **When:** `resolve_effort` is called.
- **Then:** Returns `Some("low")` — Opus supports extended thinking but isolated subprocesses
  use `.` keep-alive prompts; `low` prevents unnecessary extended thinking overhead.
- **Source fn:** `t10_sonnet_branch_writes_effort_high` (Sonnet path); effort low path
  implied by `mre_bug322_opus_override_sets_effort_max` in `tests/usage/api_tests_a.rs`
- **Source:** [algorithm/008_subprocess_effort_resolution.md](../../../docs/algorithm/008_subprocess_effort_resolution.md)

---

### AC-6: `effort::auto` + Sonnet → `low`

- **Given:** `effort_param = "auto"`; `resolved_model = Specific("claude-sonnet-4-6")`.
- **When:** `resolve_effort` is called.
- **Then:** Returns `Some("low")` — Sonnet supports extended thinking; `low` is used for
  keep-alive subprocesses to avoid timeout.
- **Source fn:** `t10_sonnet_branch_writes_effort_high` in `tests/usage/api_tests_a.rs`
  (tests the `settings.json` side); subprocess effort low also in usage_feature_test.rs
- **Source:** [algorithm/008_subprocess_effort_resolution.md](../../../docs/algorithm/008_subprocess_effort_resolution.md)

---

### AC-7: `effort::auto` + Haiku → no `--effort` flag

- **Given:** `effort_param = "auto"`; `resolved_model = Specific("claude-haiku-4-5-20251001")`.
- **When:** `resolve_effort` is called.
- **Then:** Returns `None` — Haiku does not support extended thinking; injecting `--effort`
  would cause an API error.
- **Source fn:** `tests/cli/usage_feature_test.rs` (effort flags absent for Haiku subprocess)
- **Source:** [algorithm/008_subprocess_effort_resolution.md](../../../docs/algorithm/008_subprocess_effort_resolution.md)

---

### AC-8: `effort::auto` + `KeepCurrent` → no `--effort` flag

- **Given:** `effort_param = "auto"`; `resolved_model = KeepCurrent` (unknown session model).
- **When:** `resolve_effort` is called.
- **Then:** Returns `None` — unknown model identity; conservative approach avoids injecting
  effort for a model that may not support it.
- **Source fn:** `tests/cli/usage_feature_test.rs` (keep-model effort absent path)
- **Source:** [algorithm/008_subprocess_effort_resolution.md](../../../docs/algorithm/008_subprocess_effort_resolution.md)

---

### AC-9: Model override to Opus sets `effortLevel` to `"max"` (BUG-322 fix)

- **Given:** `apply_model_override()` determines Sonnet is exhausted and writes
  `"claude-opus-4-6"` to `settings.json`.
- **When:** The Opus override branch executes.
- **Then:** `effortLevel` in `settings.json` is also written to `"max"` — Opus benefits from
  maximum effort for quality. Before Fix(BUG-322), only the model field was written; effort
  remained at the previous value (often `"high"` from Sonnet) causing suboptimal reasoning.
- **Source fn:** `mre_bug322_opus_override_sets_effort_max` in `tests/usage/api_tests_a.rs`
- **Source:** [algorithm/008_subprocess_effort_resolution.md](../../../docs/algorithm/008_subprocess_effort_resolution.md)

---

### AC-10: Model override to Sonnet sets `effortLevel` to `"high"`

- **Given:** `apply_model_override()` determines Sonnet quota is sufficient and writes
  `"claude-sonnet-4-6"` to `settings.json`.
- **When:** The Sonnet restore branch executes.
- **Then:** `effortLevel` in `settings.json` is also written to `"high"` — Sonnet's optimal
  effort for keep-alive subprocesses. Fix BUG-322 / TSK-335.
- **Source fn:** `t10_sonnet_branch_writes_effort_high`,
  `t11_opus_to_sonnet_sets_effort_high` in `tests/usage/api_tests_a.rs`
- **Source:** [algorithm/008_subprocess_effort_resolution.md](../../../docs/algorithm/008_subprocess_effort_resolution.md)

---

### AC-11: Absent-tier path sets `effortLevel` to `"high"`

- **Given:** `apply_model_override()` executes the absent-tier path (`seven_day_sonnet = None`
  with Opus session → restore to Sonnet).
- **When:** `override_session_model_to_sonnet()` runs.
- **Then:** `effortLevel` is written to `"high"` — same as the sufficient-quota Sonnet path.
  Fix BUG-322 / TSK-335.
- **Source fn:** `t12_absent_tier_with_opus_sets_effort_high` in `tests/usage/api_tests_a.rs`
- **Source:** [algorithm/008_subprocess_effort_resolution.md](../../../docs/algorithm/008_subprocess_effort_resolution.md)

---

### AC-12: `effortLevel` initialized when absent (BUG-312 fix)

- **Given:** `settings.json` has no `effortLevel` field (fresh install or after clearing).
- **When:** `apply_model_override()` runs for any model branch.
- **Then:** `effortLevel` is written unconditionally by the override branch that executes —
  the field is always initialized, never left absent. Before Fix(BUG-312), effort was only
  written via carry-forward if already present, leaving fresh installs without any effort
  level in the footer.
- **Source fn:** `mre_bug312_effort_initialized_to_high_when_absent` in
  `tests/usage/api_tests_a.rs`
- **Source:** [algorithm/008_subprocess_effort_resolution.md](../../../docs/algorithm/008_subprocess_effort_resolution.md)

---

### AC-13: Effort synced even when model is already at target (no-op model path)

- **Given:** `settings.json` model is already `"claude-sonnet-4-6"` and Sonnet quota is
  sufficient (no model write needed).
- **When:** `apply_model_override()` runs.
- **Then:** `effortLevel` is STILL written to `"high"` even though no model change occurred —
  effort sync is unconditional per TSK-335 H2. This prevents stale effort values after manual
  model changes.
- **Source fn:** `ft19_effort_synced_when_model_already_at_target` in
  `tests/usage/api_tests_a.rs`
- **Source:** [algorithm/008_subprocess_effort_resolution.md](../../../docs/algorithm/008_subprocess_effort_resolution.md)

---

### AC-14: Effort absent on fetch failure — no spurious write

- **Given:** `fetch_all_quota()` fails for an account (returns `Err`).
- **When:** The pre-switch touch context is constructed.
- **Then:** `model` and `effort` fields are absent from the context — no stale values are
  carried into the touch subprocess from a failed fetch.
- **Source fn:** `test_pre_switch_touch_ctx_model_effort_absent_on_fetch_failure` in
  `tests/usage/api_tests_a.rs`
- **Source:** [algorithm/008_subprocess_effort_resolution.md](../../../docs/algorithm/008_subprocess_effort_resolution.md)
