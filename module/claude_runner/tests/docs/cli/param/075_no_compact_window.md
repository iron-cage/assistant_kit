# Param :: `--no-compact-window`

Edge case tests for the `--no-compact-window` flag, which suppresses injection of
`CLAUDE_CODE_AUTO_COMPACT_WINDOW=200000` into the subprocess environment.

**Source:** [param/075_no_compact_window.md](../../../../docs/cli/param/075_no_compact_window.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Default (absent) → env var injected for `run` | Default |
| EC-2 | `--no-compact-window` present → env var absent for `run` | Behavioral |
| EC-3 | `--no-compact-window` present → env var absent for `isolated` | Behavioral |
| EC-4 | `--no-compact-window` present → env var absent for `refresh` | Behavioral |
| EC-5 | `--no-compact-window` present → env var absent for `ask` | Behavioral |
| EC-6 | `CLR_NO_COMPACT_WINDOW=1` fallback → env var absent (no CLI flag) | EnvFallback |
| EC-7 | CLI flag takes precedence over absent env var | Precedence |
| EC-8 | `--dry-run` reflects suppression accurately | Discovery |
| EC-9 | `--trace` reflects suppression on stderr | Discovery |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Behavioral: 4 tests (EC-2, EC-3, EC-4, EC-5)
- EnvFallback: 1 test (EC-6)
- Precedence: 1 test (EC-7)
- Discovery: 2 tests (EC-8, EC-9)

**Total:** 9 edge cases

## Test Cases
---

### EC-1: Default (absent) → env var injected for `run`

- **Given:** clean environment; `--no-compact-window` absent
- **When:** `clr --dry-run "test" 2>&1`
- **Then:** output contains `CLAUDE_CODE_AUTO_COMPACT_WINDOW=200000`
- **Exit:** 0
- **Source:** [param/075_no_compact_window.md](../../../../docs/cli/param/075_no_compact_window.md)

---

### EC-2: `--no-compact-window` present → env var absent for `run`

- **Given:** clean environment
- **When:** `clr --no-compact-window --dry-run "test" 2>&1`
- **Then:** output does NOT contain `CLAUDE_CODE_AUTO_COMPACT_WINDOW`
- **Exit:** 0
- **Source:** [param/075_no_compact_window.md](../../../../docs/cli/param/075_no_compact_window.md)

---

### EC-3: `--no-compact-window` present → env var absent for `isolated`

- **Given:** credentials JSON at `/tmp/075ec3.creds.json` (content `{}`)
- **When:** `clr isolated --creds /tmp/075ec3.creds.json --no-compact-window --dry-run 2>&1`
- **Then:** output does NOT contain `CLAUDE_CODE_AUTO_COMPACT_WINDOW`
- **Exit:** 0
- **Source:** [param/075_no_compact_window.md](../../../../docs/cli/param/075_no_compact_window.md), [command/02_isolated.md](../../../../docs/cli/command/02_isolated.md)

---

### EC-4: `--no-compact-window` present → env var absent for `refresh`

- **Given:** credentials JSON at `/tmp/075ec4.creds.json` (content `{}`)
- **When:** `clr refresh --creds /tmp/075ec4.creds.json --no-compact-window --dry-run 2>&1`
- **Then:** output does NOT contain `CLAUDE_CODE_AUTO_COMPACT_WINDOW`
- **Exit:** 0
- **Source:** [param/075_no_compact_window.md](../../../../docs/cli/param/075_no_compact_window.md), [command/03_refresh.md](../../../../docs/cli/command/03_refresh.md)

---

### EC-5: `--no-compact-window` present → env var absent for `ask`

- **Given:** clean environment
- **When:** `clr ask --no-compact-window --dry-run "test" 2>&1`
- **Then:** output does NOT contain `CLAUDE_CODE_AUTO_COMPACT_WINDOW`
- **Exit:** 0
- **Source:** [param/075_no_compact_window.md](../../../../docs/cli/param/075_no_compact_window.md)

---

### EC-6: `CLR_NO_COMPACT_WINDOW=1` fallback suppresses env var

- **Given:** `CLR_NO_COMPACT_WINDOW=1` in environment; `--no-compact-window` absent from CLI
- **When:** `CLR_NO_COMPACT_WINDOW=1 clr --dry-run "test" 2>&1`
- **Then:** output does NOT contain `CLAUDE_CODE_AUTO_COMPACT_WINDOW`; identical to passing `--no-compact-window`
- **Exit:** 0
- **Source:** [param/075_no_compact_window.md](../../../../docs/cli/param/075_no_compact_window.md), [env_param.md](../../../../docs/cli/env_param.md)

---

### EC-7: CLI flag absence with clean env → injection active

- **Given:** clean environment, no `CLR_NO_COMPACT_WINDOW`
- **When:** `clr --dry-run "test" 2>&1`
- **Then:** output CONTAINS `CLAUDE_CODE_AUTO_COMPACT_WINDOW=200000` — confirms opt-in default
- **Exit:** 0
- **Source:** [param/075_no_compact_window.md](../../../../docs/cli/param/075_no_compact_window.md)

---

### EC-8: `--dry-run` reflects suppression accurately (WYSIWYG)

- **Given:** clean environment
- **When (default):** `clr --dry-run "test" 2>&1` → var present
- **When (opt-out):** `clr --no-compact-window --dry-run "test" 2>&1` → var absent
- **Then:** dry-run output matches exactly what subprocess would receive; no misleading env entries
- **Exit:** 0 both cases
- **Source:** [param/075_no_compact_window.md](../../../../docs/cli/param/075_no_compact_window.md), [param/011_dry_run.md](../../../../docs/cli/param/011_dry_run.md)

---

### EC-9: `--trace` reflects suppression on stderr

- **Given:** claude not in PATH; `--no-compact-window` present
- **When:** `clr --no-compact-window --trace "test" 2>&1`
- **Then:** stderr does NOT contain `CLAUDE_CODE_AUTO_COMPACT_WINDOW`; subprocess attempt fails after trace
- **Exit:** 1 (claude absent)
- **Source:** [param/075_no_compact_window.md](../../../../docs/cli/param/075_no_compact_window.md), [param/013_trace.md](../../../../docs/cli/param/013_trace.md)
