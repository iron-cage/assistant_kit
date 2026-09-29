# Feature Tests: `run_isolated` / `IsolatedModel`

### Scope

- **Purpose**: FT- test cases verifying the `IsolatedModel` enum, `DEFAULT_MODEL` constant, and `run_isolated()` home-isolation behavior.
- **Responsibility**: Acceptance criteria confirming model-id resolution per `IsolatedModel` variant, isolated `CLAUDE.md` content, and `--chrome` suppression under home isolation.
- **In Scope**: `IsolatedModel::Default`/`KeepCurrent`/`Specific` `.model_id()`, `DEFAULT_MODEL` value, `run_isolated()` CLAUDE.md write, `with_home_isolation()` chrome-flag suppression, the user-tier location `resolve_isolated_default_model()` reads (`user_config_path()`).
- **Out of Scope**: stdin file piping (-> `005_stdin_file.md`), CLAUDECODE env var unsetting (-> `006_unset_claudecode.md`).

Test case planning for [feature/004_run_isolated.md](../../../docs/feature/004_run_isolated.md). Tests validate the `IsolatedModel` enum and `DEFAULT_MODEL` constant introduced alongside the model parameter to `run_isolated()`.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FT-1 | `IsolatedModel::Default.model_id()` → `Some("claude-opus-5-5")` | Unit |
| FT-2 | `IsolatedModel::KeepCurrent.model_id()` → `None` | Unit |
| FT-3 | `IsolatedModel::Specific("custom-model").model_id()` → `Some("custom-model")` | Unit |
| FT-4 | `DEFAULT_MODEL` constant → `"claude-opus-5-5"` | Unit |
| FT-5 | `run_isolated()` writes `CLAUDE.md` with immediate-response instruction to temp HOME | Unit |
| FT-6 | `ClaudeCommand` built with `with_home_isolation()` does not include `--chrome` in args | Unit |
| FT-7 | `resolve_isolated_default_model()` reads the user tier from `$CLR_CONFIG_DIR/config.toml` when set, `HOME` optional; project `.clr.toml` still wins | Unit |
| FT-8 | `user_config_path()` → `$CLR_CONFIG_DIR/config.toml`, else `$HOME/.clr/config.toml`, else `None` (empty counts as unset) | Unit |

## Test Coverage Summary

- Unit (offline, no credentials needed): 8 tests (FT-1 through FT-8)

**Total:** 8 test cases

---

### FT-1: `IsolatedModel::Default.model_id()` → `Some("claude-opus-5-5")`

- **Given:** no external resources; `IsolatedModel::Default` constructed inline
- **When:** `IsolatedModel::Default.model_id()` called
- **Then:** returns `Some("claude-opus-5-5")` (`DEFAULT_MODEL`, an explicit ID rather than the `opus` alias)
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

### FT-4: `DEFAULT_MODEL` constant equals `"claude-opus-5-5"`

- **Given:** no external resources
- **When:** `DEFAULT_MODEL` constant value is asserted
- **Then:** equals `"claude-opus-5-5"`; `IsolatedModel::Default.model_id()` returns `Some(DEFAULT_MODEL)`; the explicit ID pins one model, where the `opus` alias would resolve per release and per settings
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

---

### FT-7: `resolve_isolated_default_model()` honors `CLR_CONFIG_DIR` for the user tier (BUG-007)

- **Given:** `$CLR_CONFIG_DIR/config.toml` sets `model`; `$HOME/.clr/config.toml` sets a different `model` (`t8`), or a project `.clr.toml` sets a third value (`t9`), or `HOME` is unset (`t10`)
- **When:** `resolve_isolated_default_model()` called from the project directory
- **Then:** `t8` and `t10` return the override's value; `t9` returns the project value — the override relocates the user tier only (AC-43)
- **Source fn:** `t8_clr_config_dir_overrides_home_user_tier`, `t9_project_tier_overrides_clr_config_dir_user_tier`, `t10_clr_config_dir_honored_without_home` (in `tests/isolated_model_resolution_test.rs`)
- **Source:** [feature/004_run_isolated.md](../../../docs/feature/004_run_isolated.md) AC-43

---

### FT-8: `user_config_path()` decision table (BUG-007, BUG-560)

- **Given:** every combination of `CLR_CONFIG_DIR` and `HOME` as set, empty, or unset
- **When:** `user_config_path_from()` called with those values, and `user_config_path()` called against the process env
- **Then:** a non-empty override yields `<override>/config.toml` (a relative override is taken verbatim); otherwise a non-empty `HOME` yields `<HOME>/.clr/config.toml`; otherwise `None` — never a cwd-relative path (AC-43)
- **Source fn:** `t01_override_wins_over_home` through `t11_wrapper_none_without_env` (in `tests/config_path_test.rs`)
- **Source:** [feature/004_run_isolated.md](../../../docs/feature/004_run_isolated.md) AC-43
