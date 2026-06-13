# Test: Feature 026 — Subprocess Model and Effort Control

Feature behavioral requirement test cases for `docs/feature/026_subprocess_model_effort.md`. Each FT case maps to one acceptance criterion. Parameter edge cases are in [cli/param/035_imodel.md](../cli/param/35_imodel.md) and [cli/param/036_effort.md](../cli/param/36_effort.md). Command-level tests (IT-N) are in [cli/command/009_usage.md](../cli/command/09_usage.md).

### AC Coverage Index

| FT | Criterion | AC | Notes |
|----|-----------|-----|-------|
| FT-01 | `imodel::auto` selects haiku when 5h absent (Sonnet idle but sole-son-trigger does not fire) | AC-01 | Unit |
| FT-02 | `imodel::auto` selects haiku with high Sonnet util (5h absent — not sole-son-trigger) | AC-01 | Unit |
| FT-03 | `imodel::auto` selects haiku at util boundary (5h absent — not sole-son-trigger) | AC-01 | Unit |
| FT-04 | `imodel::auto` selects haiku when quota data absent | AC-01 | Unit |
| FT-05 | `imodel::sonnet` always injects `--model claude-sonnet-4-6` | AC-02 | Unit |
| FT-06 | `imodel::opus` always injects `--model claude-opus-4-6` | AC-03 | Unit |
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
| FT-22 | `imodel::auto` selects sonnet when `son_running=false` is sole inactive timer | AC-01 | Unit |
| FT-23 | `imodel::auto` selects haiku when Sonnet tier absent (`seven_day_sonnet=None`) | AC-01 | Unit |
| FT-24 | `imodel::auto` selects haiku when 7d timer idle (`seven_day=Some({resets_at:None})`) | AC-01 | Unit |
| FT-25 | `imodel::auto` selects sonnet when 7d running via explicit Some path | AC-01 | Unit |
| FT-26 | `imodel::auto` selects haiku when 5h absent + 7d running via Some path | AC-01 | Unit |
| FT-27 | `imodel::auto` selects haiku when 7d idle + Sonnet running (two gate failures) | AC-01 | Unit |
| FT-28 | `imodel::auto` selects haiku when 7d idle + Sonnet tier absent | AC-01 | Unit |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | imodel::auto selects haiku when 5h absent | AC-01 | Model Auto |
| FT-02 | imodel::auto selects haiku with high util 5h absent | AC-01 | Model Auto |
| FT-03 | imodel::auto selects haiku at util boundary 5h absent | AC-01 | Boundary |
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
| FT-22 | imodel::auto selects sonnet sole-son-trigger | AC-01 | Model Auto |
| FT-23 | imodel::auto selects haiku when son tier absent | AC-01 | Model Auto |
| FT-24 | imodel::auto selects haiku when d7 idle | AC-01 | Model Auto |
| FT-25 | imodel::auto selects sonnet when d7 running via Some | AC-01 | Model Auto |
| FT-26 | imodel::auto selects haiku 5h absent d7 Some running | AC-01 | Model Auto |
| FT-27 | imodel::auto selects haiku d7 idle son running | AC-01 | Model Auto |
| FT-28 | imodel::auto selects haiku d7 idle son absent | AC-01 | Model Auto |

**Total:** 28 FT cases

---

### FT-01: `imodel::auto` selects haiku when 5h absent (Sonnet idle, sole-son-trigger does not fire)

- **Given:** Account quota data where `five_hour=None` (`five_h_running=false`) and `seven_day_sonnet.resets_at=None` (Sonnet window idle). The five-hour window absence prevents the sole-son-trigger condition (`five_h_running AND d7_running AND son_idle`) from firing.
- **When:** `resolve_model(&quota, SubprocessModel::Auto)`
- **Then:** Returns `IsolatedModel::Specific("claude-haiku-4-5-20251001")`. Five-hour window absent → `five_h_running=false` → sole-son-trigger gate is false; Haiku selected. Keep-alive pings don't need expensive models; Haiku conserves Sonnet and Opus quota.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_auto_selects_haiku` (in `src/usage/subprocess.rs #[cfg(test)]`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-02: `imodel::auto` selects haiku with high Sonnet util (5h absent — not sole-son-trigger)

- **Given:** Account quota data where `five_hour=None` (`five_h_running=false`) and `seven_day_sonnet.util=35.0` with `resets_at=None` (Sonnet window idle, higher utilization). Five-hour absence prevents sole-son-trigger regardless of utilization value.
- **When:** `resolve_model(&quota, SubprocessModel::Auto)`
- **Then:** Returns `IsolatedModel::Specific("claude-haiku-4-5-20251001")`. Sole-son-trigger requires `five_h_running=true`; with 5h absent, auto yields Haiku independent of utilization level.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_auto_selects_haiku` (in `src/usage/subprocess.rs #[cfg(test)]`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-03: `imodel::auto` selects haiku at util boundary (5h absent — not sole-son-trigger)

- **Given:** Account quota data where `five_hour=None` (`five_h_running=false`) and `seven_day_sonnet.util=20.0` with `resets_at=None` (util at former 20% threshold boundary — utilization is now irrelevant for model selection). Five-hour absence prevents sole-son-trigger.
- **When:** `resolve_model(&quota, SubprocessModel::Auto)`
- **Then:** Returns `IsolatedModel::Specific("claude-haiku-4-5-20251001")`. Sole-son-trigger requires `five_h_running=true`; absent 5h, auto yields Haiku. Utilization percentage is not consulted in model selection.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_auto_selects_haiku` (in `src/usage/subprocess.rs #[cfg(test)]`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-04: `imodel::auto` selects haiku when quota data absent

- **Given:** Account quota data where `seven_day_sonnet_left_pct = None` (quota fetch returned no Sonnet data).
- **When:** `resolve_model(&quota_without_sonnet_pct, "auto")`
- **Then:** Returns `IsolatedModel::Specific("claude-haiku-4-5-20251001")`. No quota data is needed — auto always yields Haiku.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_auto_selects_haiku_without_quota_data` (in `src/usage/subprocess.rs #[cfg(test)]`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-05: `imodel::sonnet` always injects `--model claude-sonnet-4-6`

- **Given:** Account quota data with `seven_day_sonnet_left_pct = Some(5.0)` (well below threshold — would produce opus under `auto`); `imodel::sonnet`.
- **When:** `resolve_model(&quota_low_sonnet, "sonnet")`
- **Then:** Returns `IsolatedModel::Specific("claude-sonnet-4-6")`. Quota state is ignored; explicit value always wins.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_sonnet_explicit` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-02](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-06: `imodel::opus` always injects `--model claude-opus-4-6`

- **Given:** Account quota data with `seven_day_sonnet_left_pct = Some(90.0)` (well above threshold — would produce sonnet under `auto`); `imodel::opus`.
- **When:** `resolve_model(&quota_high_sonnet, "opus")`
- **Then:** Returns `IsolatedModel::Specific("claude-opus-4-6")`. Quota state is ignored; explicit value always wins.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_opus_explicit` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-03](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-07: `imodel::keep` injects no `--model` flag; `IsolatedModel::KeepCurrent` passed to `run_isolated()`

- **Given:** Any account quota data; `imodel::keep`.
- **When:** `resolve_model(&quota, "keep")`
- **Then:** Returns `IsolatedModel::KeepCurrent`. The `run_isolated()` call receives `KeepCurrent` and does not prepend any `--model` flag.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_keep_no_model_flag` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-04](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-08: `effort::auto` + resolved model=sonnet → subprocess receives `--effort low`

- **Given:** Resolved model = `IsolatedModel::Specific("claude-sonnet-4-6")`; `effort::auto`.
- **When:** `resolve_effort(&IsolatedModel::Specific("claude-sonnet-4-6"), "auto")`
- **Then:** Returns `Some("low")`. The arg slice prepended before `["--print", "."]` contains `["--effort", "low"]`. `low` prevents extended thinking which would cause isolated subprocess timeouts.
- **Exit:** n/a (unit test)
- **Source fn:** `it_effort_auto_sonnet_path` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-05](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-09: `effort::auto` + resolved model=opus → subprocess receives `--effort low`

- **Given:** Resolved model = `IsolatedModel::Specific("claude-opus-4-6")`; `effort::auto`. Same parameter as FT-08 — same `low` result regardless of model.
- **When:** `resolve_effort(&IsolatedModel::Specific("claude-opus-4-6"), "auto")`
- **Then:** Returns `Some("low")`. The arg slice contains `["--effort", "low"]`. Same as FT-08: `effort::auto` always produces `low` regardless of whether the model is Sonnet or Opus.
- **Exit:** n/a (unit test)
- **Source fn:** `it_effort_auto_opus_path` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-05](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-10: `imodel::keep effort::auto` — no `--model`, no `--effort` in subprocess args

- **Given:** Any account; `imodel::keep effort::auto`.
- **When:** `resolve_effort(&IsolatedModel::KeepCurrent, "auto")`
- **Then:** Returns `None`. Combined subprocess arg slice contains neither `--model` nor `--effort`. The `KeepCurrent` branch of `resolve_effort()` skips injection — model is unknown at dispatch time; injecting any effort level would be speculative.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_keep_effort_auto_no_effort_flag` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-05](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-11: `effort::high` always injects `--effort high`

- **Given:** Resolved model = `IsolatedModel::Specific("claude-opus-4-6")` (would produce `low` under `auto`); `effort::high`.
- **When:** `resolve_effort(&IsolatedModel::Specific("claude-opus-4-6"), "high")`
- **Then:** Returns `Some("high")`. Explicit value overrides the model-independent `auto` default of `low`.
- **Exit:** n/a (unit test)
- **Source fn:** `it_effort_high_explicit` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-06](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-12: `effort::max` always injects `--effort max`

- **Given:** Resolved model = `IsolatedModel::Specific("claude-sonnet-4-6")` (would produce `low` under `auto`); `effort::max`.
- **When:** `resolve_effort(&IsolatedModel::Specific("claude-sonnet-4-6"), "max")`
- **Then:** Returns `Some("max")`. Explicit value overrides the model-independent `auto` default of `low`.
- **Exit:** n/a (unit test)
- **Source fn:** `it_effort_max_explicit` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-07](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-13: `imodel::`/`effort::` apply to both touch and refresh subprocess call sites

- **Given:** Source code structural assertion (static grep): both the touch call site and the refresh call site in `usage/subprocess.rs` both call `resolve_model()` / `resolve_effort()` and pass the results into their subprocess arg construction.
- **When:** `grep -n "resolve_model\|resolve_effort" src/usage/subprocess.rs`
- **Then:** Both call sites are present; at least 2 hits for each function.
- **Exit:** n/a (structural test)
- **Source fn:** `it_imodel_and_effort_both_paths_structural` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-08](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-14: `imodel::`/`effort::` do not affect `format::json` output structure

- **Given:** One account with valid quota data.
- **When-A:** `clp .usage format::json`
- **When-B:** `clp .usage imodel::opus effort::max format::json`
- **Then-A and Then-B:** Both produce JSON arrays with identical schema. `imodel::` and `effort::` affect only subprocess invocation, not output rendering.
- **Exit:** 0 both cases
- **Source fn:** `it_imodel_effort_json_format_unaffected` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-09](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-15: Invalid `imodel::` value exits 1 naming all five valid values

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage imodel::bad`
- **Then:** Exits 1. Stderr contains each of: `auto`, `sonnet`, `opus`, `haiku`, `keep`.
- **Exit:** 1
- **Source fn:** `it123_imodel_bogus_exits_1` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-10](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-16: Invalid `effort::` value exits 1 naming all five valid values

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage effort::bad`
- **Then:** Exits 1. Stderr contains each of: `auto`, `low`, `normal`, `high`, `max`.
- **Exit:** 1
- **Source fn:** `it125_effort_bogus_exits_1` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-11](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-17: `imodel::` and `effort::` appear in `.usage --help` output with default `auto`

- **Given:** Standard environment.
- **When:** `clp .usage.help`
- **Then:** Exits 0. Stdout contains `"imodel"` with default value `auto` and `"effort"` with default value `auto`.
- **Exit:** 0
- **Source fn:** `it126_usage_help_shows_imodel_effort_params` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-12](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-18: `imodel::haiku` always injects `--model claude-haiku-4-5-20251001`

- **Given:** Account quota data with any `seven_day_sonnet_left_pct` value; `imodel::haiku`.
- **When:** `resolve_model(&quota, "haiku")`
- **Then:** Returns `IsolatedModel::Specific("claude-haiku-4-5-20251001")`. Quota state is ignored; explicit value always wins. `imodel::haiku` and `imodel::auto` both resolve to Haiku — auto is the default, haiku is the explicit form.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_haiku_explicit` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-13](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-19: `effort::auto` + resolved model=haiku → no `--effort` flag injected

- **Given:** Resolved model = `IsolatedModel::Specific("claude-haiku-4-5-20251001")`; `effort::auto`.
- **When:** `resolve_effort(&IsolatedModel::Specific("claude-haiku-4-5-20251001"), "auto")`
- **Then:** Returns `None`. No `--effort` flag is prepended to subprocess args. Haiku has no extended thinking support — injecting any effort level under `auto` would be incorrect.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_haiku_effort_auto_no_effort_flag` (in `src/usage/subprocess.rs #[cfg(test)]`)
- **Source:** [feature/026_subprocess_model_effort.md AC-14](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-20: `effort::low` always injects `--effort low`

- **Given:** Resolved model = `IsolatedModel::Specific("claude-opus-4-6")` (would produce `low` under `auto`); `effort::low`.
- **When:** `resolve_effort(&IsolatedModel::Specific("claude-opus-4-6"), "low")`
- **Then:** Returns `Some("low")`. Explicit `low` matches the auto default; no override needed.
- **Exit:** n/a (unit test)
- **Source fn:** `it_effort_low_explicit` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-15](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-21: `effort::normal` always injects `--effort normal`

- **Given:** Resolved model = `IsolatedModel::Specific("claude-opus-4-6")` (would produce `low` under `auto`); `effort::normal`.
- **When:** `resolve_effort(&IsolatedModel::Specific("claude-opus-4-6"), "normal")`
- **Then:** Returns `Some("normal")`. Explicit value overrides the auto default of `low`.
- **Exit:** n/a (unit test)
- **Source fn:** `it_effort_normal_explicit` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-16](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-22: `imodel::auto` selects sonnet when `son_running=false` is sole inactive timer

- **Given:** Account quota data where `five_h_running=true` (`five_hour.resets_at=Some(_)`), `d7_running=true` (7d window absent or `seven_day.resets_at=Some(_)`), and `seven_day_sonnet.resets_at=None` (`son_idle=true`). The sole-son-trigger condition (`five_h_running AND d7_running AND son_idle`) is true. Helper: `mk_aq_with_son_idle_sole_trigger()`.
- **When:** `resolve_model(&aq, SubprocessModel::Auto)`
- **Then:** Returns `IsolatedModel::Specific("claude-sonnet-4-6")`. The 7d-Sonnet window (`seven_day_sonnet.resets_at`) activates only on Sonnet-family API calls; a Haiku touch subprocess cannot start it. When Sonnet is the sole inactive timer, `auto` routes to Sonnet to break the infinite per-call no-op loop (BUG-289 fix, TSK-292).
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_auto_selects_sonnet_for_sole_son_trigger` (in `src/usage/subprocess.rs #[cfg(test)]`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-23: `imodel::auto` selects haiku when Sonnet tier absent (`seven_day_sonnet=None`)

- **Given:** Account where `seven_day_sonnet=None` (API does not track a Sonnet quota tier for this account). Other timers running: `five_hour=running, seven_day=None (absent → d7_running=true)`.
- **When:** `resolve_model(&aq, SubprocessModel::Auto)`
- **Then:** Returns `IsolatedModel::Specific("claude-haiku-4-5-20251001")`. `son_idle = None.is_some_and(...) = false`; sole-son-trigger gate does NOT fire (requires `son_idle=true`). Haiku selected.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_auto_selects_haiku_when_son_tier_absent` (in `src/usage/subprocess.rs #[cfg(test)]`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-24: `imodel::auto` selects haiku when 7d timer idle (`seven_day=Some({resets_at:None})`)

- **Given:** Account where `seven_day=Some({resets_at:None})` (7d window tracked by API but no session started). Other timers: `five_hour=running, seven_day_sonnet=Some({resets_at:None})` (sole-son-trigger base except 7d is idle).
- **When:** `resolve_model(&aq, SubprocessModel::Auto)`
- **Then:** Returns `IsolatedModel::Specific("claude-haiku-4-5-20251001")`. `d7_running = seven_day.map_or(true, |p| p.resets_at.is_some()) = false` (closure fires; `is_some()` on `None` resets_at → `false`). Gate does NOT fire. Haiku selected.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_auto_selects_haiku_when_d7_idle` (in `src/usage/subprocess.rs #[cfg(test)]`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-25: `imodel::auto` selects sonnet when 7d running via explicit Some path

- **Given:** Account where `seven_day=Some({resets_at:Some("2026-06-15T10:00:00Z")})` (7d session active, field present and non-None). Other timers: `five_hour=running, seven_day_sonnet=Some({resets_at:None})` (sole-son-trigger base with 7d overridden to Some-running).
- **When:** `resolve_model(&aq, SubprocessModel::Auto)`
- **Then:** Returns `IsolatedModel::Specific("claude-sonnet-4-6")`. `d7_running = map_or(true, |p| p.resets_at.is_some()) = true` via closure branch; all three conditions hold: `five_h_running=true AND d7_running=true (Some path) AND son_idle=true` → gate fires → Sonnet.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_auto_selects_sonnet_when_d7_running_explicit` (in `src/usage/subprocess.rs #[cfg(test)]`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-26: `imodel::auto` selects haiku when 5h absent + 7d running via Some path

- **Given:** Account where `five_hour=None` (5h absent → `five_h_running=false`) and `seven_day=Some({resets_at:Some(...)})` (7d running via `map_or` Some-branch → `d7_running=true`). `seven_day_sonnet=Some({resets_at:None})` (Sonnet idle).
- **When:** `resolve_model(&aq, SubprocessModel::Auto)`
- **Then:** Returns `IsolatedModel::Specific("claude-haiku-4-5-20251001")`. Gate short-circuits at `five_h_running=false`; `&&` stops evaluation. The `d7=Some(running)` Some-branch is exercised but irrelevant — `false && ... = false`. Haiku selected.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_auto_selects_haiku_when_5h_absent_d7_some_running` (in `src/usage/subprocess.rs #[cfg(test)]`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-27: `imodel::auto` selects haiku when 7d idle + Sonnet running (two gate failures)

- **Given:** Account where `seven_day=Some({resets_at:None})` (7d idle → `d7_running=false`) and `seven_day_sonnet=Some({resets_at:Some(...)})` (Sonnet running → `son_idle=false`). `five_hour=running` (`five_h_running=true`).
- **When:** `resolve_model(&aq, SubprocessModel::Auto)`
- **Then:** Returns `IsolatedModel::Specific("claude-haiku-4-5-20251001")`. Both `d7_running=false` and `son_idle=false` block the gate. Haiku selected. Exercises `d7=Some(idle)` with `son=running` — distinct from FT-24 (son=idle) and EC-9b (d7=None).
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_auto_selects_haiku_when_d7_idle_and_son_running` (in `src/usage/subprocess.rs #[cfg(test)]`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../docs/feature/026_subprocess_model_effort.md)

---

### FT-28: `imodel::auto` selects haiku when 7d idle + Sonnet tier absent

- **Given:** Account where `seven_day=Some({resets_at:None})` (7d idle → `d7_running=false`) and `seven_day_sonnet=None` (absent → `son_idle=false`). `five_hour=running`.
- **When:** `resolve_model(&aq, SubprocessModel::Auto)`
- **Then:** Returns `IsolatedModel::Specific("claude-haiku-4-5-20251001")`. `d7_running=false` blocks; `son_idle=false` via `None.is_some_and(...)` also blocks. Both block the gate. Haiku selected. Distinct from FT-24 (son=idle present) and FT-23 (d7=None absent).
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_auto_selects_haiku_when_d7_idle_and_son_absent` (in `src/usage/subprocess.rs #[cfg(test)]`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../docs/feature/026_subprocess_model_effort.md)
