# Feature Tests: `run_isolated` / `IsolatedModel`

Test case planning for [feature/004_run_isolated.md](../../../docs/feature/004_run_isolated.md). Tests validate the `IsolatedModel` enum and `ISOLATED_DEFAULT_MODEL` constant introduced alongside the model parameter to `run_isolated()`.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FT-1 | `IsolatedModel::Default.model_id()` → `Some("claude-sonnet-4-6")` | Unit |
| FT-2 | `IsolatedModel::KeepCurrent.model_id()` → `None` | Unit |
| FT-3 | `IsolatedModel::Specific("custom-model").model_id()` → `Some("custom-model")` | Unit |
| FT-4 | `ISOLATED_DEFAULT_MODEL` constant → `"claude-sonnet-4-6"` | Unit |

## Test Coverage Summary

- Unit (offline, no credentials needed): 4 tests (FT-1, FT-2, FT-3, FT-4)

**Total:** 4 test cases

---

### FT-1: `IsolatedModel::Default.model_id()` → `Some("claude-sonnet-4-6")`

- **Given:** no external resources; `IsolatedModel::Default` constructed inline
- **When:** `IsolatedModel::Default.model_id()` called
- **Then:** returns `Some("claude-sonnet-4-6")`
- **Source fn:** `t10_isolated_model_model_id_all_variants` (in `tests/isolated_test.rs`)
- **Source:** [feature/004_run_isolated.md](../../../docs/feature/004_run_isolated.md)

---

### FT-2: `IsolatedModel::KeepCurrent.model_id()` → `None`

- **Given:** no external resources; `IsolatedModel::KeepCurrent` constructed inline
- **When:** `IsolatedModel::KeepCurrent.model_id()` called
- **Then:** returns `None` (no `--model` flag is injected into the subprocess command)
- **Source fn:** `t10_isolated_model_model_id_all_variants` (in `tests/isolated_test.rs`)
- **Source:** [feature/004_run_isolated.md](../../../docs/feature/004_run_isolated.md)

---

### FT-3: `IsolatedModel::Specific("custom-model").model_id()` → `Some("custom-model")`

- **Given:** no external resources; `IsolatedModel::Specific("custom-model".to_string())` constructed inline
- **When:** `.model_id()` called
- **Then:** returns `Some("custom-model")`; the returned `&str` matches the string passed at construction
- **Source fn:** `t10_isolated_model_model_id_all_variants` (in `tests/isolated_test.rs`)
- **Source:** [feature/004_run_isolated.md](../../../docs/feature/004_run_isolated.md)

---

### FT-4: `ISOLATED_DEFAULT_MODEL` constant equals `"claude-sonnet-4-6"`

- **Given:** no external resources
- **When:** `ISOLATED_DEFAULT_MODEL` constant value is asserted
- **Then:** equals `"claude-sonnet-4-6"`; `IsolatedModel::Default.model_id()` returns `Some(ISOLATED_DEFAULT_MODEL)`
- **Source fn:** `t10_isolated_model_model_id_all_variants` (in `tests/isolated_test.rs`)
- **Source:** [feature/004_run_isolated.md](../../../docs/feature/004_run_isolated.md)
