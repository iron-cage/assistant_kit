# Parameter :: `no_color`

Edge case tests for the `no_color` parameter. Tests validate the
default (colors enabled), the explicit disable, and the `NO_COLOR`
environment variable.

**Source:** [param/24_no_color.md](../../../../docs/cli/param/24_no_color.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Absent -> ANSI colors enabled | Default |
| EC-2 | `no_color::1` -> plain text, no ANSI codes | Parsing |
| EC-3 | `NO_COLOR` env var set, param absent -> colors disabled | Environment Variable |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Parsing: 1 test (EC-2)
- Environment Variable: 1 test (EC-3)

**Total:** 3 edge cases

## Test Cases

---

### EC-1: Absent -> ANSI colors enabled

- **Given:** `NO_COLOR` is unset
- **When:** `clj .list`
- **Then:** exit 0; table output includes ANSI color escapes
- **Exit:** 0
- **Source:** [param/24_no_color.md](../../../../docs/cli/param/24_no_color.md)

---

### EC-2: `no_color::1` -> plain text, no ANSI codes

- **Given:** clean environment
- **When:** `clj .list no_color::1`
- **Then:** exit 0; table output contains no ANSI color escapes
- **Exit:** 0
- **Source:** [param/24_no_color.md](../../../../docs/cli/param/24_no_color.md)

---

### EC-3: `NO_COLOR` env var set, param absent -> colors disabled

- **Given:** `NO_COLOR=1` is set in the environment
- **When:** `clj .stats`
- **Then:** exit 0; output contains no ANSI color escapes, even though `no_color` was not passed
- **Exit:** 0
- **Source:** [param/24_no_color.md](../../../../docs/cli/param/24_no_color.md)
