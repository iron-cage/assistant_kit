# Parameter :: `agent::`

Edge case tests for the `agent::` parameter. Tests validate boolean enforcement, auto-enable behavior, and unset semantics.

**Source:** [params.md#parameter--1-agent](../../params.md#parameter--1-agent)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Value 0 accepted (main sessions only) | Boundary Values |
| EC-2 | Value 1 accepted (agent sessions only) | Boundary Values |
| EC-3 | Value 2 rejected | Boundary Values |
| EC-4 | String "yes" rejected | Type Validation |
| EC-5 | Unset returns all session types | Default |
| EC-6 | agent::1 auto-enables sessions display in .list | Auto-Enable |
| EC-7 | agent::0 auto-enables sessions display in .list | Auto-Enable |

## Test Coverage Summary

- Boundary Values: 3 tests (EC-1, EC-2, EC-3)
- Type Validation: 1 test (EC-4)
- Default: 1 test (EC-5)
- Auto-Enable: 2 tests (EC-6, EC-7)

## Test Cases

### EC-1: Value 0 accepted (main sessions only)

**Goal:** Verify that `agent::0` is accepted and filters to main sessions only (excludes agent/sidechain sessions).
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .list agent::0`
**Expected Output:** List of sessions where none are agent sessions (`agent-*.jsonl` files excluded).
**Verification:**
- Command exits with code 0
- No error message appears on stderr
- No session names beginning with `agent-` appear in output
**Pass Criteria:** exit 0 + only main sessions listed (no `agent-` prefixed sessions)
**Source:** [params.md](../../params.md)

---

### EC-2: Value 1 accepted (agent sessions only)

**Goal:** Verify that `agent::1` is accepted and filters to agent sessions only.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .list agent::1`
**Expected Output:** List of only agent sessions (`agent-*.jsonl` files) if any exist; empty list otherwise.
**Verification:**
- Command exits with code 0
- No error message appears on stderr
- All listed sessions are agent-type sessions (prefixed `agent-` or marked `isSidechain: true`)
**Pass Criteria:** exit 0 + only agent sessions listed (or empty list if none exist in fixture)
**Source:** [params.md](../../params.md)

---

### EC-3: Value 2 rejected

**Goal:** Verify that `agent::2` is rejected with the boolean constraint error message.
**Setup:** None
**Command:** `clg .list agent::2`
**Expected Output:** `agent must be 0 or 1`
**Verification:**
- Command exits with code 1
- Stderr contains the string `agent must be 0 or 1`
**Pass Criteria:** exit 1 + error message `agent must be 0 or 1`
**Source:** [params.md](../../params.md)

---

### EC-4: String "yes" rejected

**Goal:** Verify that the string `yes` is rejected as a non-boolean value for `agent::`.
**Setup:** None
**Command:** `clg .list agent::yes`
**Expected Output:** `agent must be 0 or 1`
**Verification:**
- Command exits with code 1
- Stderr contains the string `agent must be 0 or 1`
**Pass Criteria:** exit 1 + error message `agent must be 0 or 1`
**Source:** [params.md](../../params.md)

---

### EC-5: Unset returns all session types

**Goal:** Verify that omitting `agent::` returns both main and agent sessions with no filtering.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .list`
**Expected Output:** All sessions regardless of type — both main sessions and agent sessions appear.
**Verification:**
- Command exits with code 0
- Output includes sessions that would be excluded by `agent::0` (i.e., agent sessions if present)
- Output includes sessions that would be excluded by `agent::1` (i.e., main sessions)
**Pass Criteria:** exit 0 + result set is superset of both `agent::0` and `agent::1` result sets
**Source:** [params.md](../../params.md)

---

### EC-6: agent::1 auto-enables sessions display in .list

**Goal:** Verify that `agent::1` in `.list` triggers session display even without explicit `sessions::1`.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .list agent::1`
**Expected Output:** Project list with session-level detail shown under each project (sessions auto-displayed).
**Verification:**
- Command exits with code 0
- Output includes session entries under projects (not just project-level rows)
- Sessions shown are only agent-type sessions
**Pass Criteria:** exit 0 + sessions section visible in output (auto-enabled by `agent::1`)
**Source:** [params.md](../../params.md)

---

### EC-7: agent::0 auto-enables sessions display in .list

**Goal:** Verify that `agent::0` in `.list` also triggers session display (same auto-enable behavior as `agent::1`).
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .list agent::0`
**Expected Output:** Project list with session-level detail shown under each project; only main sessions displayed.
**Verification:**
- Command exits with code 0
- Output includes session entries under projects (sessions auto-displayed)
- Sessions shown are only main sessions (no `agent-` prefixed entries)
**Pass Criteria:** exit 0 + sessions section visible in output (auto-enabled by `agent::0`)
**Source:** [params.md](../../params.md)
