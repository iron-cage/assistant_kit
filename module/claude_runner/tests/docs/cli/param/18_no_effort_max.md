# Parameter :: `--no-effort-max`

Edge case coverage for the `--no-effort-max` parameter. See [18_no_effort_max.md](../../../../docs/cli/param/18_no_effort_max.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--no-effort-max` → no `--effort` flag in assembled command | Behavioral Divergence |
| EC-2 | `--no-effort-max` without message → accepted, bare command has no `--effort` | Edge Case |
| EC-3 | `--no-effort-max` with `--effort medium` → effort suppressed, not forwarded | Interaction |
| EC-4 | `--help` output contains `--no-effort-max` | Documentation |
| EC-5 | Default (no `--no-effort-max`) → `--effort max` present | Behavioral Divergence |
| EC-6 | `--no-effort-max` + `--new-session` → both accepted, no conflict | Interaction |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-5)
- Edge Case: 1 test
- Interaction: 2 tests
- Documentation: 1 test

**Total:** 6 edge cases


---

### EC-1: `--no-effort-max` suppresses `--effort` entirely

- **Given:** clean environment
- **When:** `clr --dry-run --no-effort-max "Fix the bug"`
- **Then:** Assembled command does NOT contain any `--effort` token.; no `--effort` present in output
- **Exit:** 0
- **Source:** [--no-effort-max](../../../../docs/cli/param/18_no_effort_max.md), [invariant/001_default_flags.md](../../../../docs/invariant/001_default_flags.md)

---

### EC-2: `--no-effort-max` without message → accepted, no error

- **Given:** clean environment
- **When:** `clr --dry-run --no-effort-max`
- **Then:** Exit 0; assembled command has no `--effort` flag; no rejection.; clean bare command without `--effort`
- **Exit:** 0
- **Source:** [--no-effort-max](../../../../docs/cli/param/18_no_effort_max.md)

---

### EC-3: `--no-effort-max` with `--effort medium` → no effort forwarded

- **Given:** clean environment
- **When:** `clr --dry-run --no-effort-max --effort medium "Fix the bug"`
- **Then:** No `--effort` token present in assembled command.; suppression beats override; no effort forwarded
- **Exit:** 0
- **Source:** [--no-effort-max (Note: mutually exclusive)](../../../../docs/cli/param/18_no_effort_max.md)

---

### EC-4: `--help` lists `--no-effort-max`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--no-effort-max`.; flag present in help
- **Exit:** 0
- **Source:** [command.md — help](../../../../docs/cli/command.md#command--2-help)

---

### EC-5: Default (no `--no-effort-max`) → `--effort max` present

- **Given:** clean environment
- **When:** `clr --dry-run "Fix the bug"`
- **Then:** Assembled command contains `--effort max`.; default injection in effect
- **Exit:** 0
- **Source:** [invariant/001_default_flags.md](../../../../docs/invariant/001_default_flags.md)

---

### EC-6: `--no-effort-max` + `--new-session` → both accepted, no conflict

- **Given:** clean environment
- **When:** `clr --dry-run --no-effort-max --new-session "Fix the bug"`
- **Then:** Assembled command contains `--new-session` and does NOT contain `--effort`; both flags coexist without error
- **Exit:** 0
- **Source:** [--no-effort-max](../../../../docs/cli/param/18_no_effort_max.md)
