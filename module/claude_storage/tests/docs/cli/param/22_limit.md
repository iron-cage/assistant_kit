# Parameter :: `limit::`

Edge case tests for the `limit::` parameter. Tests validate integer enforcement, capping behavior, and default (uncapped) behavior.

**Source:** [param/22_limit.md](../../../../docs/cli/param/22_limit.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `limit::5` → max 5 sessions shown per project | Happy Path |
| EC-2 | `limit::0` → no cap (all sessions shown) | Default |
| EC-3 | Negative limit (e.g., `limit::-1`) → rejected | Boundary Values |
| EC-4 | `limit::` empty value → rejected | Boundary Values |
| EC-5 | `limit::100` when project has fewer sessions → all shown | Boundary Values |
| EC-6 | `limit::` non-integer value → rejected | Type Validation |

## Test Coverage Summary

- Happy Path: 1 test (EC-1)
- Default: 1 test (EC-2)
- Boundary Values: 3 tests (EC-3, EC-4, EC-5)
- Type Validation: 1 test (EC-6)

**Total:** 6 edge cases

**Behavioral Divergence Pair:** EC-1 (limit::5, capped) ↔ EC-2 (limit::0, uncapped)

## Test Cases

---

### EC-1: `limit::5` → max 5 sessions per project

- **Commands:** `.projects`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project with 10 sessions)
- **When:** `clg .projects limit::5`
- **Then:** At most 5 sessions shown per project; excess sessions omitted
- **Exit:** 0
- **Source:** [param/22_limit.md](../../../../docs/cli/param/22_limit.md)

---

### EC-2: `limit::0` → all sessions shown (no cap)

- **Commands:** `.projects`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .projects limit::0`
- **Then:** All sessions shown per project; no capping applied
- **Exit:** 0
- **Source:** [param/22_limit.md](../../../../docs/cli/param/22_limit.md)

---

### EC-3: Negative limit rejected

- **Commands:** `.projects`
- **Given:** clean environment
- **When:** `clg .projects limit::-1`
- **Then:** Exit 1; error indicating `limit` must be a non-negative integer
- **Exit:** 1
- **Source:** [param/22_limit.md](../../../../docs/cli/param/22_limit.md)

---

### EC-4: Empty value rejected

- **Commands:** `.projects`
- **Given:** clean environment
- **When:** `clg .projects limit::`
- **Then:** Exit 1; error indicating `limit` requires a value
- **Exit:** 1
- **Source:** [param/22_limit.md](../../../../docs/cli/param/22_limit.md)

---

### EC-5: `limit::100` when project has fewer sessions → all shown

- **Commands:** `.projects`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project with 3 sessions)
- **When:** `clg .projects limit::100`
- **Then:** All 3 sessions shown (limit not reached); no error
- **Exit:** 0
- **Source:** [param/22_limit.md](../../../../docs/cli/param/22_limit.md)

---

### EC-6: Non-integer value rejected

- **Commands:** `.projects`
- **Given:** clean environment
- **When:** `clg .projects limit::five`
- **Then:** Exit 1; error indicating `limit` requires a non-negative integer
- **Exit:** 1
- **Source:** [param/22_limit.md](../../../../docs/cli/param/22_limit.md)
