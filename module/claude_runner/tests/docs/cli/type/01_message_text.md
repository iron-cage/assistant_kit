# Type :: `MessageText`

Validation tests for the `MessageText` semantic type. Tests validate multi-word joining, `--` separator handling, and empty-input behavior.

**Source:** [type.md](../../../../docs/cli/type.md#type--1-messagetext)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | Single-token message → forwarded verbatim | Parsing |
| TC-2 | Multi-word message (unquoted) → words joined with single space | Parsing |
| TC-3 | `--` separator → everything after treated as positional | Separator |
| TC-4 | Empty positional `""` → treated as no message (interactive mode) | Boundary |
| TC-5 | Message already ending with `ultrathink` → no double append | Idempotent Guard |

## Test Coverage Summary

- Parsing: 2 tests (TC-1, TC-2)
- Separator: 1 test (TC-3)
- Boundary: 1 test (TC-4)
- Idempotent Guard: 1 test (TC-5)

**Total:** 5 test cases

## Test Cases

---

### TC-1: Single-token message → forwarded verbatim

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command contains `"Fix bug\n\nultrathink"` (or equivalent quoted); single-token message preserved
- **Exit:** 0
- **Source:** [type.md — MessageText](../../../../docs/cli/type.md#type--1-messagetext)

---

### TC-2: Multi-word message → words joined with space

- **Given:** clean environment
- **When:** `clr --dry-run Fix the bug`
- **Then:** Message in assembled command is `"Fix the bug\n\nultrathink"`; three tokens joined with single space
- **Exit:** 0
- **Source:** [type.md — MessageText](../../../../docs/cli/type.md#type--1-messagetext)

---

### TC-3: `--` separator → everything after is positional

- **Given:** clean environment
- **When:** `clr --dry-run -- --not-a-flag`
- **Then:** `--not-a-flag` treated as message content, not a flag; assembled command contains it as positional text
- **Exit:** 0
- **Source:** [type.md — MessageText](../../../../docs/cli/type.md#type--1-messagetext)

---

### TC-4: Empty positional `""` → no message (interactive mode)

- **Given:** clean environment
- **When:** `clr --dry-run ""`
- **Then:** Empty string treated as no message; `--print` absent from assembled command
- **Exit:** 0
- **Source:** [type.md — MessageText](../../../../docs/cli/type.md#type--1-messagetext)

---

### TC-5: Message ending with `ultrathink` → no double append

- **Given:** clean environment
- **When:** `clr --dry-run "Fix the bug ultrathink"`
- **Then:** Assembled command does NOT contain `ultrathink\n\nultrathink`; idempotent guard prevents double suffix
- **Exit:** 0
- **Source:** [type.md — MessageText](../../../../docs/cli/type.md#type--1-messagetext)
