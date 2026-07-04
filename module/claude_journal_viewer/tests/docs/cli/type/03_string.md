# Type :: `String`

Validation tests for the `String` fundamental type. Tests validate that
plain UTF-8 text is unconstrained by the type itself, while specific
parameters layer additional constraints.

**Source:** [type/03_string.md](../../../../docs/cli/type/03_string.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | Arbitrary UTF-8 text accepted for `model` (substring match) | Unconstrained Text |
| TC-2 | `pattern` rejects invalid regex | Additional Constraint |
| TC-3 | `bind` rejects invalid IP address | Additional Constraint |
| TC-4 | `columns` accepts comma-separated names | Additional Constraint |

## Test Coverage Summary

- Unconstrained Text: 1 test (TC-1)
- Additional Constraint: 3 tests (TC-2, TC-3, TC-4)

**Total:** 4 test cases

## Test Cases

---

### TC-1: Arbitrary UTF-8 text accepted for `model` (substring match)

- **Given:** journal with events carrying various model names, including non-ASCII text
- **When:** `clj .list model::"claude-opus"`
- **Then:** exit 0; the raw string is accepted with no length limit or character restriction
- **Exit:** 0
- **Source:** [type/03_string.md](../../../../docs/cli/type/03_string.md)

---

### TC-2: `pattern` rejects invalid regex

- **Given:** clean environment
- **When:** `clj .search pattern::"[unclosed"`
- **Then:** exit 1; stderr indicates the pattern is not a valid regex
- **Exit:** 1
- **Source:** [type/03_string.md](../../../../docs/cli/type/03_string.md), [param/14_pattern.md](../../../../docs/cli/param/14_pattern.md)

---

### TC-3: `bind` rejects invalid IP address

- **Given:** clean environment
- **When:** `clj .serve bind::"999.999.999.999"`
- **Then:** exit 1; stderr indicates the address is not a valid IPv4/IPv6 address
- **Exit:** 1
- **Source:** [type/03_string.md](../../../../docs/cli/type/03_string.md), [param/16_bind.md](../../../../docs/cli/param/16_bind.md)

---

### TC-4: `columns` accepts comma-separated names

- **Given:** clean environment
- **When:** `clj .list columns::"time,cost,model"`
- **Then:** exit 0; only the `time`, `cost`, and `model` columns are shown
- **Exit:** 0
- **Source:** [type/03_string.md](../../../../docs/cli/type/03_string.md), [param/26_columns.md](../../../../docs/cli/param/26_columns.md)
