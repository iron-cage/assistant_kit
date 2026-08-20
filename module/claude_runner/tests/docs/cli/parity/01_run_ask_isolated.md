# Parity Test: run / ask / isolated

Cross-command behavioral parity test planning for `clr run`, `clr ask`, and `clr isolated`. See [parity/001_run_ask_isolated.md](../../../../docs/cli/parity/001_run_ask_isolated.md) for specification.

`ask` is a pure alias for `run` (identical code path). `isolated` is a distinct command with a minimal param set, hardcoded defaults, and credential isolation. Tests verify alias equivalence and key behavioral divergences.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| PC-1 | `clr ask --dry-run "X"` produces byte-identical output to `clr run --dry-run "X"` | Alias Equivalence |
| PC-2 | ask carries through all common run flags unchanged (--effort, --model, --topic) | Param Surface Parity |
| PC-3 | isolated injects hardcoded model (opus); run/ask inject no model by default | Model Injection Divergence |
| PC-4 | isolated creates a fresh temp HOME; run/ask execute in real `$HOME` | HOME Divergence |
| PC-5 | isolated print-mode timeout is 30 s; run/ask default is 0 = unlimited (TSK-503) | Timeout Divergence |

## Test Coverage Summary

- Alias Equivalence: 1 test (PC-1)
- Param Surface Parity: 1 test (PC-2)
- Model Injection Divergence: 1 test (PC-3)
- HOME Divergence: 1 test (PC-4)
- Timeout Divergence: 1 test (PC-5)

**Total:** 5 tests

---

### PC-1: ask and run produce identical dry-run output

- **Given:** clr binary available; fresh session directory
- **When:** `clr ask --dry-run "What does X do?"` and `clr run --dry-run "What does X do?"` with identical environment
- **Then:** stdout is byte-identical — same assembled command, same auto-injected flags, same env vars
- **Exit:** 0 for both
- **Source:** [parity/001_run_ask_isolated.md](../../../../docs/cli/parity/001_run_ask_isolated.md), [command/05_ask.md](../../../../docs/cli/command/05_ask.md)
- **Implemented by:** `ask_command_test.rs::t01_ask_run_dry_run_equivalence`

---

### PC-2: ask carries all run params through unchanged

- **Given:** clr binary; various run-control flags
- **When:** `clr ask --dry-run --effort high "q"`, `clr ask --dry-run --model sonnet "q"`, `clr ask --dry-run --topic feat "q"`
- **Then:** each flag appears in assembled output identically to the same run invocation; ask does not force or suppress any param
- **Exit:** 0
- **Source:** [parity/001_run_ask_isolated.md](../../../../docs/cli/parity/001_run_ask_isolated.md), [command/05_ask.md](../../../../docs/cli/command/05_ask.md)
- **Implemented by:** `ask_command_test.rs::t10_ask_topic_effective_dir` (--topic ✅), `user_story_creds_isolated_test.rs::us17_4_model_in_ask_command` (--model ✅), `ask_command_test.rs::t13_ask_explicit_effort_passthrough` (--effort ✅)

---

### PC-3: isolated injects a default model (config-pref-aware); run/ask inject no model

- **Given:** clr binary; valid credentials file; `--trace` flag
- **When:** `clr isolated --creds <f> --trace "Fix bug"` vs `clr run --trace "Fix bug"`
- **Then:** isolated stderr contains `--model opus` (`ISOLATED_DEFAULT_MODEL`) in the assembled command — unless a `model` preference is set in project `.clr.toml` or user `~/.clr/config.toml`, in which case that preference appears instead (resolved once at `run_isolated_command()` entry, so preview, no-file, and `--file` paths all agree — BUG-485); run stderr does not contain `--model` (uses claude binary default)
- **Exit:** 1 (claude absent in test env)
- **Source:** [parity/001_run_ask_isolated.md](../../../../docs/cli/parity/001_run_ask_isolated.md), [invariant/005_isolated_subprocess_defaults.md](../../../../docs/invariant/005_isolated_subprocess_defaults.md)
- **Implemented by:** `isolated_defaults_test.rs::isd_01_isolated_default_model_is_opus`, `isolated_defaults_test.rs::bug485_dry_run_preview_shows_config_model_pref_not_hardcoded_default` (config-pref parity ✅)

---

### PC-4: isolated creates temp HOME; run/ask use real HOME

- **Given:** clr binary; real `$HOME` with existing `.claude/` configuration
- **When:** `clr isolated --creds <f> "x"` with a fake `claude` binary that echoes `$HOME` (fake-binary pattern per CT-6)
- **Then:** fake-binary stdout contains a temp path (not the real `$HOME`) as the subprocess HOME env var; isolated's temp HOME contains `.claude/.credentials.json` from `--creds` and a minimal `CLAUDE.md` (CT-6)
- **Exit:** 0
- **Source:** [parity/001_run_ask_isolated.md](../../../../docs/cli/parity/001_run_ask_isolated.md), [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md)
- **Implemented by:** `isolated_correctness_test.rs` CT-1 through CT-6 (HOME isolation and CLAUDE.md structure), `isolated_correctness_test.rs::ct7_isolated_subprocess_sees_temp_home` (subprocess HOME env var ✅)

---

### PC-5: isolated print-mode timeout is 30 s; run's default is 0 = unlimited

- **Given:** clr binary; valid credentials file; `--trace` flag
- **When:** `clr isolated --creds <f> --trace "x"` (trace shows timeout in header)
- **Then:** isolated trace header shows `# timeout: 30s`; run's print-mode default is `0` = unlimited (per `DEFAULT_PRINT_TIMEOUT_SECS = 0`, TSK-503 — a run/ask watchdog exists only when expressed)
- **Exit:** 1 (claude absent)
- **Source:** [parity/001_run_ask_isolated.md](../../../../docs/cli/parity/001_run_ask_isolated.md), [param/020_timeout.md](../../../../docs/cli/param/020_timeout.md)
- **Implemented by:** `param_trace_edge_cases_test.rs::s58_isolated_trace_credential_format`
