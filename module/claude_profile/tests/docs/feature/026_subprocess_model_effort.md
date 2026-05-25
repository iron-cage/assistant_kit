# Test: Feature 026 — Subprocess Model and Effort Control

Feature behavioral requirement test cases for `docs/feature/026_subprocess_model_effort.md`. Each FT case maps to one acceptance criterion. Parameter edge cases are in [cli/param/035_imodel.md](../cli/param/035_imodel.md) and [cli/param/036_effort.md](../cli/param/036_effort.md). Command-level tests (IT-N) are in [cli/command/009_usage.md](../cli/command/009_usage.md).

### AC Coverage Index

| FT | Criterion | AC | Notes |
|----|-----------|-----|-------|
| FT-01 | `imodel::auto` selects opus when `7d(Son) < 30%` | AC-01 | Unit |
| FT-02 | `imodel::auto` selects sonnet when `7d(Son) ≥ 30%` | AC-01 | Unit |
| FT-03 | `imodel::auto` selects sonnet at exactly 30% boundary | AC-01 | Unit |
| FT-04 | `imodel::auto` fallback to opus when `7d(Son)` unavailable | AC-01 | Unit |
| FT-05 | `imodel::sonnet` always injects `--model claude-sonnet-4-6` | AC-02 | Unit |
| FT-06 | `imodel::opus` always injects `--model claude-opus-4-6` | AC-03 | Unit |
| FT-07 | `imodel::keep` injects no `--model` flag | AC-04 | Unit |
| FT-08 | `effort::auto` + sonnet → `--effort high` | AC-05 | Unit |
| FT-09 | `effort::auto` + opus → `--effort max` | AC-05 | Unit |
| FT-10 | `imodel::keep effort::auto` → no `--effort` injected | AC-05 | Unit |
| FT-11 | `effort::high` always injects `--effort high` | AC-06 | Unit |
| FT-12 | `effort::max` always injects `--effort max` | AC-07 | Unit |
| FT-13 | `imodel::`/`effort::` apply to both touch and refresh paths | AC-08 | Structural |
| FT-14 | `imodel::`/`effort::` do not affect `format::json` structure | AC-09 | Integration |
| FT-15 | Invalid `imodel::` value exits 1 naming valid values | AC-10 | Integration |
| FT-16 | Invalid `effort::` value exits 1 naming valid values | AC-11 | Integration |
| FT-17 | `imodel::` and `effort::` appear in `.usage --help` | AC-12 | Integration |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | imodel::auto selects opus when sonnet low | AC-01 | Model Auto |
| FT-02 | imodel::auto selects sonnet when sonnet above threshold | AC-01 | Model Auto |
| FT-03 | imodel::auto selects sonnet at exactly 30% boundary | AC-01 | Boundary |
| FT-04 | imodel::auto fallback opus when quota unavailable | AC-01 | Fallback |
| FT-05 | imodel::sonnet explicit always sonnet | AC-02 | Explicit |
| FT-06 | imodel::opus explicit always opus | AC-03 | Explicit |
| FT-07 | imodel::keep no model flag | AC-04 | Explicit |
| FT-08 | effort::auto sonnet path produces high | AC-05 | Effort Auto |
| FT-09 | effort::auto opus path produces max | AC-05 | Effort Auto |
| FT-10 | imodel::keep effort::auto no effort flag | AC-05 | Interaction |
| FT-11 | effort::high explicit always high | AC-06 | Explicit |
| FT-12 | effort::max explicit always max | AC-07 | Explicit |
| FT-13 | both params apply to touch and refresh paths | AC-08 | Structural |
| FT-14 | imodel::effort:: no effect on json output | AC-09 | JSON No-op |
| FT-15 | invalid imodel:: exits 1 naming valid values | AC-10 | Rejection |
| FT-16 | invalid effort:: exits 1 naming valid values | AC-11 | Rejection |
| FT-17 | imodel:: and effort:: in usage help | AC-12 | Help Output |

**Total:** 17 FT cases

---

### FT-01: `imodel::auto` selects opus when `7d(Son) < 30%`

- **Given:** Account quota data where `seven_day_sonnet_left_pct = Some(25.0)` (Sonnet utilization 75%, 25% remaining — below 30% threshold).
- **When:** `resolve_model(&quota, "auto")`
- **Then:** Returns `IsolatedModel::Specific("claude-opus-4-6")`.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_auto_selects_opus_when_sonnet_low` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-02: `imodel::auto` selects sonnet when `7d(Son) ≥ 30%`

- **Given:** Account quota data where `seven_day_sonnet_left_pct = Some(35.0)` (above threshold).
- **When:** `resolve_model(&quota, "auto")`
- **Then:** Returns `IsolatedModel::Specific("claude-sonnet-4-6")`.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_auto_selects_sonnet_above_threshold` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-03: `imodel::auto` selects sonnet at exactly 30% boundary

- **Given:** Account quota data where `seven_day_sonnet_left_pct = Some(30.0)` (exactly at threshold — boundary case).
- **When:** `resolve_model(&quota, "auto")`
- **Then:** Returns `IsolatedModel::Specific("claude-sonnet-4-6")`. The condition is `>= 30.0` (inclusive); 30.0 selects Sonnet.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_auto_selects_sonnet_at_boundary` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-04: `imodel::auto` fallback to opus when `7d(Son)` unavailable

- **Given:** Account quota data where `seven_day_sonnet_left_pct = None` (quota fetch returned no Sonnet data).
- **When:** `resolve_model(&quota_without_sonnet_pct, "auto")`
- **Then:** Returns `IsolatedModel::Specific("claude-opus-4-6")`. The `else` branch treats `None` identically to `<30.0` — Opus is the conservative safe choice.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_auto_fallback_when_quota_unavailable` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-05: `imodel::sonnet` always injects `--model claude-sonnet-4-6`

- **Given:** Account quota data with `seven_day_sonnet_left_pct = Some(5.0)` (well below threshold — would produce opus under `auto`); `imodel::sonnet`.
- **When:** `resolve_model(&quota_low_sonnet, "sonnet")`
- **Then:** Returns `IsolatedModel::Specific("claude-sonnet-4-6")`. Quota state is ignored; explicit value always wins.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_sonnet_explicit` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-02](../../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-06: `imodel::opus` always injects `--model claude-opus-4-6`

- **Given:** Account quota data with `seven_day_sonnet_left_pct = Some(90.0)` (well above threshold — would produce sonnet under `auto`); `imodel::opus`.
- **When:** `resolve_model(&quota_high_sonnet, "opus")`
- **Then:** Returns `IsolatedModel::Specific("claude-opus-4-6")`. Quota state is ignored; explicit value always wins.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_opus_explicit` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-03](../../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-07: `imodel::keep` injects no `--model` flag; `IsolatedModel::KeepCurrent` passed to `run_isolated()`

- **Given:** Any account quota data; `imodel::keep`.
- **When:** `resolve_model(&quota, "keep")`
- **Then:** Returns `IsolatedModel::KeepCurrent`. The `run_isolated()` call receives `KeepCurrent` and does not prepend any `--model` flag.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_keep_no_model_flag` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-04](../../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-08: `effort::auto` + resolved model=sonnet → subprocess receives `--effort high`

- **Given:** Resolved model = `IsolatedModel::Specific("claude-sonnet-4-6")`; `effort::auto`.
- **When:** `resolve_effort(&IsolatedModel::Specific("claude-sonnet-4-6"), "auto")`
- **Then:** Returns `Some("high")`. The arg slice prepended before `["--print", "."]` contains `["--effort", "high"]`.
- **Exit:** n/a (unit test)
- **Source fn:** `it_effort_auto_sonnet_path` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-05](../../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-09: `effort::auto` + resolved model=opus → subprocess receives `--effort max`

- **Given:** Resolved model = `IsolatedModel::Specific("claude-opus-4-6")`; `effort::auto`. Same parameter as FT-08 but different resolved model.
- **When:** `resolve_effort(&IsolatedModel::Specific("claude-opus-4-6"), "auto")`
- **Then:** Returns `Some("max")`. The arg slice contains `["--effort", "max"]`. Divergence from FT-08: Opus produces `max` while Sonnet produces `high`.
- **Exit:** n/a (unit test)
- **Source fn:** `it_effort_auto_opus_path` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-05](../../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-10: `imodel::keep effort::auto` — no `--model`, no `--effort` in subprocess args

- **Given:** Any account; `imodel::keep effort::auto`.
- **When:** `resolve_effort(&IsolatedModel::KeepCurrent, "auto")`
- **Then:** Returns `None`. Combined subprocess arg slice contains neither `--model` nor `--effort`. The `KeepCurrent` branch of `resolve_effort()` skips injection — model is unknown at dispatch time; injecting any effort level would be speculative.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_keep_effort_auto_no_effort_flag` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-05](../../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-11: `effort::high` always injects `--effort high`

- **Given:** Resolved model = `IsolatedModel::Specific("claude-opus-4-6")` (would produce `max` under `auto`); `effort::high`.
- **When:** `resolve_effort(&IsolatedModel::Specific("claude-opus-4-6"), "high")`
- **Then:** Returns `Some("high")`. Explicit value overrides model-derived maximum.
- **Exit:** n/a (unit test)
- **Source fn:** `it_effort_high_explicit` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-06](../../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-12: `effort::max` always injects `--effort max`

- **Given:** Resolved model = `IsolatedModel::Specific("claude-sonnet-4-6")` (would produce `high` under `auto`); `effort::max`.
- **When:** `resolve_effort(&IsolatedModel::Specific("claude-sonnet-4-6"), "max")`
- **Then:** Returns `Some("max")`. Explicit value overrides model-derived maximum.
- **Exit:** n/a (unit test)
- **Source fn:** `it_effort_max_explicit` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-07](../../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-13: `imodel::`/`effort::` apply to both touch and refresh subprocess call sites

- **Given:** Source code structural assertion (static grep): both the touch call site and the refresh call site in `usage.rs` both call `resolve_model()` / `resolve_effort()` and pass the results into their subprocess arg construction.
- **When:** `grep -n "resolve_model\|resolve_effort" src/usage.rs`
- **Then:** Both call sites are present; at least 2 hits for each function.
- **Exit:** n/a (structural test)
- **Source fn:** `it_imodel_and_effort_both_paths_structural` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-08](../../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-14: `imodel::`/`effort::` do not affect `format::json` output structure

- **Given:** One account with valid quota data.
- **When-A:** `clp .usage format::json`
- **When-B:** `clp .usage imodel::opus effort::max format::json`
- **Then-A and Then-B:** Both produce JSON arrays with identical schema. `imodel::` and `effort::` affect only subprocess invocation, not output rendering.
- **Exit:** 0 both cases
- **Source fn:** `it_imodel_effort_json_format_unaffected` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-09](../../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-15: Invalid `imodel::` value exits 1 naming all four valid values

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage imodel::bad`
- **Then:** Exits 1. Stderr contains each of: `auto`, `sonnet`, `opus`, `keep`.
- **Exit:** 1
- **Source fn:** `it113_imodel_bogus_exits_1` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-10](../../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-16: Invalid `effort::` value exits 1 naming all three valid values

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage effort::bad`
- **Then:** Exits 1. Stderr contains each of: `auto`, `high`, `max`.
- **Exit:** 1
- **Source fn:** `it115_effort_bogus_exits_1` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-11](../../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-17: `imodel::` and `effort::` appear in `.usage --help` output with default `auto`

- **Given:** Standard environment.
- **When:** `clp .usage.help`
- **Then:** Exits 0. Stdout contains `"imodel"` with default value `auto` and `"effort"` with default value `auto`.
- **Exit:** 0
- **Source fn:** `it116_usage_help_shows_imodel_effort_params` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-12](../../../../docs/feature/026_subprocess_model_effort.md)
