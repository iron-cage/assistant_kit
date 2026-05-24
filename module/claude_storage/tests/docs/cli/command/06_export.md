# Command :: `.export`

Integration tests for the `.export` command. Tests verify required parameter enforcement, format output, and file write behavior.

**Source:** [001_commands.md#command--6-export](../../../../docs/cli/001_commands.md#command--6-export)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | session_id:: required — missing arg exits with 1 | Exit Codes |
| IT-2 | output:: required — missing arg exits with 1 | Exit Codes |
| IT-3 | Default format is markdown | Output Format |
| IT-4 | format::json produces JSON array output | Output Format |
| IT-5 | format::text produces plain transcript | Output Format |
| IT-6 | Output file is created at output:: path | File Write |
| IT-7 | Output file is overwritten if exists | File Write |
| IT-8 | Exit code 2 when output parent dir does not exist | Exit Codes |
| IT-9 | project:: selects session from named project | Scoping |
| IT-10 | Export succeeds with valid session and output path | Read Operations |

## Test Coverage Summary

- Exit Codes: 3 tests (IT-1, IT-2, IT-8)
- Output Format: 3 tests (IT-3, IT-4, IT-5)
- File Write: 2 tests (IT-6, IT-7)
- Scoping: 1 test (IT-9)
- Read Operations: 1 test (IT-10)

## Test Cases

---

### IT-1: session_id:: required — missing arg exits with 1

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .export output::/tmp/out.md
```

**Expected behavior:**
- Error message on stderr indicating `session_id::` is required; no file written
- Exit code: 1
- **Source:** [001_commands.md](../../../../docs/cli/001_commands.md)

---

### IT-2: output:: required — missing arg exits with 1

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .export session_id::-default_topic
```

**Expected behavior:**
- Fixture: session `-default_topic` exists in storage
- Error message on stderr indicating `output::` is required; no file written
- Exit code: 1
- **Source:** [001_commands.md](../../../../docs/cli/001_commands.md)

---

### IT-3: Default format is markdown

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .export session_id::-default_topic output::/tmp/export-test.md
```

**Expected behavior:**
- Fixture: session `-default_topic` with known content
- `/tmp/export-test.md` is created; file content contains markdown formatting (headings with `#` or other markdown syntax)
- Exit code: 0
- **Source:** [001_commands.md](../../../../docs/cli/001_commands.md)

---

### IT-4: format::json produces JSON array output

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .export session_id::-default_topic format::json output::/tmp/export-test.json
```

**Expected behavior:**
- Fixture: session `-default_topic` with at least 2 entries
- `/tmp/export-test.json` is created; file content is valid JSON (array of entry objects)
- Exit code: 0
- **Source:** [001_commands.md](../../../../docs/cli/001_commands.md)

---

### IT-5: format::text produces plain transcript

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .export session_id::-default_topic format::text output::/tmp/export-test.txt
```

**Expected behavior:**
- Fixture: session `-default_topic` with known user/assistant messages
- `/tmp/export-test.txt` is created; file content is plain text without markdown heading characters or JSON braces
- Exit code: 0
- **Source:** [001_commands.md](../../../../docs/cli/001_commands.md)

---

### IT-6: Output file is created at output:: path

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .export session_id::-default_topic output::/tmp/export-new.md
```

**Expected behavior:**
- Fixture: session `-default_topic`; `/tmp/export-new.md` does not exist before test
- `/tmp/export-new.md` is created after command runs
- Exit code: 0
- **Source:** [001_commands.md](../../../../docs/cli/001_commands.md)

---

### IT-7: Output file is overwritten if exists

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .export session_id::-default_topic output::/tmp/export-overwrite.md
```

**Expected behavior:**
- Pre-condition: `/tmp/export-overwrite.md` pre-created with content `old content`
- `/tmp/export-overwrite.md` is overwritten with the exported session content; `old content` is gone
- Exit code: 0
- **Source:** [001_commands.md](../../../../docs/cli/001_commands.md)

---

### IT-8: Exit code 2 when output parent dir does not exist

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .export session_id::-default_topic output::/tmp/nonexistent-dir/out.md
```

**Expected behavior:**
- Fixture: session `-default_topic` exists; `/tmp/nonexistent-dir/` does not exist
- Error message on stderr indicating the output directory does not exist; no file created
- Exit code: 2
- **Source:** [001_commands.md](../../../../docs/cli/001_commands.md)

---

### IT-9: project:: selects session from named project

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .export session_id::-default_topic project::alpha output::/tmp/export-alpha.md
```

**Expected behavior:**
- Fixture: project `alpha` with session `-default_topic`; project `beta` with a different session `-default_topic` containing different content
- `/tmp/export-alpha.md` contains content from project `alpha`'s `-default_topic` session, not `beta`'s
- Exit code: 0
- **Source:** [001_commands.md](../../../../docs/cli/001_commands.md)

---

### IT-10: Export succeeds with valid session and output path

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .export session_id::-default_topic output::/tmp/export-happy.md
```

**Expected behavior:**
- Fixture: project `alpha`, session `-default_topic` with 3 entries
- `/tmp/export-happy.md` created with non-empty session content; command exits cleanly
- Exit code: 0
- **Source:** [001_commands.md](../../../../docs/cli/001_commands.md)
