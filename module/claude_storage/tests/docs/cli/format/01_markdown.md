# Format :: Markdown

Output format verification tests for Markdown export.

**Source:** [format/01_markdown.md](../../../../docs/cli/format/01_markdown.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FM-1 | Header block present with session metadata | Structure |
| FM-2 | Entry headings use H2 with sequential numbering | Structure |
| FM-3 | Thinking block rendered as collapsible details | Content Blocks |
| FM-4 | Default format when format:: omitted | Default Behavior |
| FM-5 | Horizontal rule separators between entries | Structure |

## Test Coverage Summary

- Structure: 3 tests (FM-1, FM-2, FM-5)
- Content Blocks: 1 test (FM-3)
- Default Behavior: 1 test (FM-4)

**Total:** 5 cases

## Test Cases

---

### FM-1: Header block present with session metadata

**Command:**
```
clg .export session_id::{id} output::{file}.md
```

**Expected behavior:**
- Output file starts with `# Session: {session_id}`
- File contains `**Path**:` line with storage path
- File contains `**Entries**:` line with entry count
- File contains `**Created**:` and `**Last Updated**:` timestamp lines

---

### FM-2: Entry headings use H2 with sequential numbering

**Command:**
```
clg .export session_id::{id} output::{file}.md
```

**Expected behavior:**
- First user entry heading is `## Entry 1 - User`
- Second assistant entry heading is `## Entry 2 - Assistant`
- Entry numbering is sequential starting at 1
- Role label is capitalized (`User`, `Assistant`)

---

### FM-3: Thinking block rendered as collapsible details

**Command:**
```
clg .export session_id::{id_with_thinking} output::{file}.md
```

**Expected behavior:**
- Output contains `<details>` and `</details>` tags
- Summary line contains `Thinking (` and `tokens)`
- Thinking content appears inside the details block
- Non-thinking text content appears outside the details block

---

### FM-4: Default format when format:: omitted

**Command:**
```
clg .export session_id::{id} output::{file}
```

**Expected behavior:**
- Output file contains `# Session:` header (Markdown format applied)
- Output file contains `## Entry` headings
- File is human-readable Markdown — not JSON, not plain text

---

### FM-5: Horizontal rule separators between entries

**Command:**
```
clg .export session_id::{id} output::{file}.md
```

**Expected behavior:**
- File contains `---` separator after the header metadata block
- File contains `---` separator between consecutive entries
- No entry immediately follows another without a separator
