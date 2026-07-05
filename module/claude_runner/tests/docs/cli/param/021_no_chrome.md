# Parameter :: `--no-chrome`

Edge case coverage for the `--no-chrome` parameter. See [021_no_chrome.md](../../../../docs/cli/param/021_no_chrome.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--no-chrome` → no `--chrome` flag in assembled command | Behavioral Divergence |
| EC-2 | Default (no `--no-chrome`) → `--chrome` present in interactive mode | Behavioral Divergence |
| EC-2b | Print mode → `--chrome` suppressed automatically (BUG-304 mitigation) | Behavioral Divergence |
| EC-3 | `--no-chrome` without message → accepted, no error | Edge Case |
| EC-4 | `--help` output contains `--no-chrome` | Documentation |
| EC-5 | `--no-chrome` + `--no-skip-permissions` → both accepted, no conflict | Interaction |
| EC-6 | `--no-chrome` + `--dry-run` → preview shows no `--chrome` | Interaction |

## Test Coverage Summary

- Behavioral Divergence: 3 tests (EC-1, EC-2, EC-2b)
- Edge Case: 1 test
- Interaction: 2 tests
- Documentation: 1 test

**Total:** 7 edge cases

---

### EC-1: `--no-chrome` suppresses `--chrome`

- **Given:** clean environment
- **When:** `clr --dry-run --no-chrome "Fix the bug"`
- **Then:** Assembled command does NOT contain `--chrome`
- **Exit:** 0
- **Source:** [--no-chrome](../../../../docs/cli/param/021_no_chrome.md)
- **Commands:** run, ask

---

### EC-2: Default → `--chrome` injected in interactive mode

- **Given:** clean environment
- **When:** `clr --dry-run` (no message — interactive mode)
- **Then:** Assembled command contains `--chrome`; default for interactive sessions
- **Exit:** 0
- **Source:** [invariant/001_default_flags.md](../../../../docs/invariant/001_default_flags.md)
- **Commands:** run, ask

---

### EC-2b: Print mode → `--chrome` suppressed automatically

- **Given:** clean environment
- **When:** `clr --dry-run "Fix the bug"` (message present → print mode)
- **Then:** Assembled command does NOT contain `--chrome`; suppressed to prevent BUG-304 hang
- **Exit:** 0
- **Source:** `src/cli/builder.rs` Fix(BUG-304)
- **Commands:** run, ask

---

### EC-3: `--no-chrome` without message → accepted, no error

- **Given:** clean environment
- **When:** `clr --dry-run --no-chrome`
- **Then:** Exit 0; assembled command has no `--chrome` flag
- **Exit:** 0
- **Source:** [--no-chrome](../../../../docs/cli/param/021_no_chrome.md)
- **Commands:** run, ask

---

### EC-4: `--help` lists `--no-chrome`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--no-chrome`
- **Exit:** 0
- **Source:** [command/02_help.md](../../../../docs/cli/command/02_help.md)
- **Commands:** run, ask

---

### EC-5: `--no-chrome` + `--no-skip-permissions` → both accepted

- **Given:** clean environment
- **When:** `clr --dry-run --no-chrome --no-skip-permissions "Fix the bug"`
- **Then:** Assembled command contains neither `--chrome` nor `--dangerously-skip-permissions`
- **Exit:** 0
- **Source:** [--no-chrome](../../../../docs/cli/param/021_no_chrome.md)
- **Commands:** run, ask

---

### EC-6: `--no-chrome` + `--dry-run` → preview without `--chrome`

- **Given:** clean environment
- **When:** `clr --dry-run --no-chrome "Fix the bug"`
- **Then:** Stdout contains assembled command without `--chrome`; stderr empty
- **Exit:** 0
- **Source:** [--no-chrome](../../../../docs/cli/param/021_no_chrome.md)
- **Commands:** run, ask
