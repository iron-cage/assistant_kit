# Test: `--effort`

Edge case coverage for the `--effort` parameter. See [params.md](../../../../../docs/cli/params.md#parameter--17---effort) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-01 | Default invocation → `--effort max` present in assembled command | Default Behavior |
| TC-02 | `--effort medium` → `--effort medium` in command (not `--effort max`) | Override |
| TC-03 | `--effort high` → `--effort high` in command | Override |
| TC-04 | `--effort low` → `--effort low` in command | Override |
| TC-05 | `--effort max` explicit → `--effort max` (idempotent with default) | Idempotent |
| TC-06 | `--effort invalid` → exit 1, stderr lists valid values | Validation |
| TC-07 | `--effort` with no value → exit 1, "requires a value" error | Validation |
| TC-08 | `--help` output contains `--effort` | Documentation |

## Test Coverage Summary

- Default Behavior: 1 test
- Override: 3 tests
- Idempotent: 1 test
- Validation: 2 tests
- Documentation: 1 test

**Total:** 8 edge cases

---

### TC-01: Default invocation → `--effort max` present

**Goal:** Without `--effort`, `clr` injects `--effort max` automatically. This is the default-on behavior.
**Setup:** None.
**Command:** `clr --dry-run "Fix the bug"`
**Expected Output:** Assembled command contains `--effort max`.
**Verification:** `output.contains("--effort max")`.
**Pass Criteria:** Exit 0; `--effort max` present in assembled command.
**Source:** [params.md — --effort](../../../../../docs/cli/params.md#parameter--17---effort), [invariant/001_default_flags.md](../../../../../docs/invariant/001_default_flags.md)

---

### TC-02: `--effort medium` overrides default

**Goal:** `--effort medium` replaces the `max` default; the assembled command contains `--effort medium` and NOT `--effort max`.
**Setup:** None.
**Command:** `clr --dry-run --effort medium "Fix the bug"`
**Expected Output:** `--effort medium` present; `--effort max` absent.
**Verification:** `output.contains("--effort medium")` and `!output.contains("--effort max")`.
**Pass Criteria:** Exit 0; override applied correctly.
**Source:** [params.md — --effort](../../../../../docs/cli/params.md#parameter--17---effort)

---

### TC-03: `--effort high` override

**Goal:** `--effort high` is accepted and forwarded to claude.
**Setup:** None.
**Command:** `clr --dry-run --effort high "Fix the bug"`
**Expected Output:** `--effort high` present.
**Verification:** `output.contains("--effort high")`.
**Pass Criteria:** Exit 0; high level accepted and used.
**Source:** [params.md — --effort](../../../../../docs/cli/params.md#parameter--17---effort)

---

### TC-04: `--effort low` override

**Goal:** `--effort low` is accepted and forwarded to claude.
**Setup:** None.
**Command:** `clr --dry-run --effort low "Fix the bug"`
**Expected Output:** `--effort low` present.
**Verification:** `output.contains("--effort low")`.
**Pass Criteria:** Exit 0; low level accepted and used.
**Source:** [params.md — --effort](../../../../../docs/cli/params.md#parameter--17---effort)

---

### TC-05: `--effort max` explicit is idempotent

**Goal:** Explicitly passing `--effort max` produces the same result as the default (no double injection).
**Setup:** None.
**Command:** `clr --dry-run --effort max "Fix the bug"`
**Expected Output:** `--effort max` appears exactly once.
**Verification:** Assembled command contains exactly one `--effort max` occurrence.
**Pass Criteria:** Exit 0; exactly one `--effort max` in output.
**Source:** [params.md — --effort](../../../../../docs/cli/params.md#parameter--17---effort)

---

### TC-06: `--effort invalid` → validation error

**Goal:** Unknown effort level produces a clear error listing valid values.
**Setup:** None.
**Command:** `clr --effort bad_level "Fix the bug"`
**Expected Output:** Exit 1; stderr contains "valid values" and/or the list `low, medium, high, max`.
**Verification:** `!out.status.success()` and `stderr.contains("valid values")`.
**Pass Criteria:** Exit 1; error message references valid levels.
**Source:** [types.md — EffortLevel validation errors](../../../../../docs/cli/types.md#type--7-effortlevel)

---

### TC-07: `--effort` with no value → missing value error

**Goal:** `--effort` at end of argv without a following value is rejected with "requires a value".
**Setup:** None.
**Command:** `clr --effort`
**Expected Output:** Exit 1; stderr contains "requires a value".
**Verification:** `!out.status.success()` and `stderr.contains("requires a value")`.
**Pass Criteria:** Exit 1; error message is clear about missing value.
**Source:** [params.md — --effort (Validation)](../../../../../docs/cli/params.md#parameter--17---effort)
**Automated Test:** `effort_args_test.rs::t67_effort_missing_value_rejected`

---

### TC-08: `--help` lists `--effort`

**Goal:** `--effort` is visible in help output so users can discover the override mechanism.
**Setup:** None.
**Command:** `clr --help`
**Expected Output:** Stdout contains `--effort`.
**Verification:** `output.contains("--effort")`.
**Pass Criteria:** Exit 0; flag present in help.
**Source:** [commands.md — help](../../../../../docs/cli/commands.md#command--2-help)
