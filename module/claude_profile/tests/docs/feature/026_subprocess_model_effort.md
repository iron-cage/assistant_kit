# Test: Feature 026 — Subprocess Model and Effort Control

### Scope

- **Purpose**: Test cases for subprocess model and effort control resolution.
- **Source**: `docs/feature/026_subprocess_model_effort.md`
- **Covers**: AC-01 through AC-16

Feature behavioral requirement test cases for `docs/feature/026_subprocess_model_effort.md`. Each FT case maps to one acceptance criterion. Parameter edge cases are in [cli/param/035_imodel.md](../cli/param/35_imodel.md) and [cli/param/036_effort.md](../cli/param/36_effort.md). Command-level tests (IT-N) are in [cli/command/009_usage.md](../cli/command/09_usage.md).

### AC Coverage Index

| FT | Criterion | AC | Notes |
|----|-----------|-----|-------|
| FT-01 | `imodel::auto` selects sonnet when 5h absent and `son_idle=true` | AC-01 | Unit |
| FT-02 | `imodel::auto` selects sonnet with high Sonnet util and `son_idle=true` | AC-01 | Unit |
| FT-03 | `imodel::auto` selects sonnet at util boundary and `son_idle=true` | AC-01 | Unit |
| FT-04 | `imodel::auto` selects haiku when quota data absent | AC-01 | Unit |
| FT-05 | `imodel::sonnet` always injects `--model claude-sonnet-5` | AC-02 | Unit |
| FT-06 | `imodel::opus` always injects `--model claude-opus-4-8` | AC-03 | Unit |
| FT-07 | `imodel::keep` injects no `--model` flag | AC-04 | Unit |
| FT-08 | `effort::auto` + sonnet → `--effort low` | AC-05 | Unit |
| FT-09 | `effort::auto` + opus → `--effort low` | AC-05 | Unit |
| FT-10 | `imodel::keep effort::auto` → no `--effort` injected | AC-05 | Unit |
| FT-11 | `effort::high` always injects `--effort high` | AC-06 | Unit |
| FT-12 | `effort::max` always injects `--effort max` | AC-07 | Unit |
| FT-13 | `imodel::`/`effort::` apply to both touch and refresh paths | AC-08 | Structural |
| FT-14 | `imodel::`/`effort::` do not affect `format::json` structure | AC-09 | Integration |
| FT-15 | Invalid `imodel::` value exits 1 naming valid values | AC-10 | Integration |
| FT-16 | Invalid `effort::` value exits 1 naming valid values | AC-11 | Integration |
| FT-17 | `imodel::` and `effort::` appear in `.usage --help` | AC-12 | Integration |
| FT-18 | `imodel::haiku` always injects `--model claude-haiku-4-5-20251001` | AC-13 | Unit |
| FT-19 | `effort::auto` + haiku → no `--effort` flag | AC-14 | Unit |
| FT-20 | `effort::low` always injects `--effort low` | AC-15 | Unit |
| FT-21 | `effort::normal` always injects `--effort normal` | AC-16 | Unit |
| FT-22 | `imodel::auto` selects sonnet when `son_idle=true` (any 5h/7d state — son_idle gate fires) | AC-01 | Unit |
| FT-23 | `imodel::auto` selects haiku when Sonnet tier absent (`seven_day_sonnet=None`) | AC-01 | Unit |
| FT-24 | `imodel::auto` selects sonnet when 7d timer idle and `son_idle=true` | AC-01 | Unit |
| FT-25 | `imodel::auto` selects sonnet when 7d running via explicit Some path and `son_idle=true` | AC-01 | Unit |
| FT-26 | `imodel::auto` selects sonnet when 5h absent + 7d running and `son_idle=true` | AC-01 | Unit |
| FT-27 | `imodel::auto` selects haiku when 7d idle + Sonnet exhausted (both gates fail) | AC-01 | Unit |
| FT-28 | `imodel::auto` selects haiku when 7d idle + Sonnet tier absent | AC-01 | Unit |
| FT-29 | `imodel::auto` selects haiku when 7d running via Some + Sonnet tier absent | AC-01 | Unit |
| FT-30 | `imodel::auto` selects haiku when 7d running via Some + Sonnet exhausted | AC-01 | Unit |
| FT-31 | `imodel::auto` selects sonnet when Sonnet window active with 40% remaining (MRE BUG-301) | AC-01 | Unit |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | imodel::auto selects sonnet when 5h absent son_idle | AC-01 | Model Auto |
| FT-02 | imodel::auto selects sonnet with high util son_idle | AC-01 | Model Auto |
| FT-03 | imodel::auto selects sonnet at util boundary son_idle | AC-01 | Boundary |
| FT-04 | imodel::auto selects haiku when quota absent | AC-01 | Fallback |
| FT-05 | imodel::sonnet explicit always sonnet | AC-02 | Explicit |
| FT-06 | imodel::opus explicit always opus | AC-03 | Explicit |
| FT-07 | imodel::keep no model flag | AC-04 | Explicit |
| FT-08 | effort::auto sonnet path produces low | AC-05 | Effort Auto |
| FT-09 | effort::auto opus path produces low | AC-05 | Effort Auto |
| FT-10 | imodel::keep effort::auto no effort flag | AC-05 | Interaction |
| FT-11 | effort::high explicit always high | AC-06 | Explicit |
| FT-12 | effort::max explicit always max | AC-07 | Explicit |
| FT-13 | both params apply to touch and refresh paths | AC-08 | Structural |
| FT-14 | imodel::effort:: no effect on json output | AC-09 | JSON No-op |
| FT-15 | invalid imodel:: exits 1 naming five valid values | AC-10 | Rejection |
| FT-16 | invalid effort:: exits 1 naming five valid values | AC-11 | Rejection |
| FT-17 | imodel:: and effort:: in usage help | AC-12 | Help Output |
| FT-18 | imodel::haiku explicit always haiku | AC-13 | Explicit |
| FT-19 | effort::auto haiku path no effort flag | AC-14 | Interaction |
| FT-20 | effort::low explicit always low | AC-15 | Explicit |
| FT-21 | effort::normal explicit always normal | AC-16 | Explicit |
| FT-22 | imodel::auto selects sonnet any son_idle case | AC-01 | Model Auto |
| FT-23 | imodel::auto selects haiku when son tier absent | AC-01 | Model Auto |
| FT-24 | imodel::auto selects sonnet when d7 idle son_idle | AC-01 | Model Auto |
| FT-25 | imodel::auto selects sonnet when d7 running via Some son_idle | AC-01 | Model Auto |
| FT-26 | imodel::auto selects sonnet 5h absent d7 Some running son_idle | AC-01 | Model Auto |
| FT-27 | imodel::auto selects haiku d7 idle son exhausted | AC-01 | Model Auto |
| FT-28 | imodel::auto selects haiku d7 idle son absent | AC-01 | Model Auto |
| FT-29 | imodel::auto selects haiku d7 Some running son absent | AC-01 | Model Auto |
| FT-30 | imodel::auto selects haiku d7 Some running son exhausted | AC-01 | Model Auto |
| FT-31 | imodel::auto selects sonnet son active 40% remaining MRE BUG-301 | AC-01 | Model Auto |

**Total:** 31 FT cases

---

### FT-01: `imodel::auto` selects sonnet when 5h absent and `son_idle=true`

- **Given:** Account quota data where `five_hour=None` (`five_h_running=false`) and `seven_day_sonnet.resets_at=None` (`son_idle=true`). Helper: `mk_aq_with_sonnet_util(85.0)`.
- **When:** `resolve_model(&quota, SubprocessModel::Auto)`
- **Then:** Returns `IsolatedModel::Specific("claude-sonnet-5")`. `son_idle=true` → `son_idle` gate fires regardless of 5h state; Sonnet selected. A single Sonnet touch opens 5h and Son simultaneously. Verifies the old `five_h_running` constraint is gone.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_auto_selects_sonnet_when_5h_absent` (in `tests/usage/subprocess_tests.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-02: `imodel::auto` selects sonnet with high Sonnet util and `son_idle=true`

- **Given:** Account quota data where `five_hour=None` (`five_h_running=false`) and `seven_day_sonnet.util=35.0` with `resets_at=None` (`son_idle=true`, higher utilization). Helper: `mk_aq_with_sonnet_util(35.0)`.
- **When:** `resolve_model(&quota, SubprocessModel::Auto)`
- **Then:** Returns `IsolatedModel::Specific("claude-sonnet-5")`. `son_idle=true` → gate fires regardless of 5h state or utilization value. Utilization percentage is not consulted in model selection.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_auto_selects_sonnet_when_5h_absent_high_util` (in `tests/usage/subprocess_tests.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-03: `imodel::auto` selects sonnet at util boundary and `son_idle=true`

- **Given:** Account quota data where `five_hour=None` (`five_h_running=false`) and `seven_day_sonnet.util=20.0` with `resets_at=None` (`son_idle=true`, former 20% threshold boundary — utilization is irrelevant for model selection). Helper: `mk_aq_with_sonnet_util(20.0)`.
- **When:** `resolve_model(&quota, SubprocessModel::Auto)`
- **Then:** Returns `IsolatedModel::Specific("claude-sonnet-5")`. `son_idle=true` → gate fires; utilization percentage is not consulted. Verifies boundary value doesn't accidentally enable an old util-based path.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_auto_selects_sonnet_when_5h_absent_boundary_util` (in `tests/usage/subprocess_tests.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-04: `imodel::auto` selects haiku when quota data absent

- **Given:** Account quota data where `seven_day_sonnet_left_pct = None` (quota fetch returned no Sonnet data).
- **When:** `resolve_model(&quota_without_sonnet_pct, "auto")`
- **Then:** Returns `IsolatedModel::Specific("claude-haiku-4-5-20251001")`. No quota data is needed — auto always yields Haiku.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_auto_selects_haiku_without_quota_data` (in `tests/usage/subprocess_tests.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-05: `imodel::sonnet` always injects `--model claude-sonnet-5`

- **Given:** Account quota data with no Sonnet tier tracked (`mk_aq_no_sonnet_data()` — `seven_day_sonnet=None`, would produce Haiku under `auto`); `imodel::sonnet`.
- **When:** `resolve_model(&aq, SubprocessModel::Sonnet)`
- **Then:** Returns `IsolatedModel::Specific("claude-sonnet-5")`. Quota state is ignored; explicit value always wins. (`SubprocessModel::Auto` never resolves to Opus — only Sonnet or Haiku.)
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_sonnet_explicit` (in `subprocess_tests.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-02](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-06: `imodel::opus` always injects `--model claude-opus-4-8`

- **Given:** Account quota data with no Sonnet tier tracked (`mk_aq_no_sonnet_data()` — `seven_day_sonnet=None`, would produce Haiku under `auto`); `imodel::opus`.
- **When:** `resolve_model(&aq, SubprocessModel::Opus)`
- **Then:** Returns `IsolatedModel::Specific("claude-opus-4-8")`. Quota state is ignored; explicit value always wins.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_opus_explicit` (in `subprocess_tests.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-03](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-07: `imodel::keep` injects no `--model` flag; `IsolatedModel::KeepCurrent` passed to `run_isolated()`

- **Given:** Any account quota data; `imodel::keep`.
- **When:** `resolve_model(&quota, "keep")`
- **Then:** Returns `IsolatedModel::KeepCurrent`. The `run_isolated()` call receives `KeepCurrent` and does not prepend any `--model` flag.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_keep_no_model_flag` (in `subprocess_tests.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-04](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-08: `effort::auto` + resolved model=sonnet → subprocess receives `--effort low`

- **Given:** Resolved model = `IsolatedModel::Specific("claude-sonnet-5")`; `effort::auto`.
- **When:** `resolve_effort(&IsolatedModel::Specific("claude-sonnet-5"), "auto")`
- **Then:** Returns `Some("low")`. The arg slice prepended before `["--print", "."]` contains `["--effort", "low"]`. `low` prevents extended thinking which would cause isolated subprocess timeouts.
- **Exit:** n/a (unit test)
- **Source fn:** `it_effort_auto_uniform_low` (in `tests/usage/subprocess_tests.rs` — unified test covering both Sonnet and Opus paths)
- **Source:** [feature/026_subprocess_model_effort.md AC-05](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-09: `effort::auto` + resolved model=opus → subprocess receives `--effort low`

- **Given:** Resolved model = `IsolatedModel::Specific("claude-opus-4-8")`; `effort::auto`. Same parameter as FT-08 — same `low` result regardless of model.
- **When:** `resolve_effort(&IsolatedModel::Specific("claude-opus-4-8"), "auto")`
- **Then:** Returns `Some("low")`. The arg slice contains `["--effort", "low"]`. Same as FT-08: `effort::auto` always produces `low` regardless of whether the model is Sonnet or Opus.
- **Exit:** n/a (unit test)
- **Source fn:** `it_effort_auto_uniform_low` (in `tests/usage/subprocess_tests.rs` — unified test covering both Sonnet and Opus paths)
- **Source:** [feature/026_subprocess_model_effort.md AC-05](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-10: `imodel::keep effort::auto` — no `--model`, no `--effort` in subprocess args

- **Given:** Any account (`mk_aq_no_sonnet_data()`); `imodel::keep effort::auto`.
- **When:** `resolve_model(&aq, SubprocessModel::Keep)` resolves to `IsolatedModel::KeepCurrent`; `effort_pre_args(&model, SubprocessEffort::Auto)` is called on that resolved model.
- **Then:** `effort_pre_args` returns an empty `Vec` — no `--effort` flag is prepended. Combined with `Keep` already producing no `--model` flag, the subprocess arg slice contains neither. The `KeepCurrent` branch of `resolve_effort()` (called internally by `effort_pre_args`) skips injection — model is unknown at dispatch time; injecting any effort level would be speculative.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_keep_effort_auto_no_effort_flag` (in `subprocess_tests.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-05](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-11: `effort::high` always injects `--effort high`

- **Given:** Resolved model = `IsolatedModel::Specific("claude-opus-4-8")` (would produce `low` under `auto`); `effort::high`.
- **When:** `resolve_effort(&IsolatedModel::Specific("claude-opus-4-8"), "high")`
- **Then:** Returns `Some("high")`. Explicit value overrides the model-independent `auto` default of `low`.
- **Exit:** n/a (unit test)
- **Source fn:** `it_effort_high_explicit` (in `subprocess_tests.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-06](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-12: `effort::max` always injects `--effort max`

- **Given:** Resolved model = `IsolatedModel::Specific("claude-sonnet-5")` (would produce `low` under `auto`); `effort::max`.
- **When:** `resolve_effort(&IsolatedModel::Specific("claude-sonnet-5"), "max")`
- **Then:** Returns `Some("max")`. Explicit value overrides the model-independent `auto` default of `low`.
- **Exit:** n/a (unit test)
- **Source fn:** `it_effort_max_explicit` (in `subprocess_tests.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-07](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-13: `imodel::`/`effort::` apply to both touch and refresh subprocess call sites

- **Given:** Source code of `src/usage/touch.rs` and `src/usage/refresh.rs`, each read via `include_str!`.
- **When:** Each file's source text is searched for the literal substrings `resolve_model(` and `effort_pre_args(`.
- **Then:** All four checks pass: `touch.rs` contains `resolve_model(`; `touch.rs` contains `effort_pre_args(`; `refresh.rs` contains `resolve_model(`; `refresh.rs` contains `effort_pre_args(`. Both call sites wire `imodel::` (via `resolve_model`) and `effort::` (via `effort_pre_args`, which wraps `resolve_effort`) into their subprocess arg construction.
- **Exit:** n/a (structural test)
- **Source fn:** `it_ft026_13_imodel_effort_both_paths_structural` (in `usage_model_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-08](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-14: `imodel::`/`effort::` do not affect `format::json` output structure

- **Given:** One account with no `accessToken` (errored quota fetch) — chosen deliberately for a deterministic, offline JSON response.
- **When-A:** `clp .usage format::json`
- **When-B:** `clp .usage imodel::opus effort::max format::json`
- **Then-A and Then-B:** Both exit 0. Both JSON outputs are compared after normalizing the wall-clock-derived `expires_in_secs` field to `null` on both sides (tolerating a legitimate 1-second straddle between the two sequential invocations) — the normalized JSON values are identical. `imodel::` and `effort::` affect only subprocess invocation, not output rendering.
- **Exit:** 0 both cases
- **Source fn:** `it_ft026_14_imodel_effort_no_effect_on_json_schema` (in `usage_model_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-09](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-15: Invalid `imodel::` value exits 1 naming all five valid values

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage imodel::bogus`
- **Then:** Exits 1. Stderr contains each of: `auto`, `sonnet`, `opus`, `haiku`, `keep`.
- **Exit:** 1
- **Source fn:** `it123_imodel_bogus_exits_1` (in `usage_model_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-10](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-16: Invalid `effort::` value exits 1 naming all five valid values

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage effort::bogus`
- **Then:** Exits 1. Stderr contains each of: `auto`, `low`, `normal`, `high`, `max`.
- **Exit:** 1
- **Source fn:** `it125_effort_bogus_exits_1` (in `usage_model_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-11](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-17: `imodel::` and `effort::` appear in `.usage --help` output with default `auto`

- **Given:** Standard environment.
- **When:** `clp .usage.help`
- **Then:** Exits 0. Stdout contains `"imodel"` with default value `auto` and `"effort"` with default value `auto`.
- **Exit:** 0
- **Source fn:** `it126_usage_help_shows_imodel_effort_params` (in `usage_model_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-12](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-18: `imodel::haiku` always injects `--model claude-haiku-4-5-20251001`

- **Given:** Account quota data with any `seven_day_sonnet_left_pct` value; `imodel::haiku`.
- **When:** `resolve_model(&quota, "haiku")`
- **Then:** Returns `IsolatedModel::Specific("claude-haiku-4-5-20251001")`. Quota state is ignored; explicit value always wins. `imodel::haiku` and `imodel::auto` both resolve to Haiku — auto is the default, haiku is the explicit form.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_haiku_explicit` (in `subprocess_tests.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-13](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-19: `effort::auto` + resolved model=haiku → no `--effort` flag injected

- **Given:** Resolved model = `IsolatedModel::Specific("claude-haiku-4-5-20251001")`; `effort::auto`.
- **When:** `effort_pre_args(&IsolatedModel::Specific("claude-haiku-4-5-20251001"), SubprocessEffort::Auto)`
- **Then:** Returns an empty `Vec`. No `--effort` flag is prepended to subprocess args. Haiku has no extended thinking support — injecting any effort level under `auto` would be incorrect.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_haiku_effort_auto_no_effort_flag` (in `tests/usage/subprocess_tests.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-14](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-20: `effort::low` always injects `--effort low`

- **Given:** Three resolved models: `IsolatedModel::Specific("claude-sonnet-5")`, `IsolatedModel::Specific("claude-haiku-4-5-20251001")`, and `IsolatedModel::KeepCurrent`; `effort::low`.
- **When:** `resolve_effort(&model, SubprocessEffort::Low)` is called for each of the three models.
- **Then:** All three return `Some("low")` — explicit `low` is model-independent, applying even to `KeepCurrent` and to Haiku (which gets no effort flag under `auto`).
- **Exit:** n/a (unit test)
- **Source fn:** `it_effort_low_explicit` (in `subprocess_tests.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-15](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-21: `effort::normal` always injects `--effort normal`

- **Given:** Three resolved models: `IsolatedModel::Specific("claude-sonnet-5")`, `IsolatedModel::Specific("claude-haiku-4-5-20251001")`, and `IsolatedModel::KeepCurrent`; `effort::normal`.
- **When:** `resolve_effort(&model, SubprocessEffort::Normal)` is called for each of the three models.
- **Then:** All three return `Some("normal")` — explicit value overrides the auto default of `low`/`None`, applying even to `KeepCurrent` and Haiku.
- **Exit:** n/a (unit test)
- **Source fn:** `it_effort_normal_explicit` (in `subprocess_tests.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-16](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-22: `imodel::auto` selects sonnet when `son_idle=true` (5h running, 7d absent)

- **Given:** Account quota data where `five_h_running=true` (`five_hour.resets_at=Some(_)`), `d7_running=true` (7d window absent → `map_or(true, ...)=true`), and `seven_day_sonnet.resets_at=None` (`son_idle=true`). Helper: `mk_aq_with_son_idle()`.
- **When:** `resolve_model(&aq, SubprocessModel::Auto)`
- **Then:** Returns `IsolatedModel::Specific("claude-sonnet-5")`. `son_idle=true` → `son_idle` gate fires. The 7d-Sonnet window activates only on Sonnet-family API calls; Haiku cannot start it. Fix(BUG-289, BUG-290, TSK-292).
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_auto_selects_sonnet_when_son_idle` (in `tests/usage/subprocess_tests.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-23: `imodel::auto` selects haiku when Sonnet tier absent (`seven_day_sonnet=None`)

- **Given:** Account where `seven_day_sonnet=None` (API does not track a Sonnet quota tier for this account). Other timers running: `five_hour=running, seven_day=None (absent → d7_running=true)`.
- **When:** `resolve_model(&aq, SubprocessModel::Auto)`
- **Then:** Returns `IsolatedModel::Specific("claude-haiku-4-5-20251001")`. `son_idle = None.is_some_and(...) = false` (Sonnet tier absent); `son_idle` gate does NOT fire. Haiku selected.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_auto_selects_haiku_when_son_tier_absent` (in `tests/usage/subprocess_tests.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-24: `imodel::auto` selects sonnet when 7d timer idle and `son_idle=true`

- **Given:** Account where `seven_day=Some({resets_at:None})` (7d window tracked but no session started → `d7_running=false`). Other timers: `five_hour=running, seven_day_sonnet=Some({resets_at:None})` (`son_idle=true`).
- **When:** `resolve_model(&aq, SubprocessModel::Auto)`
- **Then:** Returns `IsolatedModel::Specific("claude-sonnet-5")`. `son_idle=true` → gate fires regardless of `d7_running` state. Verifies the old `d7_running` constraint is gone: 7d-idle no longer blocks Sonnet selection. A single Sonnet touch opens 7d and Son simultaneously.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_auto_selects_sonnet_when_d7_idle` (in `tests/usage/subprocess_tests.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-25: `imodel::auto` selects sonnet when 7d running via explicit Some path and `son_idle=true`

- **Given:** Account where `seven_day=Some({resets_at:Some("2026-06-15T10:00:00Z")})` (7d session active via Some-branch). Other timers: `five_hour=running, seven_day_sonnet=Some({resets_at:None})` (`son_idle=true`).
- **When:** `resolve_model(&aq, SubprocessModel::Auto)`
- **Then:** Returns `IsolatedModel::Specific("claude-sonnet-5")`. `son_idle=true` → gate fires. Exercises the `seven_day=Some(running)` Some-branch of `map_or` — verifies that path correctly resolves to Sonnet.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_auto_selects_sonnet_when_d7_running_explicit` (in `tests/usage/subprocess_tests.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-26: `imodel::auto` selects sonnet when 5h absent + 7d running and `son_idle=true` (cold account)

- **Given:** Account where `five_hour=None` (5h absent → `five_h_running=false`) and `seven_day=Some({resets_at:Some(...)})` (7d running via `map_or` Some-branch). `seven_day_sonnet=Some({resets_at:None})` (`son_idle=true`). This is the BUG-290 cold account scenario.
- **When:** `resolve_model(&aq, SubprocessModel::Auto)`
- **Then:** Returns `IsolatedModel::Specific("claude-sonnet-5")`. `son_idle=true` → gate fires regardless of 5h absent state. Verifies the old `five_h_running` short-circuit is gone: a single Sonnet touch opens 5h and Son simultaneously without a two-touch sequence. Fix(BUG-290).
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_auto_selects_sonnet_when_5h_absent_d7_some_running` (in `tests/usage/subprocess_tests.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-27: `imodel::auto` selects haiku when Sonnet exhausted (7d idle, both gates fail)

- **Given:** Account where `seven_day=Some({resets_at:None})` (7d idle) and `seven_day_sonnet=Some({resets_at:Some(...), utilization:90.0})` (Sonnet running, 10% remaining → `son_idle=false`, `son_available=(100-90>20)=false`). `five_hour=running`.
- **When:** `resolve_model(&aq, SubprocessModel::Auto)`
- **Then:** Returns `IsolatedModel::Specific("claude-haiku-4-5-20251001")`. Both gate conditions fail: `son_idle=false` (resets_at is Some) AND `son_available=false` (only 10% remaining < 20% threshold). Haiku selected; `d7_running` state irrelevant. Exercises `son=exhausted` with 7d-idle. Fix(BUG-301, TSK-311).
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_auto_selects_haiku_when_d7_idle_and_son_running` (in `tests/usage/subprocess_tests.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-28: `imodel::auto` selects haiku when Sonnet tier absent (`seven_day_sonnet=None`) and 7d idle

- **Given:** Account where `seven_day=Some({resets_at:None})` (7d idle) and `seven_day_sonnet=None` (Sonnet tier absent → `son_idle=false`). `five_hour=running`.
- **When:** `resolve_model(&aq, SubprocessModel::Auto)`
- **Then:** Returns `IsolatedModel::Specific("claude-haiku-4-5-20251001")`. `son_idle = None.is_some_and(...)=false` (Sonnet tier absent); gate does NOT fire; `d7_running` state is irrelevant. Haiku selected. Distinct from FT-24 (`son_idle=true` present) and FT-23 (`d7=None` absent).
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_auto_selects_haiku_when_d7_idle_and_son_absent` (in `tests/usage/subprocess_tests.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-29: `imodel::auto` selects haiku when 7d running via Some + Sonnet tier absent

- **Given:** Account where `seven_day=Some({resets_at:Some(...)})` (7d session active → `d7_running=true` via `map_or` Some-branch) and `seven_day_sonnet=None` (Sonnet tier absent → `son_idle=false`). `five_hour=running` (`five_h_running=true`).
- **When:** `resolve_model(&aq, SubprocessModel::Auto)`
- **Then:** Returns `IsolatedModel::Specific("claude-haiku-4-5-20251001")`. `son_idle = None.is_some_and(...)=false` (Sonnet tier absent); gate does NOT fire. Haiku selected. Exercises `d7=Some(running)` Some-branch via `map_or` closure. Closes the `d7=Some(running) + son=absent` cell (complementary to FT-25 `son=idle` → Sonnet and FT-30 `son=running` → Haiku).
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_auto_selects_haiku_when_d7_some_running_and_son_absent` (in `tests/usage/subprocess_tests.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-30: `imodel::auto` selects haiku when 7d running via Some + Sonnet exhausted

- **Given:** Account where `seven_day=Some({resets_at:Some(...)})` (7d active → `d7_running=true` via Some-branch) and `seven_day_sonnet=Some({resets_at:Some(...), utilization:90.0})` (Sonnet running, 10% remaining → `son_idle=false`, `son_available=false`). `five_hour=running`.
- **When:** `resolve_model(&aq, SubprocessModel::Auto)`
- **Then:** Returns `IsolatedModel::Specific("claude-haiku-4-5-20251001")`. Both gate conditions fail: `son_idle=false` (resets_at is Some) AND `son_available=(100-90>20)=false` (10% remaining < 20% threshold). Haiku selected. Closes the `d7=Some(running) + son=exhausted` cell; together with FT-25 (son=idle → Sonnet), FT-29 (son=absent → Haiku), and FT-31 (son=active+available → Sonnet), all key `son` states are covered. Fix(BUG-301, TSK-311).
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_auto_selects_haiku_when_d7_some_running_and_son_running` (in `tests/usage/subprocess_tests.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-31: `imodel::auto` selects sonnet when Sonnet window active with 40% remaining (MRE BUG-301)

- **Given:** Account where `seven_day_sonnet=Some({resets_at:Some("2026-06-20T..."), utilization:60.0})` (Sonnet window active, 40% remaining → `son_idle=false`, `son_available=(100-60>20)=true`). Helper: `mk_aq_with_son_idle()` with son mutated to `resets_at=Some` and `utilization=60.0`.
- **When:** `resolve_model(&aq, SubprocessModel::Auto)`
- **Then:** Returns `IsolatedModel::Specific("claude-sonnet-5")`. `son_available=true` → gate fires; remaining Sonnet quota (40%) is used before the window expires. Before BUG-301 fix: `son_idle=false` caused Haiku — wasting 40% quota. Fix(BUG-301, TSK-311).
- **Exit:** n/a (unit test)
- **Source fn:** `mre_bug301_son_active_with_remaining_quota_selects_sonnet` (in `tests/usage/subprocess_tests.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../docs/feature/026_subprocess_model_effort.md)
