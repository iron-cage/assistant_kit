# Type :: `String`

Validation tests for the `String` fundamental type. Tests validate that
plain UTF-8 text is unconstrained by the type itself, while specific
parameters layer additional constraints.

**Source:** [type/03_string.md](../../../../docs/cli/type/03_string.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | Arbitrary UTF-8 text accepted for `model` (substring match) | Unconstrained Text |
| TC-2 | `pattern::"[unclosed"` is searched for literally, exit 0 | Unconstrained Text |
| TC-3 | `bind::"999.999.999.999"` fails at bind, not at parse | Deferred Constraint |

## Test Coverage Summary

- Unconstrained Text: 2 tests (TC-1, TC-2)
- Deferred Constraint: 1 test (TC-3)

**Total:** 3 test cases

**No `String` parameter is validated for its content at parse time.** TC-2 and
TC-3 both used to be filed as parse-time "Additional Constraint" cases, which
described a validation layer that does not exist. What is actually true is one
statement with two shapes: a value the caller thinks is malformed is accepted
either way, and then either does something harmless (`pattern`) or fails later
in terms that name the operation rather than the parameter (`bind`).

TC-4 covered `columns`, which was retracted rather than built. IDs keep their
historical numbers instead of being renumbered, so a reference written against an
earlier revision still resolves to the same case.

## Test Cases

---

### TC-1: Arbitrary UTF-8 text accepted for `model` (substring match)

- **Given:** journal with events carrying various model names, including non-ASCII text
- **When:** `clj .list model::"claude-opus"`
- **Then:** exit 0; the raw string is accepted with no length limit or character restriction
- **Exit:** 0
- **Source:** [type/03_string.md](../../../../docs/cli/type/03_string.md)

---

### TC-2: `pattern::"[unclosed"` is searched for literally, exit 0

- **Given:** journal with one event whose `stdout` contains the literal text `[unclosed` and others that do not
- **When:** `clj .search pattern::"[unclosed"`
- **Then:** exit **0**; the literal-text event matches and nothing is rejected — the crate has no `regex` dependency, so there is no such thing as an invalid pattern
- **Exit:** 0
- **Note:** this case previously asserted exit 1 with "not a valid regex", a diagnostic no code path produces. The genuine hazard is the opposite one: a caller writes a regex, gets exit 0 and no matches, and reads that as "nothing found" rather than "that query cannot work"
- **Source:** [type/03_string.md](../../../../docs/cli/type/03_string.md), [param/14_pattern.md](../../../../docs/cli/param/14_pattern.md)

---

### TC-3: `bind::"999.999.999.999"` fails at bind, not at parse

- **Given:** clean environment
- **When:** `clj .serve bind::"999.999.999.999" port::0`
- **Then:** exit 1; stderr contains `could not start server on 999.999.999.999:0`
- **Exit:** 1
- **Note:** the rejection happens at bind time, not parse time — the value is handed to `tiny_http::Server::http()` unvalidated and the OS refuses it. The assertion is therefore on the bind-failure message, not on a "not a valid IPv4/IPv6 address" wording that no code produces. What the case actually protects against is a malformed address being silently swallowed in favour of some default
- **Implemented as:** `tc3_invalid_bind_address_exits_1`
- **Source:** [type/03_string.md](../../../../docs/cli/type/03_string.md), [param/16_bind.md](../../../../docs/cli/param/16_bind.md)
