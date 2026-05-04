# Command :: `.export`

Integration tests for the `.export` command. Tests verify required parameter enforcement, format output, and file write behavior.

**Source:** [commands.md#command--6-export](../../../../docs/cli/commands.md#command--6-export)

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

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .export output::/tmp/out.md`
- **Then:** Error message on stderr indicating `session_id::` is required; no file written.; error message indicating `session_id::` is required
- **Exit:** 1
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-2: output:: required — missing arg exits with 1

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session `-default_topic` exists)
- **When:** `clg .export session_id::-default_topic`
- **Then:** Error message on stderr indicating `output::` is required; no file written.; + error message indicating `output::` is required
- **Exit:** 1
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-3: Default format is markdown

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session `-default_topic` with known content)
- **When:** `clg .export session_id::-default_topic output::/tmp/export-test.md`
- **Then:** `/tmp/export-test.md` is created; file content contains markdown formatting (headings with `#`, or other markdown syntax).; + output file contains markdown formatting
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-4: format::json produces JSON array output

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session `-default_topic` with at least 2 entries)
- **When:** `clg .export session_id::-default_topic format::json output::/tmp/export-test.json`
- **Then:** `/tmp/export-test.json` is created; file content is valid JSON (array of entry objects).; + output file is valid JSON array
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-5: format::text produces plain transcript

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session `-default_topic` with known user/assistant messages)
- **When:** `clg .export session_id::-default_topic format::text output::/tmp/export-test.txt`
- **Then:** `/tmp/export-test.txt` is created; file content is plain text without markdown heading characters or JSON braces.; + output file is plain text transcript
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-6: Output file is created at output:: path

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session `-default_topic`); ensure `/tmp/export-new.md` does not exist before test.
- **When:** `clg .export session_id::-default_topic output::/tmp/export-new.md`
- **Then:** `/tmp/export-new.md` is created after command runs.; + file created at specified output path
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-7: Output file is overwritten if exists

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`; pre-create `/tmp/export-overwrite.md` with content `old content`.
- **When:** `clg .export session_id::-default_topic output::/tmp/export-overwrite.md`
- **Then:** `/tmp/export-overwrite.md` is overwritten with the exported session content; `old content` is gone.; + file overwritten silently with new content
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-8: Exit code 2 when output parent dir does not exist

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session `-default_topic` exists); `/tmp/nonexistent-dir/` does not exist.
- **When:** `clg .export session_id::-default_topic output::/tmp/nonexistent-dir/out.md`
- **Then:** Error message on stderr indicating the output directory does not exist; no file created.; + error message on stderr for unwritable output path
- **Exit:** 2
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-9: project:: selects session from named project

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project `alpha` with session `-default_topic`; project `beta` with a different session `-default_topic` containing different content)
- **When:** `clg .export session_id::-default_topic project::alpha output::/tmp/export-alpha.md`
- **Then:** `/tmp/export-alpha.md` contains content from project `alpha`'s `-default_topic` session, not `beta`'s.; + exported content is scoped to project `alpha`
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-10: Export succeeds with valid session and output path

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project `alpha`, session `-default_topic` with 3 entries)
- **When:** `clg .export session_id::-default_topic output::/tmp/export-happy.md`
- **Then:** `/tmp/export-happy.md` created with non-empty session content; command exits cleanly.; + output file created with session content
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)
