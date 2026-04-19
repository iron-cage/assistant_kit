# Parameter :: `verbosity::`

Edge case tests for the `verbosity::` parameter. Tests validate range enforcement, alias handling, and per-command defaults.

**Source:** [params.md#parameter--18-verbosity](../../../../../docs/cli/params.md#parameter--18-verbosity) | [types.md#verbositylevel](../../../../../docs/cli/types.md#verbositylevel)

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

## Test Cases

### EC-1: Value 0 accepted (silent mode)

**Goal:** Verify that `verbosity::0` is accepted and produces minimal output.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .status verbosity::0`
**Expected Output:** Minimal output with no decorative headers or labels; machine-readable format only.
**Verification:**
- Command exits with code 0
- Output contains no verbose section headers or extra whitespace lines
**Pass Criteria:** exit 0 + output is reduced/minimal compared to default level
**Source:** [params.md](../../../../../docs/cli/params.md)

---

### EC-2: Value 5 accepted (max allowed)

**Goal:** Verify that `verbosity::5` (the maximum valid value) is accepted without error.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .status verbosity::5`
**Expected Output:** Full verbose output with all available fields shown.
**Verification:**
- Command exits with code 0
- No error message about out-of-range value appears on stderr
**Pass Criteria:** exit 0 + command produces output (not an error)
**Source:** [params.md](../../../../../docs/cli/params.md)

---

### EC-3: Value 6 rejected with error message

**Goal:** Verify that `verbosity::6` (one above the maximum) produces the exact error message.
**Setup:** None
**Command:** `clg .status verbosity::6`
**Expected Output:** `verbosity must be 0-5, got 6`
**Verification:**
- Command exits with code 1
- Stderr contains the string `verbosity must be 0-5, got 6`
**Pass Criteria:** exit 1 + error message `verbosity must be 0-5, got 6`
**Source:** [params.md](../../../../../docs/cli/params.md)

---

### EC-4: Negative value rejected

**Goal:** Verify that a negative verbosity value is rejected with the out-of-range error message.
**Setup:** None
**Command:** `clg .status verbosity::-1`
**Expected Output:** `verbosity must be 0-5, got -1`
**Verification:**
- Command exits with code 1
- Stderr contains the string `verbosity must be 0-5, got -1`
**Pass Criteria:** exit 1 + error message `verbosity must be 0-5, got -1`
**Source:** [params.md](../../../../../docs/cli/params.md)

---

### EC-5: Non-integer string rejected

**Goal:** Verify that a non-integer string for verbosity produces the non-integer error message.
**Setup:** None
**Command:** `clg .status verbosity::high`
**Expected Output:** `verbosity must be an integer 0-5, got high`
**Verification:**
- Command exits with code 1
- Stderr contains the string `verbosity must be an integer 0-5, got high`
**Pass Criteria:** exit 1 + error message `verbosity must be an integer 0-5, got high`
**Source:** [params.md](../../../../../docs/cli/params.md)

---

### EC-6: Alias v:: accepted same as verbosity::

**Goal:** Verify that the `v::` alias is functionally equivalent to `verbosity::`.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .status v::2`
**Expected Output:** Same output as `clg .status verbosity::2`; detailed view with extended information.
**Verification:**
- Command exits with code 0
- No error message about unknown parameter appears on stderr
- Output matches what `verbosity::2` would produce
**Pass Criteria:** exit 0 + output identical to `verbosity::2` output
**Source:** [params.md](../../../../../docs/cli/params.md)

---

### EC-7: Omitted uses default of 1

**Goal:** Verify that omitting `verbosity::` uses the default level of 1 (standard summary).
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .status`
**Expected Output:** Standard summary output; same as `clg .status verbosity::1`.
**Verification:**
- Command exits with code 0
- Output matches the output of `clg .status verbosity::1`
- Output is neither the minimal level-0 format nor the fully verbose level-3+ format
**Pass Criteria:** exit 0 + output is the standard summary (equivalent to `verbosity::1`)
**Source:** [params.md](../../../../../docs/cli/params.md)

---

### EC-8: Float value rejected

**Goal:** Verify that a float value (e.g., `1.5`) is rejected as a non-integer.
**Setup:** None
**Command:** `clg .status verbosity::1.5`
**Expected Output:** `verbosity must be an integer 0-5, got 1.5`
**Verification:**
- Command exits with code 1
- Stderr contains the string `verbosity must be an integer 0-5, got 1.5`
**Pass Criteria:** exit 1 + error message `verbosity must be an integer 0-5, got 1.5`
**Source:** [params.md](../../../../../docs/cli/params.md)
