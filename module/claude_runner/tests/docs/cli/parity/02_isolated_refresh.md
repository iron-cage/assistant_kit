# Parity Test: isolated / refresh

Cross-command behavioral parity test planning for `clr isolated` and `clr refresh`. See [parity/002_isolated_refresh.md](../../../../docs/cli/parity/002_isolated_refresh.md) for specification.

Both commands share `run_isolated()` infrastructure but diverge on purpose, defaults, and param surface. `isolated` executes a user task; `refresh` performs an OAuth credential exchange only.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| PC-1 | isolated injects opus model; refresh injects sonnet model | Model Divergence |
| PC-2 | isolated injects `--effort max`; refresh injects `--effort low` | Effort Divergence |
| PC-3 | isolated default timeout 30 s; refresh default timeout 45 s | Timeout Divergence |
| PC-4 | isolated injects `--chrome` (default on); refresh injects `--no-chrome` | Chrome Divergence |
| PC-5 | isolated accepts user MESSAGE; refresh uses hardcoded `"."` with no user message | Param Surface Divergence |

## Test Coverage Summary

- Model Divergence: 1 test (PC-1)
- Effort Divergence: 1 test (PC-2)
- Timeout Divergence: 1 test (PC-3)
- Chrome Divergence: 1 test (PC-4)
- Param Surface Divergence: 1 test (PC-5)

**Total:** 5 tests

---

### PC-1: isolated injects opus model; refresh injects sonnet model

- **Given:** `ISOLATED_DEFAULT_MODEL` and `REFRESH_DEFAULT_MODEL` constants in source
- **When:** `ISOLATED_DEFAULT_MODEL` value read; `REFRESH_DEFAULT_MODEL` value read
- **Then:** `ISOLATED_DEFAULT_MODEL == "claude-opus-4-8"` (heavyweight model for user tasks); `REFRESH_DEFAULT_MODEL == "claude-sonnet-5"` (lightweight model for trivial OAuth ping)
- **Source:** [parity/002_isolated_refresh.md](../../../../docs/cli/parity/002_isolated_refresh.md), [invariant/005_isolated_subprocess_defaults.md](../../../../docs/invariant/005_isolated_subprocess_defaults.md)
- **Implemented by:** `isolated_defaults_test.rs::isd_01_isolated_default_model_is_opus`, `isolated_defaults_test.rs::isd_02_refresh_default_model_is_sonnet`

---

### PC-2: isolated injects --effort max; refresh injects --effort low

- **Given:** clr binary; valid credentials file; `--trace` flag
- **When:** `clr isolated --creds <f> --trace "x"` vs `clr refresh --creds <f> --trace`
- **Then:** isolated trace shows `--effort max` in assembled command; refresh trace shows `--effort low` (OAuth exchange needs no reasoning budget)
- **Exit:** 1 (claude absent)
- **Source:** [parity/002_isolated_refresh.md](../../../../docs/cli/parity/002_isolated_refresh.md), [invariant/005_isolated_subprocess_defaults.md](../../../../docs/invariant/005_isolated_subprocess_defaults.md)
- **Implemented by:** `isolated_defaults_test.rs::isd_03_isolated_trace_shows_effort_max`, `isolated_defaults_test.rs::isd_04_refresh_trace_shows_effort_low`

---

### PC-3: isolated default timeout 30 s; refresh default timeout 45 s

- **Given:** clr binary; valid credentials file; `--trace` flag
- **When:** `clr isolated --creds <f> --trace "x"` vs `clr refresh --creds <f> --trace`
- **Then:** isolated trace header shows `# timeout: 30s`; refresh trace header shows `# timeout: 45s` (extra headroom for network OAuth exchange)
- **Exit:** 1 (claude absent)
- **Source:** [parity/002_isolated_refresh.md](../../../../docs/cli/parity/002_isolated_refresh.md), [param/020_timeout.md](../../../../docs/cli/param/020_timeout.md)
- **Implemented by:** `param_trace_edge_cases_test.rs::s58_isolated_trace_credential_format`, `param_trace_edge_cases_test.rs::s59_refresh_trace_credential_format`

---

### PC-4: isolated injects --chrome; refresh injects --no-chrome

- **Given:** clr binary; valid credentials file; `--trace` flag
- **When:** `clr isolated --creds <f> --trace "x"` vs `clr refresh --creds <f> --trace`
- **Then:** isolated trace shows `--chrome` in assembled command (ClaudeCommand default, not suppressed); refresh trace shows `--no-chrome` (OAuth exchange is HTTP-only, no browser interaction needed)
- **Exit:** 1 (claude absent)
- **Source:** [parity/002_isolated_refresh.md](../../../../docs/cli/parity/002_isolated_refresh.md), [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md), [command/04_refresh.md](../../../../docs/cli/command/04_refresh.md)
- **Implemented by:** `isolated_defaults_test.rs::isd_09_refresh_trace_shows_no_chrome`, `isolated_defaults_test.rs::isd_10_isolated_trace_does_not_suppress_chrome`

---

### PC-5: isolated accepts user MESSAGE; refresh uses hardcoded "."

- **Given:** clr binary; valid credentials file
- **When:** `clr isolated --creds <f> --dry-run "Fix the bug"` (preview confirms MESSAGE in assembled command) and `clr refresh --creds <f> "Fix the bug"` (positional arg triggers parse-time rejection)
- **Then:** isolated dry-run preview contains `"Fix the bug"` in the assembled subprocess command; refresh exits 1 with an "unexpected argument" error and does not attempt to spawn claude
- **Exit:** 0 (isolated dry-run); 1 (refresh: parse error before spawn)
- **Source:** [parity/002_isolated_refresh.md](../../../../docs/cli/parity/002_isolated_refresh.md), [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md), [command/04_refresh.md](../../../../docs/cli/command/04_refresh.md)
- **Implemented by:** `isolated_plan034_test.rs::it13_dry_run_includes_message` (isolated MESSAGE acceptance ✅), `refresh_test.rs::test_it10_refresh_rejects_positional_message` (refresh MESSAGE rejection ✅)
