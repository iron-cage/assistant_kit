# Parameter :: `--timeout`

Edge case test planning for the `--timeout` parameter. See [020_timeout.md](../../../../docs/cli/param/020_timeout.md) for specification.

**Source:** [param/020_timeout.md](../../../../docs/cli/param/020_timeout.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--timeout 30` → accepted (default value) | Behavioral Divergence |
| EC-2 | `--timeout 0` → unlimited (no watchdog) | Behavioral Divergence |
| EC-3 | `--timeout 3600` → large value accepted | Valid |
| EC-4 | `--timeout -1` → exit 1, negative not accepted | Invalid |
| EC-5 | `--timeout abc` → exit 1, non-numeric rejected | Invalid |
| EC-6 | `--timeout` without value → exit 1, requires argument | Missing Value |

## Test Coverage Summary

- Behavioral Divergence: 3 tests (EC-1, EC-2, EC-7)
- Valid: 1 test (EC-3)
- Invalid: 2 tests (EC-4, EC-5)
- Missing Value: 1 test (EC-6)

**Total:** 7 test cases

---

### EC-1: `--timeout 30` → accepted (default value)

- **Given:** credentials JSON at `/tmp/ec1_to_creds.json`
- **When:** `clr isolated --creds /tmp/ec1_to_creds.json --timeout 30 "test"`
- **Then:** no parse error; subprocess runs with 30-second window (same as omitting `--timeout`)
- **Exit:** 0
- **Source:** [020_timeout.md](../../../../docs/cli/param/020_timeout.md)
- **Commands:** isolated, refresh

---

### EC-2: `--timeout 0` → unlimited (no watchdog)

- **Given:** credentials JSON at `/tmp/ec2_to_creds.json`
- **When:** `clr isolated --creds /tmp/ec2_to_creds.json --timeout 0 "test"`
- **Then:** no parse error; `0` disables the watchdog entirely — subprocess runs until it exits naturally (no deadline enforced); exit code is the subprocess exit code, not 2
- **Exit:** 0 or passthrough
- **Source:** [020_timeout.md](../../../../docs/cli/param/020_timeout.md), [invariant/005_isolated_subprocess_defaults.md](../../../../docs/invariant/005_isolated_subprocess_defaults.md)
- **Commands:** isolated, refresh

---

### EC-3: `--timeout 3600` → large value accepted

- **Given:** credentials JSON at `/tmp/ec3_to_creds.json`
- **When:** `clr isolated --creds /tmp/ec3_to_creds.json --timeout 3600 "test"`
- **Then:** no parse error; subprocess runs with 1-hour window
- **Exit:** 0
- **Source:** [020_timeout.md](../../../../docs/cli/param/020_timeout.md)
- **Commands:** isolated, refresh

---

### EC-4: `--timeout -1` → exit 1, negative rejected

- **Given:** credentials JSON at `/tmp/ec4_to_creds.json`
- **When:** `clr isolated --creds /tmp/ec4_to_creds.json --timeout -1 "test"`
- **Then:** exit 1; stderr contains invalid `--timeout` error; no subprocess launched
- **Exit:** 1
- **Source:** [type/09_timeout_secs.md](../../../../docs/cli/type/09_timeout_secs.md)
- **Commands:** isolated, refresh

---

### EC-5: `--timeout abc` → exit 1, non-numeric rejected

- **Given:** credentials JSON at `/tmp/ec5_to_creds.json`
- **When:** `clr isolated --creds /tmp/ec5_to_creds.json --timeout abc "test"`
- **Then:** exit 1; stderr contains invalid `--timeout` error; no subprocess launched
- **Exit:** 1
- **Source:** [type/09_timeout_secs.md](../../../../docs/cli/type/09_timeout_secs.md)
- **Commands:** isolated, refresh

---

### EC-6: `--timeout` without value → exit 1, requires argument

- **Given:** credentials JSON at `/tmp/ec6_to_creds.json`
- **When:** `clr isolated --creds /tmp/ec6_to_creds.json --timeout`
- **Then:** exit 1; stderr indicates `--timeout` requires a value
- **Exit:** 1
- **Source:** [020_timeout.md](../../../../docs/cli/param/020_timeout.md)
- **Commands:** isolated, refresh

---

### EC-7: timeout preserves partial stdout in error output

- **Given:** fake-claude subprocess that emits partial output then hangs; `--timeout 1`
- **When:** `clr isolated --creds /tmp/ec7_creds.json --timeout 1 "test"` (subprocess never exits)
- **Then:** exit 2; error message includes any partial stdout emitted before the timeout fired
- **Exit:** 2
- **Source:** [020_timeout.md — partial stdout note](../../../../docs/cli/param/020_timeout.md)
- **Commands:** isolated
- **Note:** Implemented in TSK-196 (BUG-243); test function `timeout_includes_partial_stdout` in `tests/bug_reproducers_239_244_test.rs`
