# Test: Invariant — Exit Code Contract

Test case planning for [invariant/006_exit_codes.md](../../../docs/invariant/006_exit_codes.md). Tests validate that CLR-layer exit codes are correct at the binary level — timeout produces exit 4, expect mismatch produces exit 3, and gate bypass produces exit 0.

**Source:** [invariant/006_exit_codes.md](../../../docs/invariant/006_exit_codes.md)
**Related:** [type/13_error_kind.md](../../../docs/cli/type/13_error_kind.md), [type/14_error_class.md](../../../docs/cli/type/14_error_class.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Timeout → exit 4; stderr contains `"Error: timeout after 1s"` | CLR Exit Code |
| EC-2 | Expect mismatch → exit 3; stderr contains mismatch message | CLR Exit Code |
| EC-3 | Gate bypass (`--max-sessions 0`) → exit 0 on success | CLR Exit Code |

## Test Coverage Summary

- CLR Exit Code: 3 tests (EC-1, EC-2, EC-3)

**Total:** 3 test cases

## Architectural Constraint

These are integration tests that verify CLR-layer exit codes at the binary level, not unit tests
of `classify_error()`. EC-1 requires a real sleeping process killed by the CLR watchdog — the
subprocess must NOT be mocked. EC-2 requires a fake claude that prints output not matching the
`--expect` pattern. EC-3 requires a fake claude that exits 0 with `--max-sessions 0` to confirm
gate bypass does not interfere with the exit code. All tests must NOT use `#[ignore]` — use
`--max-sessions 0` to bypass the session gate instead.

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `ec_01_timeout_exits_4` | `exit_code_contract_test.rs` |
| EC-2 | `ec_02_expect_mismatch_exits_3` | `exit_code_contract_test.rs` |
| EC-3 | `ec_03_gate_bypass_exits_0` | `exit_code_contract_test.rs` |

---

### EC-1: Timeout → exit 4

- **Given:** fake `claude` process that sleeps 10s; `--timeout 1 --max-sessions 0 -p "x"`
- **When:** `clr --timeout 1 --max-sessions 0 -p "x"` using sleeping fake process
- **Then:** `clr` exits 4; stderr contains `"Error: timeout after 1s"`
- **Exit:** 4
- **Source:** [invariant/006_exit_codes.md](../../../docs/invariant/006_exit_codes.md) Rule 3 (Timeout uses exit 4)

---

### EC-2: Expect mismatch → exit 3

- **Given:** fake `claude` that prints `"foo"` to stdout and exits 0; `--expect "bar" --max-sessions 0 -p "x"`
- **When:** `clr --expect "bar" --max-sessions 0 -p "x"` using fake script
- **Then:** `clr` exits 3; stderr contains expect-mismatch message
- **Exit:** 3
- **Source:** [invariant/006_exit_codes.md](../../../docs/invariant/006_exit_codes.md) Rule 4 (Exit 3 is exclusively expect validation)

---

### EC-3: Gate bypass → exit 0

- **Given:** fake `claude` that exits 0; `--max-sessions 0 -p "x"`
- **When:** `clr --max-sessions 0 -p "x"` using fake script
- **Then:** `clr` exits 0 (gate disabled; subprocess success relayed)
- **Exit:** 0
- **Source:** [invariant/006_exit_codes.md](../../../docs/invariant/006_exit_codes.md) Rule 1 (Exit 0 means success)
