# Parameter :: `no_color`

Edge case tests for the `no_color` parameter. Tests validate the
default (colors enabled), the explicit disable, and the `NO_COLOR`
environment variable.

**Source:** [param/24_no_color.md](../../../../docs/cli/param/24_no_color.md)

## Test Case Index

| ID | Test Name | Category | Status | Implemented as |
|----|-----------|----------|--------|----------------|
| EC-1 | Absent -> ANSI colors enabled | Default | ✅ | `viewer_integration_test.rs::ec29_no_color_param_suppresses_ansi` (control half) |
| EC-2 | `no_color::1` -> plain text, no ANSI codes | Parsing | ✅ | `viewer_integration_test.rs::ec29_no_color_param_suppresses_ansi` |
| EC-3 | `NO_COLOR` env var set, param absent -> colors disabled | Environment Variable | ✅ | `viewer_integration_test.rs::ec11_no_color_suppresses_ansi` |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Parsing: 1 test (EC-2)
- Environment Variable: 1 test (EC-3)

**Total:** 3 edge cases (all executable)

EC-1 and EC-2 are asserted in the same test rather than separately, and that
pairing is the point: a build where color is globally off would satisfy EC-2
alone. The control half — same command, no `no_color::` — fails in that case,
so the pair cannot pass vacuously.

EC-2 was the case whose absence let `no_color::` sit unread for the whole of
the parameter's documented life: EC-3 covered the environment variable, which
is a *different input path* to the same decision, and passing meant nothing
about whether the parameter form worked.

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
