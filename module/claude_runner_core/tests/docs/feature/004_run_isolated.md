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
| FT-1 | `IsolatedModel::Default.model_id()` → `Some("claude-sonnet-5")` | Unit |
| FT-2 | `IsolatedModel::KeepCurrent.model_id()` → `None` | Unit |
| FT-3 | `IsolatedModel::Specific("custom-model").model_id()` → `Some("custom-model")` | Unit |
| FT-4 | `DEFAULT_MODEL` constant → `"claude-sonnet-5"` | Unit |
| FT-5 | `run_isolated()` writes `CLAUDE.md` with immediate-response instruction to temp HOME | Unit |
| FT-6 | `ClaudeCommand` built with `with_home_isolation()` does not include `--chrome` in args | Unit |
| FT-7 | `resolve_isolated_default_model()` reads the user tier from `$CLR_CONFIG_DIR/config.toml` when set, `HOME` optional; project still wins; no user tier → no file read | Unit |
| FT-8 | `user_config_path()` → `$CLR_CONFIG_DIR/config.toml`, else `$HOME/.clr/config.toml`, else `None` (empty counts as unset) | Unit |
| FT-9 | Timeout-with-refreshed-credentials result preserves recovered `stdout`/`stderr` on both timeout branches (BUG-005) | Unit |

## Test Coverage Summary

- Unit (offline, no credentials needed): 9 tests (FT-1 through FT-9)

**Total:** 9 test cases

---

### FT-1: `IsolatedModel::Default.model_id()` → `Some("claude-sonnet-5")`

- **Given:** no external resources; `IsolatedModel::Default` constructed inline
- **When:** `IsolatedModel::Default.model_id()` called
- **Then:** returns `Some("claude-sonnet-5")` (`DEFAULT_MODEL`, an explicit ID rather than the `sonnet` alias)
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

### FT-4: `DEFAULT_MODEL` constant equals `"claude-sonnet-5"`

- **Given:** no external resources
- **When:** `DEFAULT_MODEL` constant value is asserted
- **Then:** equals `"claude-sonnet-5"`; `IsolatedModel::Default.model_id()` returns `Some(DEFAULT_MODEL)`; the explicit ID pins one model, where the `sonnet` alias would resolve per release and per settings
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

- **Given:** `$CLR_CONFIG_DIR/config.toml` sets `model`; `$HOME/.clr/config.toml` sets a different `model` (`t8`), or a project `.clr.toml` sets a third value (`t9`), or `HOME` is unset (`t10`).
  `t11`: no user tier (`CLR_CONFIG_DIR` unset, `HOME` unset or empty) and a project `.clr.toml` that sets `model`.
  `t12`: the override has no `config.toml`, in a directory that exists and then in one that doesn't, and `$HOME/.clr/config.toml` sets `model`; `t13`: the same `HOME` file with the override empty.
  `t14`: the same `HOME` file with an override `config.toml` that sets only `effort`, is empty, or holds only a comment.
- **When:** `resolve_isolated_default_model()` called from the project directory
- **Then:** `t8` and `t10` return the override's value; `t9` returns the project value — the override relocates the user tier only (AC-43).
  `t11` returns `None`: with no user tier the project file isn't read either (open decision D-14).
  `t12` returns `None` both times (a set override is the user tier even without a file or a directory); `t13` returns `HOME`'s value (an empty override counts as unset).
  `t14` returns `None` all three times: an override `config.toml` without `model` is still the whole user tier, so `HOME`'s file isn't read.
- **Source fn:** `t8_clr_config_dir_overrides_home_user_tier`, `t9_project_tier_overrides_clr_config_dir_user_tier`, `t10_clr_config_dir_honored_without_home`,
  `t11_no_user_tier_skips_project_tier`, `t12_clr_config_dir_without_file_does_not_fall_back_to_home`, `t13_empty_clr_config_dir_falls_back_to_home`,
  `t14_clr_config_dir_file_without_model_does_not_fall_back_to_home` (in `tests/isolated_model_resolution_test.rs`)
- **Source:** [feature/004_run_isolated.md](../../../docs/feature/004_run_isolated.md) AC-43

---

### FT-8: `user_config_path()` decision table (BUG-007, BUG-560)

- **Given:** every combination of `CLR_CONFIG_DIR` and `HOME` as set, empty, or unset
- **When:** `user_config_path_from()` called with those values, and `user_config_path()` called against the process env
- **Then:** a non-empty override yields `<override>/config.toml`, and a relative override is taken verbatim.
  Otherwise a non-empty `HOME` yields `<HOME>/.clr/config.toml`, and otherwise the result is `None`, not a cwd-relative `.clr/config.toml` (AC-43).
- **Source fn:** `t01_override_wins_over_home` through `t12_empty_override_unset_home_is_none` (in `tests/config_path_test.rs`)
- **Source:** [feature/004_run_isolated.md](../../../docs/feature/004_run_isolated.md) AC-43

---

### FT-9: Timeout-with-refreshed-credentials result preserves recovered `stdout`/`stderr` (BUG-005)

- **Given:** a fake `claude` that prints a marker to stdout, rewrites `.claude/.credentials.json` in its isolated HOME, then outlives the deadline
- **When:** `run_isolated()` times out with `credentials.is_some()` true
- **Then:** returns `Ok(IsolatedRunResult { exit_code: -1, stdout, credentials: Some(..), .. })` where `stdout` contains the pre-deadline marker — the already-recovered output is forwarded, not discarded as `String::new()`
- **Source fn:** `timeout_with_credentials_preserves_partial_stdout` (in `tests/bug_005_test.rs`)
- **Source:** [feature/004_run_isolated.md](../../../docs/feature/004_run_isolated.md) AC-37
