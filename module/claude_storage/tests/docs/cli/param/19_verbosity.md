# Parameter :: `verbosity::`

Edge case tests for the `verbosity::` parameter. Tests validate range enforcement, alias handling, and per-command defaults.

**Source:** [params.md#parameter--18-verbosity](../../../../docs/cli/params.md#parameter--18-verbosity) | [types.md#verbositylevel](../../../../docs/cli/types.md#verbositylevel)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Value 0 accepted (silent mode) | Boundary Values |
| EC-2 | Value 5 accepted (max allowed) | Boundary Values |
| EC-3 | Value 6 rejected with error message | Boundary Values |
| EC-4 | Negative value rejected | Boundary Values |
| EC-5 | Non-integer string rejected | Type Validation |
| EC-6 | Alias v:: accepted same as verbosity:: | Alias |
| EC-7 | Omitted uses default of 1 | Default |
| EC-8 | Float value rejected | Type Validation |

## Test Coverage Summary

- Boundary Values: 4 tests (EC-1, EC-2, EC-3, EC-4)
- Type Validation: 2 tests (EC-5, EC-8)
- Alias: 1 test (EC-6)
- Default: 1 test (EC-7)

**Total:** 8 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

## Test Cases

---

### EC-1: Value 0 accepted (silent mode)

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .status verbosity::0`
- **Then:** Minimal output with no decorative headers or labels; machine-readable format only.; output is reduced/minimal compared to default level
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-2: Value 5 accepted (max allowed)

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .status verbosity::5`
- **Then:** Full verbose output with all available fields shown.; + command produces output (not an error)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-3: Value 6 rejected with error message

- **Given:** clean environment
- **When:** `clg .status verbosity::6`
- **Then:** `verbosity must be 0-5, got 6`; + error message `verbosity must be 0-5, got 6`
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-4: Negative value rejected

- **Given:** clean environment
- **When:** `clg .status verbosity::-1`
- **Then:** `verbosity must be 0-5, got -1`; + error message `verbosity must be 0-5, got -1`
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-5: Non-integer string rejected

- **Given:** clean environment
- **When:** `clg .status verbosity::high`
- **Then:** `verbosity must be an integer 0-5, got high`; + error message `verbosity must be an integer 0-5, got high`
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-6: Alias v:: accepted same as verbosity::

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .status v::2`
- **Then:** Same output as `clg .status verbosity::2`; detailed view with extended information.; + output identical to `verbosity::2` output
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-7: Omitted uses default of 1

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .status`
- **Then:** Standard summary output; same as `clg .status verbosity::1`.; + output is the standard summary (equivalent to `verbosity::1`)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-8: Float value rejected

- **Given:** clean environment
- **When:** `clg .status verbosity::1.5`
- **Then:** `verbosity must be an integer 0-5, got 1.5`; + error message `verbosity must be an integer 0-5, got 1.5`
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md)
