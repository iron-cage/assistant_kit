# Feature Tests: `run_isolated` / `IsolatedModel`

Test case planning for [feature/004_run_isolated.md](../../../docs/feature/004_run_isolated.md). Tests validate the `IsolatedModel` enum and `ISOLATED_DEFAULT_MODEL` constant introduced alongside the model parameter to `run_isolated()`.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FT-1 | `IsolatedModel::Default.model_id()` → `Some("claude-opus-4-6")` | Unit |
| FT-2 | `IsolatedModel::KeepCurrent.model_id()` → `None` | Unit |
| FT-3 | `IsolatedModel::Specific("custom-model").model_id()` → `Some("custom-model")` | Unit |
| FT-4 | `ISOLATED_DEFAULT_MODEL` constant → `"claude-opus-4-6"` | Unit |
| FT-5 | `run_isolated()` writes `CLAUDE.md` with immediate-response instruction to temp HOME | Unit |
| FT-6 | `ClaudeCommand` built with `with_home_isolation()` does not include `--chrome` in args | Unit |

## Test Coverage Summary

- Unit (offline, no credentials needed): 6 tests (FT-1, FT-2, FT-3, FT-4, FT-5, FT-6)

**Total:** 6 test cases

---

### FT-1: `IsolatedModel::Default.model_id()` → `Some("claude-opus-4-6")`

- **Given:** no external resources; `IsolatedModel::Default` constructed inline
- **When:** `IsolatedModel::Default.model_id()` called
- **Then:** returns `Some("claude-opus-4-6")`
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

### FT-4: `ISOLATED_DEFAULT_MODEL` constant equals `"claude-opus-4-6"`

- **Given:** no external resources
- **When:** `ISOLATED_DEFAULT_MODEL` constant value is asserted
- **Then:** equals `"claude-opus-4-6"`; `IsolatedModel::Default.model_id()` returns `Some(ISOLATED_DEFAULT_MODEL)` (Task 021 changed isolated default from Sonnet to Opus)
- **Source fn:** `t10_isolated_model_model_id_all_variants` (in `tests/isolated_test.rs`)
- **Source:** [feature/004_run_isolated.md](../../../docs/feature/004_run_isolated.md)

---

### FT-5: `run_isolated()` writes CLAUDE.md with immediate-response instruction to temp HOME

- **Given:** a temporary directory structure is prepared as by `run_isolated()`
- **When:** the CLAUDE.md content written to `<temp>/.claude/CLAUDE.md` is inspected (via the `ISOLATED_CLAUDE_MD` constant or equivalent)
- **Then:** the content contains at minimum the instruction to respond immediately to `--print` prompts without extended thinking, no preamble, and no tool use (AC-42)
- **Source fn:** `t_run_isolated_claude_md_content` (in `tests/isolated_test.rs`)
- **Source:** [feature/004_run_isolated.md](../../../docs/feature/004_run_isolated.md) AC-42

---

### FT-6: `ClaudeCommand` built with `with_home_isolation()` does not include `--chrome` in args

- **Given:** a `ClaudeCommand` built using `ClaudeCommand::new().with_home(<temp>).with_home_isolation()`
- **When:** the args list produced by the command is inspected
- **Then:** `--chrome` is absent from the arg list regardless of `ClaudeCommand::new()` defaults; home-isolated mode suppresses the chrome flag (AC-41)
- **Source fn:** `t_isolated_no_chrome_flag` (in `tests/isolated_test.rs`)
- **Source:** [feature/004_run_isolated.md](../../../docs/feature/004_run_isolated.md) AC-41
