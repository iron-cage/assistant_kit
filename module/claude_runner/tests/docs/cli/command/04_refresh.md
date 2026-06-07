# Test: `refresh`

Integration test planning for the `refresh` command. See [command/03_refresh.md](../../../../docs/cli/command/03_refresh.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `--creds file.json` → refresh succeeds, exit 0, creds updated in-place | Happy Path |
| IT-2 | `--creds missing.json` → exit 1, file-not-found error | Error: Missing Creds |
| IT-3 | `--creds file.json --timeout 90` → explicit timeout applied, exit 0 | Custom Timeout |
| IT-4 | `--creds file.json --timeout 0` → unlimited (no watchdog) | Timeout |
| IT-5 | No `--creds`, `CLR_CREDS` unset → defaults to `$HOME/.claude/.credentials.json`; trace confirms | Default Fallback |
| IT-6 | `--creds file.json --timeout abc` → exit 1, invalid timeout error | Error: Invalid Timeout |
| IT-7 | `--creds file.json --trace` → call details printed to stderr before execution | Trace |
| IT-8 | `clr refresh --help` → exit 0, prints refresh-specific help | Help |

## Test Coverage Summary

- Happy Path: 1 test (IT-1)
- Error Handling: 2 tests (IT-2, IT-6)
- Timeout Behavior: 2 tests (IT-3, IT-4)
- Default Fallback: 1 test (IT-5)
- Trace: 1 test (IT-7)
- Help: 1 test (IT-8)

**Total:** 8 test cases

---

### IT-1: `--creds file.json` → refresh succeeds, exit 0, creds updated in-place

- **Setup:** valid but expired credentials JSON at `/tmp/it1_refresh_creds.json`; `claude` binary in PATH
- **Command:** `clr refresh --creds /tmp/it1_refresh_creds.json`
- **Expected behavior:** subprocess spawned with `HOME=<temp>` and `["--print", "."]`; OAuth token refreshed; `/tmp/it1_refresh_creds.json` overwritten with updated token; exit 0
- **Exit:** 0
- **Note:** lim_it — requires valid but expiring credentials
- **Source:** [command/03_refresh.md](../../../../docs/cli/command/03_refresh.md)

---

### IT-2: `--creds missing.json` → exit 1, file-not-found error

- **Setup:** `/tmp/it2_refresh_missing.json` does not exist
- **Command:** `clr refresh --creds /tmp/it2_refresh_missing.json`
- **Expected behavior:** exit 1; stderr contains "not found" or equivalent; no subprocess launched
- **Exit:** 1
- **Source:** [command/03_refresh.md](../../../../docs/cli/command/03_refresh.md)
- **Note:** Implemented; test function `test_it2_creds_file_not_found` in `tests/refresh_test.rs`

---

### IT-3: `--creds file.json --timeout 90` → explicit timeout applied

- **Setup:** valid credentials JSON at `/tmp/it3_refresh_creds.json`
- **Command:** `clr refresh --creds /tmp/it3_refresh_creds.json --timeout 90`
- **Expected behavior:** subprocess launched with 90-second deadline (not the 45-second default); behavior identical to an explicit `--timeout 90`; exit 0 on successful refresh
- **Exit:** 0
- **Note:** lim_it — requires valid credentials; validates explicit timeout overrides default
- **Source:** [command/03_refresh.md](../../../../docs/cli/command/03_refresh.md), [--timeout](../../../../docs/cli/param/020_timeout.md)

---

### IT-4: `--timeout 0` → unlimited (no watchdog)

- **Setup:** valid credentials JSON at `/tmp/it4_refresh_creds.json`
- **Command:** `clr refresh --creds /tmp/it4_refresh_creds.json --timeout 0`
- **Expected behavior:** `0` disables the watchdog entirely — subprocess runs until it exits naturally (matching `run`/`ask` semantics); exit code is the subprocess exit code, not 2
- **Exit:** 0 or passthrough
- **Note:** lim_it variant for live-credential environments; offline fake-claude variant `test_it4_timeout_zero_exits_two` in `tests/refresh_test.rs` needs updating to expect non-timeout behavior
- **Source:** [command/03_refresh.md](../../../../docs/cli/command/03_refresh.md)

---

### IT-5: No `--creds` → defaults to `$HOME/.claude/.credentials.json`; trace confirms path

- **Setup:** `$HOME/.claude/.credentials.json` exists (readable; content `{}`; no live credentials needed — trace fires before subprocess attempt); `CLR_CREDS` unset
- **Command:** `clr refresh --trace`
- **Expected behavior:** trace stderr contains `# creds: <HOME>/.claude/.credentials.json`; subprocess attempt fails (claude absent in test environment)
- **Exit:** 1
- **Source:** [command/03_refresh.md](../../../../docs/cli/command/03_refresh.md), [--creds](../../../../docs/cli/param/019_creds.md)
- **Note:** Implemented; test function `t2_refresh_no_creds_uses_home_default` in `tests/creds_default_test.rs`

---

### IT-6: `--timeout abc` → exit 1, invalid timeout error

- **Setup:** valid credentials JSON at `/tmp/it6_refresh_creds.json`
- **Command:** `clr refresh --creds /tmp/it6_refresh_creds.json --timeout abc`
- **Expected behavior:** exit 1; stderr contains invalid `--timeout` error; no subprocess launched
- **Exit:** 1
- **Source:** [type/09_timeout_secs.md](../../../../docs/cli/type/09_timeout_secs.md)
- **Note:** Implemented; test function `test_it6_invalid_timeout` in `tests/refresh_test.rs`

---

### IT-7: `--trace` → credential trace on stderr before execution attempt

- **Setup:** credentials JSON written to a temp file at `/tmp/it7_refresh_creds.json` (file is readable; content `{}`; no live credentials needed — trace fires before subprocess attempt)
- **Command:** `clr refresh --creds /tmp/it7_refresh_creds.json --trace`
- **Expected behavior:** stderr contains `# clr refresh`, `# creds: /tmp/it7_refresh_creds.json`, `# timeout: 45s`, env var block (including `CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000`), and `claude --model claude-sonnet-4-6 --no-chrome --effort low --no-session-persistence --print "."` before any subprocess attempt; subprocess attempt fails (claude absent in test environment)
- **Exit:** 1
- **Source:** [command/03_refresh.md](../../../../docs/cli/command/03_refresh.md), [--trace](../../../../docs/cli/param/013_trace.md), [invariant/004_trace_universality.md](../../../../docs/invariant/004_trace_universality.md)
- **Note:** Implemented; test function `it_04_refresh_trace_stderr_output` in `tests/invariant_trace_universality_test.rs`

---

### IT-8: `clr refresh --help` → exit 0, prints refresh-specific help

- **Command:** `clr refresh --help` (also: `clr refresh -h`)
- **Expected behavior:** exit 0; stdout contains `--creds`, `--timeout`, `--trace`, and `--help`; no subprocess launched; no error in stderr
- **Exit:** 0
- **Source:** [command/03_refresh.md](../../../../docs/cli/command/03_refresh.md)
- **Note:** Implemented; test function `test_it8_help_exits_zero` in `tests/refresh_test.rs`
