# Parameter :: `--quiet`

Edge case tests for the `--quiet` flag that suppresses non-fatal CLR runner diagnostics.
Fatal errors, `--dry-run` output, and `--trace` output are never suppressed.

**Source:** [074_quiet.md](../../../../docs/cli/param/074_quiet.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| QT-1 | `--quiet` suppresses gate-wait message | Behavioral |
| QT-2 | `--quiet` suppresses retry warning message | Behavioral |
| QT-3 | No `--quiet` — CLR diagnostics appear on stderr (default) | Default |
| QT-4 | `CLR_QUIET=true` env var fallback | Env Var |
| QT-5 | `--quiet` does not suppress `--dry-run` preview | Non-gate |
| QT-6 | `--quiet` + spawn failure → fatal error still on stderr | Fatal Exception |

## Test Coverage Summary

- Behavioral: 2 tests (QT-1, QT-2)
- Default: 1 test (QT-3)
- Env Var: 1 test (QT-4)
- Non-gate: 1 test (QT-5)
- Fatal Exception: 1 test (QT-6)

**Total:** 6 test cases

## Test Cases

---

### QT-1: `--quiet` suppresses gate-wait message

- **Given:** `--max-sessions 1` and one active claude session in env (or fake to trigger gate)
- **When:** `clr --print --quiet --max-sessions 1 "x"` with fake claude
- **Then:** stderr does NOT contain "Waiting" or gate-wait diagnostic text
- **Exit:** 0
- **Source:** [074_quiet.md](../../../../docs/cli/param/074_quiet.md)
- **Commands:** run, ask

---

### QT-2: `--quiet` suppresses retry warning

- **Given:** fake claude binary emitting rate-limit exit code with `--retry-on-transient 1`
- **When:** `clr --print --quiet --max-sessions 0 --retry-on-transient 1 "x"` with fake claude emitting transient error
- **Then:** stderr does NOT contain "retrying" or retry progress messages
- **Exit:** 2 (retries exhausted) or 0 (if fake claude recovers)
- **Source:** [074_quiet.md](../../../../docs/cli/param/074_quiet.md)
- **Commands:** run, ask

---

### QT-3: Default (no `--quiet`) — CLR diagnostics appear

- **Given:** `--max-sessions 0`; no `CLR_QUIET` set; clean environment
- **When:** `clr --dry-run --max-sessions 0 "Fix bug"`
- **Then:** Exit 0; no unknown-flag error; dry-run output shown; default behavior unchanged
- **Exit:** 0
- **Source:** [074_quiet.md](../../../../docs/cli/param/074_quiet.md)
- **Commands:** run, ask

---

### QT-4: `CLR_QUIET=true` env var fallback

- **Given:** `CLR_QUIET=true` set in environment; fake claude binary available
- **When:** `CLR_QUIET=true clr --print --max-sessions 0 "x"` with fake claude
- **Then:** Equivalent to `--quiet`; stderr is free of CLR diagnostic messages
- **Exit:** 0
- **Source:** [074_quiet.md](../../../../docs/cli/param/074_quiet.md)
- **Commands:** run, ask

---

### QT-5: `--quiet` does not suppress `--dry-run` preview

- **Given:** clean environment; `--quiet` + `--dry-run` combined
- **When:** `clr --quiet --dry-run "Fix bug"`
- **Then:** Exit 0; dry-run stdout contains `CLAUDE_CODE_MAX_OUTPUT_TOKENS=` (preview always shown); `--quiet` does not gate `--dry-run` output
- **Exit:** 0
- **Source:** [074_quiet.md](../../../../docs/cli/param/074_quiet.md)
- **Commands:** run, ask

---

### QT-6: `--quiet` + spawn failure → fatal error still on stderr

- **Given:** PATH set to directory with no `claude` binary; `CLR_CLAUDE_BIN` unset
- **When:** `clr --quiet "Fix bug"`
- **Then:** stderr is NOT empty; fatal spawn error emitted regardless of `--quiet`; `--quiet` suppresses diagnostics only — never fatal errors
- **Exit:** 1
- **Source:** [074_quiet.md](../../../../docs/cli/param/074_quiet.md)
- **Commands:** run
