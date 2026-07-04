# Env Param :: `CLAUDE_CODE_AUTO_COMPACT_WINDOW`

Edge case tests for the `CLAUDE_CODE_AUTO_COMPACT_WINDOW` environment variable injection
mechanism and its `--no-compact-window` / `CLR_NO_COMPACT_WINDOW` opt-out.

**Source:** [env_param.md](../../../../docs/cli/env_param.md), [param/075_no_compact_window.md](../../../../docs/cli/param/075_no_compact_window.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Default injection — `CLAUDE_CODE_AUTO_COMPACT_WINDOW=300000` present without flag | Default |
| EC-2 | `--no-compact-window` suppresses injection entirely | Opt-Out |
| EC-3 | `CLR_NO_COMPACT_WINDOW=1` env var fallback suppresses injection | EnvFallback |
| EC-4 | `CLR_NO_COMPACT_WINDOW=true` also accepted | EnvFallback |
| EC-5 | `--dry-run` reflects value (present or absent) accurately | Discovery |
| EC-6 | `--trace` reflects value on stderr before subprocess attempt | Discovery |
| EC-7 | Injection applies to `isolated` command by default | CrossCommand |
| EC-8 | Injection applies to `refresh` command by default | CrossCommand |
| EC-9 | `CLR_NO_COMPACT_WINDOW=0` does NOT suppress (only `1`/`true` activate) | Boundary |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Opt-Out: 1 test (EC-2)
- EnvFallback: 2 tests (EC-3, EC-4)
- Discovery: 2 tests (EC-5, EC-6)
- CrossCommand: 2 tests (EC-7, EC-8)
- Boundary: 1 test (EC-9)

**Total:** 9 edge cases

## Test Cases
---

### EC-1: Default injection — `CLAUDE_CODE_AUTO_COMPACT_WINDOW=300000` present without flag

- **Given:** clean environment, no `CLR_NO_COMPACT_WINDOW`
- **When:** `clr --dry-run "test" 2>&1`
- **Then:** output contains `CLAUDE_CODE_AUTO_COMPACT_WINDOW=300000`
- **Exit:** 0
- **Source:** [env_param.md](../../../../docs/cli/env_param.md)

---

### EC-2: `--no-compact-window` suppresses injection entirely

- **Given:** clean environment
- **When:** `clr --no-compact-window --dry-run "test" 2>&1`
- **Then:** output does NOT contain `CLAUDE_CODE_AUTO_COMPACT_WINDOW`
- **Exit:** 0
- **Source:** [env_param.md](../../../../docs/cli/env_param.md), [param/075_no_compact_window.md](../../../../docs/cli/param/075_no_compact_window.md)

---

### EC-3: `CLR_NO_COMPACT_WINDOW=1` env var fallback suppresses injection

- **Given:** `CLR_NO_COMPACT_WINDOW=1` in environment; `--no-compact-window` absent from CLI
- **When:** `CLR_NO_COMPACT_WINDOW=1 clr --dry-run "test" 2>&1`
- **Then:** output does NOT contain `CLAUDE_CODE_AUTO_COMPACT_WINDOW`; env var applied as fallback
- **Exit:** 0
- **Source:** [env_param.md](../../../../docs/cli/env_param.md)

---

### EC-4: `CLR_NO_COMPACT_WINDOW=true` also accepted

- **Given:** `CLR_NO_COMPACT_WINDOW=true` in environment
- **When:** `CLR_NO_COMPACT_WINDOW=true clr --dry-run "test" 2>&1`
- **Then:** output does NOT contain `CLAUDE_CODE_AUTO_COMPACT_WINDOW`
- **Exit:** 0
- **Source:** [env_param.md](../../../../docs/cli/env_param.md)

---

### EC-5: `--dry-run` reflects presence or absence accurately

- **Given:** clean environment
- **When (with):** `clr --dry-run "test" 2>&1` → `CLAUDE_CODE_AUTO_COMPACT_WINDOW=300000` visible
- **When (without):** `clr --no-compact-window --dry-run "test" 2>&1` → var absent
- **Then:** dry-run output WYSIWYG matches what would be passed to subprocess; no phantom entries
- **Exit:** 0 both cases
- **Source:** [env_param.md](../../../../docs/cli/env_param.md), [param/011_dry_run.md](../../../../docs/cli/param/011_dry_run.md)

---

### EC-6: `--trace` reflects value on stderr before subprocess attempt

- **Given:** clean environment; claude not in PATH
- **When:** `clr --trace "test" 2>&1`
- **Then:** stderr contains `CLAUDE_CODE_AUTO_COMPACT_WINDOW=300000` before command line; subprocess attempt fails
- **Exit:** 1 (claude absent)
- **Source:** [env_param.md](../../../../docs/cli/env_param.md), [param/013_trace.md](../../../../docs/cli/param/013_trace.md)

---

### EC-7: Injection applies to `isolated` command by default

- **Given:** credentials JSON at `/tmp/ec7.creds.json` (content `{}`); no `CLR_NO_COMPACT_WINDOW`
- **When:** `clr isolated --creds /tmp/ec7.creds.json --dry-run 2>&1`
- **Then:** output contains `CLAUDE_CODE_AUTO_COMPACT_WINDOW=300000`
- **Exit:** 0
- **Source:** [env_param.md](../../../../docs/cli/env_param.md)

---

### EC-8: Injection applies to `refresh` command by default

- **Given:** credentials JSON at `/tmp/ec8.creds.json` (content `{}`); no `CLR_NO_COMPACT_WINDOW`
- **When:** `clr refresh --creds /tmp/ec8.creds.json --dry-run 2>&1`
- **Then:** output contains `CLAUDE_CODE_AUTO_COMPACT_WINDOW=300000`
- **Exit:** 0
- **Source:** [env_param.md](../../../../docs/cli/env_param.md)

---

### EC-9: `CLR_NO_COMPACT_WINDOW=0` does NOT suppress

- **Given:** `CLR_NO_COMPACT_WINDOW=0` in environment (falsy value)
- **When:** `CLR_NO_COMPACT_WINDOW=0 clr --dry-run "test" 2>&1`
- **Then:** output CONTAINS `CLAUDE_CODE_AUTO_COMPACT_WINDOW=300000` — `"0"` is not a truthy value for this bool env var
- **Exit:** 0
- **Source:** [env_param.md](../../../../docs/cli/env_param.md)
