# Type :: `Boolean`

Validation tests for the `Boolean` fundamental type. Tests validate the
0/1 integer convention and rejection of any other value.

**Source:** [type/08_boolean.md](../../../../docs/cli/type/08_boolean.md)

## Test Case Index

| ID | Test Name | Category | Status | Implemented as |
|----|-----------|----------|--------|----------------|
| TC-1 | `0` -> false/disabled | Parsing | ✅ | `ec36_boolean_params_accept_only_0_and_1` |
| TC-2 | `1` -> true/enabled | Parsing | ✅ | `ec36_boolean_params_accept_only_0_and_1` |
| TC-3 | Any other value (`true`, `2`, `yes`) -> exit 1 | Error Handling | ✅ | `ec36_boolean_params_accept_only_0_and_1` |

## Test Coverage Summary

- Parsing: 2 tests (TC-1, TC-2)
- Error Handling: 1 test (TC-3)

**Total:** 3 test cases

All three are enforced by one table-driven case rather than three, because the
contract they describe is per-*type*, not per-parameter. EC-36 walks the
Referenced Parameters table in
[type/08_boolean.md](../../../../docs/cli/type/08_boolean.md) and applies all
three to each entry, so a parameter added to that table without an
implementation fails rather than going unnoticed.

TC-3 was specified here long before anything enforced it: `.serve open::`,
`.chart open::` and `no_color::` matched `"1" | "true"` and silently treated
every other value as `0`, and `.prune dry_run::` accepted `true`/`false` too.
The plan was right; nothing checked it.

## Test Cases

---

### TC-1: `0` -> false/disabled

- **Given:** clean environment
- **When:** `clj .list reverse::0`
- **Then:** exit 0; reverse sort is disabled
- **Exit:** 0
- **Source:** [type/08_boolean.md](../../../../docs/cli/type/08_boolean.md), [param/12_reverse.md](../../../../docs/cli/param/12_reverse.md)

---

### TC-2: `1` -> true/enabled

- **Given:** clean environment
- **When:** `clj .list reverse::1`
- **Then:** exit 0; reverse sort is enabled
- **Exit:** 0
- **Source:** [type/08_boolean.md](../../../../docs/cli/type/08_boolean.md), [param/12_reverse.md](../../../../docs/cli/param/12_reverse.md)

---

### TC-3: Any other value (`true`, `2`, `yes`) -> exit 1

- **Given:** clean environment
- **When:** `clj .list reverse::true`
- **Then:** exit 1; stderr contains `invalid boolean 'true' for parameter 'reverse' — expected 0 or 1`
- **Exit:** 1
- **Source:** [type/08_boolean.md](../../../../docs/cli/type/08_boolean.md)

`reverse::true` is the illustrative case; EC-36 applies the same three values
plus `false`, `banana`, `-1` and the empty string to `dry_run`, `no_color` and
`.chart open::`, and serve_test's FT-14 covers `.serve open::`. `true` and
`false` are in the set deliberately — they were accepted at one site and
silently ignored at three others, which is the divergence this case closes.
