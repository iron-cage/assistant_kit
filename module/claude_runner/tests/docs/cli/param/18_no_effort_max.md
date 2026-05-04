# Test: `--no-effort-max`

Edge case coverage for the `--no-effort-max` parameter. See [params.md](../../../../docs/cli/params.md#parameter--18---no-effort-max) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--no-effort-max` → no `--effort` flag in assembled command | Suppression |
| EC-2 | `--no-effort-max` without message → accepted, bare command has no `--effort` | Edge Case |
| EC-3 | `--no-effort-max` with `--effort medium` → effort suppressed, not forwarded | Interaction |
| EC-4 | `--help` output contains `--no-effort-max` | Documentation |
| EC-5 | Default (no `--no-effort-max`) → `--effort max` present | Default Behavior |
| EC-6 | `--no-effort-max` + `--new-session` → both accepted, no conflict | Interaction |

## Test Coverage Summary

- Suppression: 1 test
- Edge Case: 1 test
- Interaction: 2 tests
- Documentation: 1 test
- Default Behavior: 1 test

**Total:** 6 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

---

### EC-1: `--no-effort-max` suppresses `--effort` entirely

- **Given:** clean environment
- **When:** `clr --dry-run --no-effort-max "Fix the bug"`
- **Then:** Assembled command does NOT contain any `--effort` token.; no `--effort` present in output
- **Exit:** 0
- **Source:** [params.md — --no-effort-max](../../../../docs/cli/params.md#parameter--18---no-effort-max), [invariant/001_default_flags.md](../../../../docs/invariant/001_default_flags.md)

---

### EC-2: `--no-effort-max` without message → accepted, no error

- **Given:** clean environment
- **When:** `clr --dry-run --no-effort-max`
- **Then:** Exit 0; assembled command has no `--effort` flag; no rejection.; clean bare command without `--effort`
- **Exit:** 0
- **Source:** [params.md — --no-effort-max](../../../../docs/cli/params.md#parameter--18---no-effort-max)

---

### EC-3: `--no-effort-max` with `--effort medium` → no effort forwarded

- **Given:** clean environment
- **When:** `clr --dry-run --no-effort-max --effort medium "Fix the bug"`
- **Then:** No `--effort` token present in assembled command.; suppression beats override; no effort forwarded
- **Exit:** 0
- **Source:** [params.md — --no-effort-max (Note: mutually exclusive)](../../../../docs/cli/params.md#parameter--18---no-effort-max)
**Automated Test:** `effort_args_test.rs::t68_no_effort_max_suppresses_explicit_effort`

---

### EC-4: `--help` lists `--no-effort-max`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--no-effort-max`.; flag present in help
- **Exit:** 0
- **Source:** [commands.md — help](../../../../docs/cli/commands.md#command--2-help)

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
- **Source:** [params.md — --no-effort-max](../../../../docs/cli/params.md#parameter--18---no-effort-max)
