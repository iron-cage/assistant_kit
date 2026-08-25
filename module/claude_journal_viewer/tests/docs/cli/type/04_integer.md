# Type :: `Integer`

Validation tests for the `Integer` fundamental type. Tests validate
non-negative parsing, rejection of negative/non-numeric input, and
declared-range acceptance.

**Source:** [type/04_integer.md](../../../../docs/cli/type/04_integer.md)

## Test Case Index

| ID | Test Name | Category | Status | Implemented as |
|----|-----------|----------|--------|----------------|
| TC-1 | Valid non-negative integer accepted | Parsing | ✅ | `ec37_integer_params_honour_documented_domain` |
| TC-2 | Negative value -> exit 1 | Error Handling | ✅ | `ec37_integer_params_honour_documented_domain` |
| TC-3 | Non-numeric input -> exit 1 | Error Handling | ✅ | `ec37_integer_params_honour_documented_domain` |
| TC-4 | `exit::255` (upper bound of declared range) -> accepted | Boundary | ✅ | `ec37_integer_params_honour_documented_domain` |
| TC-5 | `exit::256` (one past the declared range) -> exit 1 | Boundary | ✅ | `ec37_integer_params_honour_documented_domain` |

## Test Coverage Summary

- Parsing: 1 test (TC-1)
- Error Handling: 2 tests (TC-2, TC-3)
- Boundary: 2 tests (TC-4, TC-5)

**Total:** 5 test cases

Like the `Boolean` cases, these are enforced by one table-driven test that
walks the Referenced Parameters table in
[type/04_integer.md](../../../../docs/cli/type/04_integer.md) — `exit`,
`limit` and `verbosity` — rather than naming sites individually. `refresh` is
the fourth entry and is `.serve`-only; it is covered by serve_test's FT-11.

TC-4 had no counterpart on the other side of the boundary until TC-5 was
added, which is how `exit` came to parse as `i32`: 255 was asserted to work
and nothing asserted that 256 did not, so an out-of-range value parsed
cleanly and then matched nothing — indistinguishable, at the command line,
from a filter that legitimately found no failures.

## Test Cases

---

### TC-1: Valid non-negative integer accepted

- **Given:** clean environment
- **When:** `clj .list limit::10`
- **Then:** exit 0; `limit` parses as 10
- **Exit:** 0
- **Source:** [type/04_integer.md](../../../../docs/cli/type/04_integer.md)

---

### TC-2: Negative value -> exit 1

- **Given:** clean environment
- **When:** `clj .list limit::-5`
- **Then:** exit 1; stderr indicates the value is not a valid non-negative integer
- **Exit:** 1
- **Source:** [type/04_integer.md](../../../../docs/cli/type/04_integer.md)

---

### TC-3: Non-numeric input -> exit 1

- **Given:** clean environment
- **When:** `clj .list limit::abc`
- **Then:** exit 1; stderr contains `invalid integer 'abc' for parameter 'limit'`
- **Exit:** 1
- **Source:** [type/04_integer.md](../../../../docs/cli/type/04_integer.md)

---

### TC-4: `exit::255` (upper bound of declared range) -> accepted

- **Given:** journal containing an event with exit code 255
- **When:** `clj .list exit::255`
- **Then:** exit 0; the event with exit code 255 is shown
- **Exit:** 0
- **Source:** [type/04_integer.md](../../../../docs/cli/type/04_integer.md), [param/05_exit.md](../../../../docs/cli/param/05_exit.md)

---

### TC-5: `exit::256` (one past the declared range) -> exit 1

- **Given:** clean environment
- **When:** `clj .list exit::256`
- **Then:** exit 1; stderr contains `invalid integer '256' for parameter 'exit'`
- **Exit:** 1
- **Source:** [type/04_integer.md](../../../../docs/cli/type/04_integer.md), [param/05_exit.md](../../../../docs/cli/param/05_exit.md)

The range is enforced rather than advisory because a Unix wait status is one
byte: 256 is not an exit code that happens to be absent from this journal, it
is not an exit code. `verbosity` deliberately clamps above its own range
instead of erroring — asking for more detail than exists is a coherent
request, and [param_group/02_display.md](../../../../docs/cli/param_group/02_display.md)
says so — so the two are not one rule and are not asserted in one loop.
