# Type :: `MessageText`

Validation tests for the `MessageText` semantic type. Tests validate multi-word joining, `--` separator handling, and empty-input behavior.

**Source:** [type/01_message_text.md](../../../../docs/cli/type/01_message_text.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | Single-token message → forwarded verbatim | Parsing |
| TC-2 | Multi-word message (unquoted) → words joined with single space | Parsing |
| TC-3 | `--` separator → everything after treated as positional | Separator |
| TC-4 | Empty positional `""` → treated as no message (interactive mode) | Boundary |
| TC-5 | Message with special characters → preserved in assembled command | Special Characters |

## Test Coverage Summary

- Parsing: 2 tests (TC-1, TC-2)
- Separator: 1 test (TC-3)
- Boundary: 1 test (TC-4)
- Special Characters: 1 test (TC-5)

**Total:** 5 test cases

## Test Cases

---

### TC-1: Single-token message → forwarded verbatim

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command contains `"Fix bug\n\nultrathink"` (or equivalent quoted); single-token message preserved
- **Exit:** 0
- **Source:** [type/01_message_text.md](../../../../docs/cli/type/01_message_text.md)

---

### TC-2: Multi-word message → words joined with space

- **Given:** clean environment
- **When:** `clr --dry-run Fix the bug`
- **Then:** Message in assembled command is `"Fix the bug\n\nultrathink"`; three tokens joined with single space
- **Exit:** 0
- **Source:** [type/01_message_text.md](../../../../docs/cli/type/01_message_text.md)

---

### TC-3: `--` separator → everything after is positional

- **Given:** clean environment
- **When:** `clr --dry-run -- --not-a-flag`
- **Then:** `--not-a-flag` treated as message content, not a flag; assembled command contains it as positional text
- **Exit:** 0
- **Source:** [type/01_message_text.md](../../../../docs/cli/type/01_message_text.md)

---

### TC-4: Empty positional `""` → no message (interactive mode)

- **Given:** clean environment
- **When:** `clr --dry-run ""`
- **Then:** Empty string treated as no message; `--print` absent from assembled command
- **Exit:** 0
- **Source:** [type/01_message_text.md](../../../../docs/cli/type/01_message_text.md)

---

### TC-5: Message with special characters → preserved in assembled command

- **Given:** clean environment
- **When:** `clr --dry-run "Fix the 'auth' bug & deploy"`
- **Then:** Message content including single quotes and `&` preserved in assembled command; special shell characters do not corrupt the message
- **Exit:** 0
- **Source:** [type/01_message_text.md](../../../../docs/cli/type/01_message_text.md)
