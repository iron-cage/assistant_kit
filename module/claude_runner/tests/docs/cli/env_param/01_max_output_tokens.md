# Env Param :: `CLAUDE_CODE_MAX_OUTPUT_TOKENS`

Edge case tests for the `CLAUDE_CODE_MAX_OUTPUT_TOKENS` environment variable injection mechanism.

**Source:** [env_param.md](../../../../docs/cli/env_param.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--max-tokens 50000` → env var value matches | Behavioral |
| EC-2 | No `--max-tokens` → default 200000 injected | Default |
| EC-3 | `--max-tokens 0` → env var is `0` | Boundary |
| EC-4 | `--max-tokens 4294967295` → maximum u32 accepted | Boundary |
| EC-5 | `--dry-run` shows env var in output | Discovery |
| EC-6 | `--trace` shows env var on stderr | Discovery |

## Test Coverage Summary

- Behavioral: 1 test (EC-1)
- Default: 1 test (EC-2)
- Boundary: 2 tests (EC-3, EC-4)
- Discovery: 2 tests (EC-5, EC-6)

**Total:** 6 edge cases

## Test Cases
---

### EC-1: `--max-tokens 50000` → env var value matches

- **Given:** clean environment
- **When:** `clr --dry-run --max-tokens 50000 "test"`
- **Then:** Output contains `CLAUDE_CODE_MAX_OUTPUT_TOKENS=50000`
- **Exit:** 0
- **Source:** [env_param.md](../../../../docs/cli/env_param.md)

---

### EC-2: No `--max-tokens` → default 200000 injected

- **Given:** clean environment
- **When:** `clr --dry-run "test"`
- **Then:** Output contains `CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000`
- **Exit:** 0
- **Source:** [env_param.md](../../../../docs/cli/env_param.md)

---

### EC-3: `--max-tokens 0` → env var is `0`

- **Given:** clean environment
- **When:** `clr --dry-run --max-tokens 0 "test"`
- **Then:** Output contains `CLAUDE_CODE_MAX_OUTPUT_TOKENS=0`
- **Exit:** 0
- **Source:** [env_param.md](../../../../docs/cli/env_param.md)

---

### EC-4: `--max-tokens 4294967295` → maximum u32 accepted

- **Given:** clean environment
- **When:** `clr --dry-run --max-tokens 4294967295 "test"`
- **Then:** Output contains `CLAUDE_CODE_MAX_OUTPUT_TOKENS=4294967295`; no overflow error
- **Exit:** 0
- **Source:** [env_param.md](../../../../docs/cli/env_param.md), [param/09_max_tokens.md](../../../../docs/cli/param/09_max_tokens.md)

---

### EC-5: `--dry-run` shows env var in output

- **Given:** clean environment
- **When:** `clr --dry-run "test"`
- **Then:** `CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000` appears in the env section before the command line
- **Exit:** 0
- **Source:** [env_param.md](../../../../docs/cli/env_param.md), [param/11_dry_run.md](../../../../docs/cli/param/11_dry_run.md)

---

### EC-6: `--trace` shows env var on stderr

- **Given:** clean environment
- **When:** `clr --trace "test"` (claude unavailable in test environment)
- **Then:** Stderr contains `CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000` before the command line
- **Exit:** 1 (claude absent)
- **Source:** [env_param.md](../../../../docs/cli/env_param.md), [param/13_trace.md](../../../../docs/cli/param/13_trace.md)
