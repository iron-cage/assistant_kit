# Test: Invariant 004 — No Process Execution

Property assertion cases for `docs/invariant/004_no_process_execution.md`. Verifies that
`claude_profile`'s source tree contains zero `std::process` imports and that the automated
responsibility test enforces this at every CI run.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IN-1 | Source tree contains zero std::process occurrences | Invariant holds (normal) |
| IN-2 | Automated responsibility test catches any future violation | Invariant holds (boundary) |

**Total:** 2 IN cases

---

### IN-1: Source tree contains zero std::process occurrences

- **Given:** The `src/` directory at the current HEAD
- **When:** `grep -r "std::process" src/` is run
- **Then:** The command returns empty output (exit 1 from grep); no file in `src/` imports or
  uses `std::process` in any form
- **Source:** [docs/invariant/004_no_process_execution.md](../../../docs/invariant/004_no_process_execution.md)

---

### IN-2: Automated responsibility test catches any future violation

- **Given:** The test `responsibility_no_process_execution_test` exists in `tests/` and is
  executed as part of the standard test suite
- **When:** `cargo nextest run --test responsibility_no_process_execution_test` is run
- **Then:** The test passes; if `std::process` were introduced into `src/`, this test would fail
  the CI build, enforcing the invariant automatically
- **Source:** [docs/invariant/004_no_process_execution.md](../../../docs/invariant/004_no_process_execution.md)
