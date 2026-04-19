# Test: `interval::`

Edge case coverage for the `interval::` parameter. See [params.md](../../../../../docs/cli/params.md#parameter--9-interval) and [types.md](../../../../../docs/cli/types.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `interval::0` behaves as one-shot (default) | Default Behavior |
| EC-2 | Default (omitted) resolves to `interval::0` | Default Behavior |
| EC-3 | `interval::` (empty) rejected with exit 1 and message | Empty Value |
| EC-4 | `interval::-1` rejected — negative values not accepted | Invalid Value |
| EC-5 | `interval::abc` rejected — non-integer not accepted | Format Violation |
| EC-6 | `interval::5` starts watch mode (process does not exit immediately) | Watch Mode |
| EC-7 | `interval::0` with `dry::1` combines correctly | Interaction |
| EC-8 | `interval::` only accepted by `.version.guard` | Command Scope |

## Test Coverage Summary

- Default Behavior: 2 tests
- Empty Value: 1 test
- Invalid Value: 1 test
- Format Violation: 1 test
- Watch Mode: 1 test
- Interaction: 1 test
- Command Scope: 1 test

**Total:** 8 edge cases

---

### EC-1: `interval::0` behaves as one-shot (default)

**Goal:** Explicit `interval::0` produces identical behavior to omitting the parameter.
**Setup:** No preferred version set.
**Command:** `cm .version.guard interval::0`
**Expected Output:** output contains "stable" (defaults to stable); process exits immediately.
**Verification:**
- exit code 0
- process exits (does not loop)
- output identical to `cm .version.guard`
**Pass Criteria:** Exit 0; one-shot behavior; immediate exit.
**Source:** [params.md — interval:: default: 0](../../../../../docs/cli/params.md#parameter--9-interval)

---

### EC-2: Default (omitted) resolves to `interval::0`

**Goal:** Omitting `interval::` defaults to one-shot mode.
**Setup:** No preferred version set.
**Command:** `cm .version.guard`
**Expected Output:** output contains "stable" (defaults to stable); process exits immediately.
**Verification:**
- exit code 0
- process exits (does not loop)
- behavior identical to `interval::0`
**Pass Criteria:** Exit 0; one-shot mode; same as explicit `interval::0`.
**Source:** [params.md — interval:: default](../../../../../docs/cli/params.md#parameter--9-interval)

---

### EC-3: `interval::` (empty) rejected with exit 1 and message

**Goal:** Empty integer value is a usage error.
**Setup:** None.
**Command:** `cm .version.guard interval::`
**Expected Output:** stderr: error about `interval::` requiring a value; exit code 1.
**Verification:**
- exit code equals 1
- error mentions `interval::`
**Pass Criteria:** Exit 1; usage error.
**Source:** [params.md — interval:: constraints](../../../../../docs/cli/params.md#parameter--9-interval)

---

### EC-4: `interval::-1` rejected — negative values not accepted

**Goal:** Negative values are rejected for an unsigned integer parameter.
**Setup:** None.
**Command:** `cm .version.guard interval::-1`
**Expected Output:** stderr: error about invalid `interval::` value; exit code 1.
**Verification:**
- exit code equals 1
- error message mentions valid range or invalid value
**Pass Criteria:** Exit 1; negative value rejected.
**Source:** [params.md — interval:: type: u64](../../../../../docs/cli/params.md#parameter--9-interval)

---

### EC-5: `interval::abc` rejected — non-integer not accepted

**Goal:** Non-numeric strings are rejected.
**Setup:** None.
**Command:** `cm .version.guard interval::abc`
**Expected Output:** stderr: error about invalid `interval::` format; exit code 1.
**Verification:**
- exit code equals 1
- no guard check occurs
**Pass Criteria:** Exit 1; string value rejected.
**Source:** [params.md — interval:: type: u64](../../../../../docs/cli/params.md#parameter--9-interval)

---

### EC-6: `interval::5` starts watch mode (process does not exit immediately)

**Goal:** Non-zero interval starts a looping watch process.
**Setup:** No preferred version set.
**Command:** `timeout 3 cm .version.guard interval::5`
**Expected Output:** Process produces at least one status line; continues running until killed by `timeout`.
**Verification:**
- process does not exit within 3 seconds on its own
- `timeout` terminates the process (exit code 124 from `timeout`)
- at least one status line appears in output
**Pass Criteria:** Process stays alive; watch mode active.
**Source:** [params.md — interval:: watch mode](../../../../../docs/cli/params.md#parameter--9-interval)

---

### EC-7: `interval::0` with `dry::1` combines correctly

**Goal:** One-shot mode works correctly with dry-run.
**Setup:** Preferred version set in settings.
**Command:** `cm .version.guard interval::0 dry::1`
**Expected Output:** `[dry-run]` prefixed output; process exits immediately.
**Verification:**
- exit code 0
- output contains `[dry-run]`
- process exits (does not loop)
**Pass Criteria:** Exit 0; dry-run markers present; one-shot exit.
**Source:** [parameter_interactions.md — dry+interval](../../../../../docs/cli/parameter_interactions.md)

---

### EC-8: `interval::` only accepted by `.version.guard`

**Goal:** Commands that do not declare `interval::` reject it.
**Setup:** None.
**Command:** `cm .version.install interval::5`
**Expected Output:** stderr: error about unknown parameter `interval::`; exit code 1.
**Verification:**
- exit code equals 1
- error mentions `interval::` as unrecognized
**Pass Criteria:** Exit 1; parameter rejected by non-guard command.
**Source:** [commands.md — .version.install parameters](../../../../../docs/cli/commands.md#command--5-versioninstall)
