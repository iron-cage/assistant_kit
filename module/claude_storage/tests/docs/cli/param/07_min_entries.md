# Parameter :: `min_entries::`

Edge case tests for the `min_entries::` parameter. Tests validate non-negative integer enforcement and auto-enable behavior.

**Source:** [params.md#parameter--7-min_entries](../../../../docs/cli/params.md#parameter--7-min_entries) | [types.md#entrycount](../../../../docs/cli/types.md#entrycount)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Value 0 accepted (no minimum) | Boundary Values |
| EC-2 | Value 1 accepted | Boundary Values |
| EC-3 | Large value (e.g., 10000) accepted | Boundary Values |
| EC-4 | Negative value rejected | Boundary Values |
| EC-5 | Float value rejected | Type Validation |
| EC-6 | String "ten" rejected | Type Validation |
| EC-7 | Auto-enables sessions display in .list | Auto-Enable |
| EC-8 | Unset shows all sessions (no threshold) | Default |

## Test Coverage Summary

- Boundary Values: 4 tests (EC-1, EC-2, EC-3, EC-4)
- Type Validation: 2 tests (EC-5, EC-6)
- Auto-Enable: 1 test (EC-7)
- Default: 1 test (EC-8)

**Total:** 8 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

## Test Cases

---

### EC-1: Value 0 accepted (no minimum)

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list min_entries::0`
- **Then:** All sessions listed regardless of entry count, including sessions with very few entries.; result set matches the unfiltered session count for the fixture
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-2: Value 1 accepted

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list min_entries::1`
- **Then:** Sessions with at least 1 entry listed; empty sessions excluded.; + only sessions with ≥ 1 entry appear in output
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-3: Large value (e.g., 10000) accepted

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list min_entries::10000`
- **Then:** Empty session list (no sessions in the fixture have 10000+ entries).; + empty result (large threshold accepted, no sessions match)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-4: Negative value rejected

- **Given:** clean environment
- **When:** `clg .list min_entries::-1`
- **Then:** `min_entries must be ≥ 0, got -1`; + error message `min_entries must be ≥ 0, got -1`
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-5: Float value rejected

- **Given:** clean environment
- **When:** `clg .list min_entries::2.5`
- **Then:** `min_entries must be a non-negative integer, got 2.5`; + error message `min_entries must be a non-negative integer, got 2.5`
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-6: String "ten" rejected

- **Given:** clean environment
- **When:** `clg .list min_entries::ten`
- **Then:** `min_entries must be a non-negative integer, got ten`; + error message `min_entries must be a non-negative integer, got ten`
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-7: Auto-enables sessions display in .list

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list min_entries::2`
- **Then:** Project list with per-project session rows shown; only sessions with ≥ 2 entries appear.; + sessions section visible in output (auto-enabled by `min_entries::2`)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-8: Unset shows all sessions (no threshold)

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list`
- **Then:** All sessions listed without any entry-count filter applied.; + all sessions in fixture are included (no implicit threshold applied)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)
