# Pitfall :: Vacuous Assertions Mask Stub Implementations

<!-- BUG-002 — new PF- contract test spec mirroring the vacuous-assertions pitfall -->

Contract tests verifying that command integration tests assert real output content, not just process liveness.

**Source:** [cli/pitfall/04_vacuous_assertions_mask_stubs.md](../../../../docs/cli/pitfall/04_vacuous_assertions_mask_stubs.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| PF-1 | No integration test's sole assertion is `is_ok()`/`is_err()` on a `Command::output()` result | Assertion Completeness |
| PF-2 | No integration test discards a captured exit code without asserting its value | Exit Code Assertion |
| PF-3 | `.tail`'s integration tests assert exact stdout content against known fixture entries | Regression Guard |

## Test Coverage Summary

- Assertion Completeness: 1 test (PF-1)
- Exit Code Assertion: 1 test (PF-2)
- Regression Guard: 1 test (PF-3)

**Total:** 3 pitfall contract cases

**Implementation target:** `tests/invariant_contracts_test.rs`

## Test Cases

---

### PF-1: No integration test's sole assertion is `is_ok()`/`is_err()` on a `Command::output()` result

- **Given:** all files under `tests/*.rs`
- **When:** each test function is scanned for its full set of assertions
- **Then:** no test function's ONLY assertion is `.is_ok()` or `.is_err()` called directly on a `Command::output()`/`Command::status()` `Result` — every such result must also be checked for stdout/stderr content or a specific exit code
- **Note:** Bug-driven expansion: Gap Class — no pitfall/invariant entry required CLI routine tests to assert real output content instead of process liveness alone. Source: BUG-002.

---

### PF-2: No integration test discards a captured exit code without asserting its value

- **Given:** all files under `tests/*.rs`
- **When:** each test function is scanned for `.status.code()` or equivalent exit-code capture
- **Then:** no captured exit code is bound to `_` or otherwise discarded without an assertion on its value
- **Note:** Bug-driven expansion: Gap Class — no pitfall/invariant entry required CLI routine tests to assert real output content instead of process liveness alone. Source: BUG-002.

---

### PF-3: `.tail`'s integration tests assert exact stdout content against known fixture entries

- **Given:** the `.tail` command's integration tests in `tests/cli_cmd_tail_test.rs`
- **When:** each test's assertions are inspected
- **Then:** every test asserts on the actual entry content returned by `.tail` (not just `Result::is_ok()` or a discarded exit code) — direct regression guard against this defect pattern recurring
- **Note:** Bug-driven expansion: Gap Class — no pitfall/invariant entry required CLI routine tests to assert real output content instead of process liveness alone. Source: BUG-002.
