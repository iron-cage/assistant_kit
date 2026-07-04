# Parameter :: `open`

Edge case tests for the `open` parameter. Tests validate the
default (no auto-open) and the auto-open shortcut.

**Source:** [param/17_open.md](../../../../docs/cli/param/17_open.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Absent -> browser not opened | Default |
| EC-2 | `open::1` -> browser opened via `xdg-open`/`open` | Parsing |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Parsing: 1 test (EC-2)

**Total:** 2 edge cases

## Test Cases

---

### EC-1: Absent -> browser not opened

- **Given:** clean environment
- **When:** `clj .serve`
- **Then:** exit 0 on shutdown; no browser is launched
- **Exit:** 0
- **Source:** [param/17_open.md](../../../../docs/cli/param/17_open.md)

---

### EC-2: `open::1` -> browser opened via `xdg-open`/`open`

- **Given:** a desktop environment with `xdg-open` (Linux) or `open` (macOS) available
- **When:** `clj .serve open::1`
- **Then:** exit 0 on shutdown; the default browser is launched pointing at the viewer URL
- **Exit:** 0
- **Source:** [param/17_open.md](../../../../docs/cli/param/17_open.md)
