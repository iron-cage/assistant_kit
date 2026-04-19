# Test: `force::`

Edge case coverage for the `force::` parameter. See [params.md](../../../../../docs/cli/params.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-303 | `dry::1 force::1` → dry wins, no install | Interaction (dry wins) |
| TC-312 | `dry::1 force::1` on `.processes.kill` → dry wins | Interaction (dry wins) |
| TC-406 | `.version.guard force::1 dry::1` → dry wins | Interaction (dry wins) |
| IT-6 | `force::1` on `.version.guard` → reinstalls despite match | Explicit True |
| EC-1 | Default (absent) → `force::0` (guard active) | Default Behavior |
| EC-2 | `force::0` explicit → same as absent | Explicit False |
| EC-3 | `force::2` → exit 1, out of range | Invalid Value |
| EC-4 | `force::-1` → exit 1, out of range | Invalid Value |
| EC-5 | `force::abc` → exit 1, non-integer | Format Violation |
| EC-6 | `force::` (empty) → exit 1 | Empty Value |
| EC-7 | `force::` only for `.version.install`, `.version.guard`, `.processes.kill` | Command Scope |

## Test Coverage Summary

- Interaction (dry wins): 3 tests
- Explicit True: 1 test
- Default Behavior: 1 test
- Explicit False: 1 test
- Invalid Value: 2 tests
- Format Violation: 1 test
- Empty Value: 1 test
- Command Scope: 1 test

**Total:** 12 edge cases

---

### TC-303: `dry::1 force::1` → dry wins

**Goal:** `dry::1` takes absolute precedence over `force::1`.
**Setup:** None.
**Command:** `cm .version.install dry::1 force::1`
**Expected Output:** `[dry-run]` prefix; no install.
**Pass Criteria:** Exit 0; preview only.
**Source:** [parameter_interactions.md — dry+force](../../../../../docs/cli/parameter_interactions.md)

---

### IT-6: `force::1` bypasses match check

**Goal:** When `force::1` is set, `.version.guard` reinstalls even when version matches.
**Setup:** Installed version matches `preferredVersionResolved`.
**Command:** `cm .version.guard force::1`
**Expected Output:** Install proceeds; no "matches" skip message.
**Pass Criteria:** Exit 0; reinstall occurs.
**Source:** [feature/001_version_management.md](../../../../../docs/feature/001_version_management.md)

---

### EC-3: `force::2` → exit 1

**Goal:** Boolean parameters only accept 0 or 1.
**Setup:** None.
**Command:** `cm .version.install force::2`
**Expected Output:** exit code 1.
**Pass Criteria:** Exit 1.
**Source:** [params.md — force:: type: Boolean (0/1)](../../../../../docs/cli/params.md)

---

### EC-4: `force::-1` → exit 1

**Goal:** Negative values are rejected.
**Setup:** None.
**Command:** `cm .version.install force::-1`
**Expected Output:** exit code 1.
**Pass Criteria:** Exit 1.
**Source:** [params.md — force:: type: Boolean (0/1)](../../../../../docs/cli/params.md)

---

### EC-5: `force::abc` → exit 1

**Goal:** Non-integer strings rejected.
**Setup:** None.
**Command:** `cm .version.install force::abc`
**Expected Output:** exit code 1.
**Pass Criteria:** Exit 1.
**Source:** [params.md — force:: type: Boolean (0/1)](../../../../../docs/cli/params.md)

---

### EC-6: `force::` (empty) → exit 1

**Goal:** Empty boolean value is a usage error.
**Setup:** None.
**Command:** `cm .version.install force::`
**Expected Output:** exit code 1.
**Pass Criteria:** Exit 1.
**Source:** [feature/005_cli_design.md](../../../../../docs/feature/005_cli_design.md)

---

### EC-7: `force::` only for its declared commands

**Goal:** Commands without `force::` reject it as unknown.
**Setup:** None.
**Command:** `cm .settings.set key::k value::v force::1`
**Expected Output:** exit code 1; unknown parameter.
**Pass Criteria:** Exit 1.
**Source:** [feature/005_cli_design.md](../../../../../docs/feature/005_cli_design.md)
