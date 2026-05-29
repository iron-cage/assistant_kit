# Test: `effort::` Parameter

Edge case coverage for the `effort::` parameter on `.usage`. For `.account.use` `effort::` validation, see [command/005_account_use.md](../command/005_account_use.md) (IT-22). See [param/036_effort.md](../../../../docs/cli/param/036_effort.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `effort::auto` accepted with empty credential store | Valid Value |
| EC-2 | `effort::high` accepted with empty credential store | Valid Value |
| EC-3 | `effort::max` accepted with empty credential store | Valid Value |
| EC-4 | `effort::bad` exits 1, stderr names all five valid values | Invalid Value |
| EC-5 | `effort::high` — args contain `--effort high` regardless of model | Arg Construction |
| EC-6 | `effort::max` — args contain `--effort max` regardless of model | Arg Construction |
| EC-7 | `effort::auto` with resolved model=sonnet → `--effort high` | Behavioral Divergence |
| EC-8 | `effort::auto` with resolved model=opus → `--effort max` | Behavioral Divergence |
| EC-9 | `imodel::keep effort::auto` — neither `--model` nor `--effort` in args | Interaction |
| EC-10 | `effort::low` accepted with empty credential store | Valid Value |
| EC-11 | `effort::normal` accepted with empty credential store | Valid Value |
| EC-12 | `effort::low` — args contain `--effort low` regardless of model | Arg Construction |
| EC-13 | `effort::normal` — args contain `--effort normal` regardless of model | Arg Construction |
| EC-14 | `imodel::haiku effort::auto` — no `--effort` flag injected | Interaction |

---

### EC-1: `effort::auto` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage effort::auto`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter. No subprocess spawned.
- **Exit:** 0
- **Source fn:** `it124_effort_auto_accepted_empty_store_exits_0` (in `tests/cli/usage_test.rs`)
- **Source:** [param/036_effort.md](../../../../docs/cli/param/036_effort.md)

---

### EC-2: `effort::high` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage effort::high`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source fn:** `it130_effort_high_accepted_empty_store_exits_0` (in `tests/cli/usage_test.rs`)
- **Source:** [param/036_effort.md](../../../../docs/cli/param/036_effort.md)

---

### EC-3: `effort::max` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage effort::max`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source fn:** `it131_effort_max_accepted_empty_store_exits_0` (in `tests/cli/usage_test.rs`)
- **Source:** [param/036_effort.md](../../../../docs/cli/param/036_effort.md)

---

### EC-4: `effort::bad` exits 1 (invalid value)

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage effort::bad`
- **Then:** Exits 1. Stderr contains each of the five valid values: `auto`, `low`, `normal`, `high`, `max`.
- **Exit:** 1
- **Source fn:** `it125_effort_bogus_exits_1` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-11](../../../../docs/feature/026_subprocess_model_effort.md)

---

### EC-5: `effort::high` — args contain `--effort high` regardless of model

- **Given:** Account with resolved model = opus; `effort::high`.
- **When:** Unit test of `resolve_effort(&IsolatedModel::Specific("claude-opus-4-6"), "high")`
- **Then:** Returns `Some("high")`. Subprocess arg slice contains `--effort high`. Model does not influence the explicit value.
- **Exit:** n/a (unit test)
- **Source fn:** `it_effort_high_explicit` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-06](../../../../docs/feature/026_subprocess_model_effort.md)

---

### EC-6: `effort::max` — args contain `--effort max` regardless of model

- **Given:** Account with resolved model = sonnet; `effort::max`.
- **When:** Unit test of `resolve_effort(&IsolatedModel::Specific("claude-sonnet-4-6"), "max")`
- **Then:** Returns `Some("max")`. Subprocess arg slice contains `--effort max`. Model does not influence the explicit value.
- **Exit:** n/a (unit test)
- **Source fn:** `it_effort_max_explicit` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-07](../../../../docs/feature/026_subprocess_model_effort.md)

---

### EC-7: `effort::auto` with resolved model=sonnet → `--effort high` (Behavioral Divergence A)

- **Given:** Resolved model = `IsolatedModel::Specific("claude-sonnet-4-6")`; `effort::auto`.
- **When:** Unit test of `resolve_effort(&IsolatedModel::Specific("claude-sonnet-4-6"), "auto")`
- **Then:** Returns `Some("high")`. Subprocess arg slice contains `--effort high`.
- **Exit:** n/a (unit test)
- **Source fn:** `it_effort_auto_sonnet_path` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-05](../../../../docs/feature/026_subprocess_model_effort.md)

---

### EC-8: `effort::auto` with resolved model=opus → `--effort max` (Behavioral Divergence B)

- **Given:** Resolved model = `IsolatedModel::Specific("claude-opus-4-6")`; `effort::auto`. Same `effort::auto` parameter as EC-7 but different resolved model.
- **When:** Unit test of `resolve_effort(&IsolatedModel::Specific("claude-opus-4-6"), "auto")`
- **Then:** Returns `Some("max")`. Divergence from EC-7: the SAME `effort::auto` produces `high` for Sonnet and `max` for Opus, proving model-dependent resolution.
- **Exit:** n/a (unit test)
- **Source fn:** `it_effort_auto_opus_path` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-05](../../../../docs/feature/026_subprocess_model_effort.md)

---

### EC-9: `imodel::keep effort::auto` — neither `--model` nor `--effort` in args

- **Given:** Any account; `imodel::keep effort::auto`.
- **When:** Unit test of `resolve_effort(&IsolatedModel::KeepCurrent, "auto")`
- **Then:** Returns `None`. Combined arg slice contains neither `--model` nor `--effort`. The `imodel::keep` path in `resolve_effort()` skips effort injection when model is unknown.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_keep_effort_auto_no_effort_flag` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-05](../../../../docs/feature/026_subprocess_model_effort.md)

---

### EC-10: `effort::low` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage effort::low`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter. No subprocess spawned.
- **Exit:** 0
- **Source fn:** `it143_effort_low_accepted_empty_store_exits_0` (in `tests/cli/usage_test.rs`)
- **Source:** [param/036_effort.md](../../../../docs/cli/param/036_effort.md)

---

### EC-11: `effort::normal` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage effort::normal`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter. No subprocess spawned.
- **Exit:** 0
- **Source fn:** `it144_effort_normal_accepted_empty_store_exits_0` (in `tests/cli/usage_test.rs`)
- **Source:** [param/036_effort.md](../../../../docs/cli/param/036_effort.md)

---

### EC-12: `effort::low` — args contain `--effort low` regardless of model

- **Given:** Account with resolved model = opus; `effort::low`.
- **When:** Unit test of `resolve_effort(&IsolatedModel::Specific("claude-opus-4-6"), "low")`
- **Then:** Returns `Some("low")`. Subprocess arg slice contains `--effort low`. Model does not influence the explicit value.
- **Exit:** n/a (unit test)
- **Source fn:** `it_effort_low_explicit` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-15](../../../../docs/feature/026_subprocess_model_effort.md)

---

### EC-13: `effort::normal` — args contain `--effort normal` regardless of model

- **Given:** Account with resolved model = opus; `effort::normal`.
- **When:** Unit test of `resolve_effort(&IsolatedModel::Specific("claude-opus-4-6"), "normal")`
- **Then:** Returns `Some("normal")`. Subprocess arg slice contains `--effort normal`. Model does not influence the explicit value.
- **Exit:** n/a (unit test)
- **Source fn:** `it_effort_normal_explicit` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-16](../../../../docs/feature/026_subprocess_model_effort.md)

---

### EC-14: `imodel::haiku effort::auto` — no `--effort` flag injected

- **Given:** Resolved model = `IsolatedModel::Specific("claude-haiku-4-5-20251001")`; `effort::auto`.
- **When:** Unit test of `resolve_effort(&IsolatedModel::Specific("claude-haiku-4-5-20251001"), "auto")`
- **Then:** Returns `None`. No `--effort` flag is prepended. Haiku has no extended thinking support — `effort::auto` correctly injects nothing rather than an unsupported flag.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_haiku_effort_auto_no_effort_flag` (in `src/usage/subprocess.rs #[cfg(test)]`)
- **Source:** [feature/026_subprocess_model_effort.md AC-14](../../../../docs/feature/026_subprocess_model_effort.md)
