# Test: `--no-effort-max`

Edge case coverage for the `--no-effort-max` parameter. See [params.md](../../../../../docs/cli/params.md#parameter--18---no-effort-max) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-01 | `--no-effort-max` → no `--effort` flag in assembled command | Suppression |
| TC-02 | `--no-effort-max` without message → accepted, bare command has no `--effort` | Edge Case |
| TC-03 | `--no-effort-max` with `--effort medium` → effort suppressed, not forwarded | Interaction |
| TC-04 | `--help` output contains `--no-effort-max` | Documentation |
| TC-05 | Default (no `--no-effort-max`) → `--effort max` present | Default Behavior |

## Test Coverage Summary

- Suppression: 1 test
- Edge Case: 1 test
- Interaction: 1 test
- Documentation: 1 test
- Default Behavior: 1 test

**Total:** 5 edge cases

---

### TC-01: `--no-effort-max` suppresses `--effort` entirely

**Goal:** `--no-effort-max` prevents any `--effort` flag from appearing in the assembled command.
**Setup:** None.
**Command:** `clr --dry-run --no-effort-max "Fix the bug"`
**Expected Output:** Assembled command does NOT contain any `--effort` token.
**Verification:** `!output.contains("--effort")`.
**Pass Criteria:** Exit 0; no `--effort` present in output.
**Source:** [params.md — --no-effort-max](../../../../../docs/cli/params.md#parameter--18---no-effort-max), [invariant/001_default_flags.md](../../../../../docs/invariant/001_default_flags.md)

---

### TC-02: `--no-effort-max` without message → accepted, no error

**Goal:** `--no-effort-max` with no message is valid; bare `clr` still enters interactive REPL without error.
**Setup:** None.
**Command:** `clr --dry-run --no-effort-max`
**Expected Output:** Exit 0; assembled command has no `--effort` flag; no rejection.
**Verification:** `out.status.success()` and `!output.contains("--effort")`.
**Pass Criteria:** Exit 0; clean bare command without `--effort`.
**Source:** [params.md — --no-effort-max](../../../../../docs/cli/params.md#parameter--18---no-effort-max)

---

### TC-03: `--no-effort-max` with `--effort medium` → no effort forwarded

**Goal:** When `--no-effort-max` is combined with `--effort medium`, the suppression wins and no `--effort` is forwarded.
**Setup:** None.
**Command:** `clr --dry-run --no-effort-max --effort medium "Fix the bug"`
**Expected Output:** No `--effort` token present in assembled command.
**Verification:** `!output.contains("--effort")`.
**Pass Criteria:** Exit 0; suppression beats override; no effort forwarded.
**Source:** [params.md — --no-effort-max (Note: mutually exclusive)](../../../../../docs/cli/params.md#parameter--18---no-effort-max)
**Automated Test:** `effort_args_test.rs::t68_no_effort_max_suppresses_explicit_effort`

---

### TC-04: `--help` lists `--no-effort-max`

**Goal:** `--no-effort-max` is visible in help output so users can discover how to suppress the default.
**Setup:** None.
**Command:** `clr --help`
**Expected Output:** Stdout contains `--no-effort-max`.
**Verification:** `output.contains("--no-effort-max")`.
**Pass Criteria:** Exit 0; flag present in help.
**Source:** [commands.md — help](../../../../../docs/cli/commands.md#command--2-help)

---

### TC-05: Default (no `--no-effort-max`) → `--effort max` present

**Goal:** Without `--no-effort-max`, the default injection of `--effort max` occurs normally.
**Setup:** None.
**Command:** `clr --dry-run "Fix the bug"`
**Expected Output:** Assembled command contains `--effort max`.
**Verification:** `output.contains("--effort max")`.
**Pass Criteria:** Exit 0; default injection in effect.
**Source:** [invariant/001_default_flags.md](../../../../../docs/invariant/001_default_flags.md)
