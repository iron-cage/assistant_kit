# Parameter :: `agent::`

Edge case tests for the `agent::` parameter. Tests validate boolean enforcement, auto-enable behavior, and unset semantics.

**Source:** [params.md#parameter--1-agent](../../../../docs/cli/params.md#parameter--1-agent)

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

**Total:** 7 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

## Test Cases

---

### EC-1: Value 0 accepted (main sessions only)

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list agent::0`
- **Then:** List of sessions where none are agent sessions (`agent-*.jsonl` files excluded).; only main sessions listed (no `agent-` prefixed sessions)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-2: Value 1 accepted (agent sessions only)

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list agent::1`
- **Then:** List of only agent sessions (`agent-*.jsonl` files) if any exist; empty list otherwise.; + only agent sessions listed (or empty list if none exist in fixture)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-3: Value 2 rejected

- **Given:** clean environment
- **When:** `clg .list agent::2`
- **Then:** `agent must be 0 or 1`; + error message `agent must be 0 or 1`
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-4: String "yes" rejected

- **Given:** clean environment
- **When:** `clg .list agent::yes`
- **Then:** `agent must be 0 or 1`; + error message `agent must be 0 or 1`
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-5: Unset returns all session types

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list`
- **Then:** All sessions regardless of type — both main sessions and agent sessions appear.; + result set is superset of both `agent::0` and `agent::1` result sets
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-6: agent::1 auto-enables sessions display in .list

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list agent::1`
- **Then:** Project list with session-level detail shown under each project (sessions auto-displayed).; + sessions section visible in output (auto-enabled by `agent::1`)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-7: agent::0 auto-enables sessions display in .list

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list agent::0`
- **Then:** Project list with session-level detail shown under each project; only main sessions displayed.; + sessions section visible in output (auto-enabled by `agent::0`)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)
