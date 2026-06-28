# Format :: 3. Text

Output format verification tests for plain-text export.

**Source:** [format/03_text.md](../../../../docs/cli/format/03_text.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FM-1 | Header block uses plain labels without markup | Structure |
| FM-2 | Entry prefix uses [Role] format (no H2 headings) | Structure |
| FM-3 | Thinking blocks absent from output | Content Filtering |
| FM-4 | Tool use and tool results absent from output | Content Filtering |
| FM-5 | No Markdown syntax in output | Plain-Text Constraint |

## Test Coverage Summary

- Structure: 2 tests (FM-1, FM-2)
- Content Filtering: 2 tests (FM-3, FM-4)
- Plain-Text Constraint: 1 test (FM-5)

**Total:** 5 cases

## Test Cases

---

### FM-1: Header block uses plain labels without markup

**Command:**
```
clg .export session_id::{id} format::text output::{file}.txt
```

**Expected behavior:**
- File starts with `Session: {session_id}` (no `#` prefix, no `**` bold)
- Second line is `Path: {storage_path}`
- Third line is `Entries: {count}`
- No bold markers (`**`), no backticks around values in the header

---

### FM-2: Entry prefix uses [Role] format (no H2 headings)

**Command:**
```
clg .export session_id::{id} format::text output::{file}.txt
```

**Expected behavior:**
- User entries begin with `[User] {timestamp}`
- Assistant entries begin with `[Assistant] {timestamp}`
- No `## Entry N - Role` heading syntax present in the file
- Role prefixes are capitalized (`User`, `Assistant`)

---

### FM-3: Thinking blocks absent from output

**Command:**
```
clg .export session_id::{id_with_thinking} format::text output::{file}.txt
```

**Expected behavior:**
- Output contains no `<details>` tags
- Output contains no `Thinking (` substring
- The thinking content itself does not appear in the file
- Text content from the same entry still appears

---

### FM-4: Tool use and tool results absent from output

**Command:**
```
clg .export session_id::{id_with_tools} format::text output::{file}.txt
```

**Expected behavior:**
- Output contains no `**Tool Use**:` lines
- Output contains no `**Tool Result**:` lines
- Tool input JSON does not appear in the file
- Tool result content does not appear in the file

---

### FM-5: No Markdown syntax in output

**Command:**
```
clg .export session_id::{id} format::text output::{file}.txt
```

**Expected behavior:**
- File contains no `#` heading characters at line start
- File contains no `**` bold markers
- File contains no `` ` `` backtick code delimiters
- Output is readable as plain text without a Markdown renderer
