# Command :: `.export`

Integration tests for the `.export` command. Tests verify required parameter enforcement, format output, and file write behavior.

**Source:** [commands.md#command--6-export](../../../../../docs/cli/commands.md#command--6-export)

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

### IT-1: session_id:: required — missing arg exits with 1

**Goal:** Verify `.export` without `session_id::` exits with code `1` and emits an argument error.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .export output::/tmp/out.md`
**Expected Output:** Error message on stderr indicating `session_id::` is required; no file written.
**Verification:**
- `$?` is `1`
- stderr contains an error message referencing missing `session_id` parameter
- `/tmp/out.md` is not created
**Pass Criteria:** exit 1 + error message indicating `session_id::` is required

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-2: output:: required — missing arg exits with 1

**Goal:** Verify `.export` without `output::` exits with code `1` and emits an argument error.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session `-default_topic` exists)
**Command:** `clg .export session_id::-default_topic`
**Expected Output:** Error message on stderr indicating `output::` is required; no file written.
**Verification:**
- `$?` is `1`
- stderr contains an error message referencing missing `output` parameter
- no file is written to disk
**Pass Criteria:** exit 1 + error message indicating `output::` is required

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-3: Default format is markdown

**Goal:** Verify that omitting `format::` produces a markdown-formatted export file.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session `-default_topic` with known content)
**Command:** `clg .export session_id::-default_topic output::/tmp/export-test.md`
**Expected Output:** `/tmp/export-test.md` is created; file content contains markdown formatting (headings with `#`, or other markdown syntax).
**Verification:**
- `/tmp/export-test.md` exists after command completes
- file content contains markdown syntax (e.g., lines starting with `#` or `**`)
- file is not empty
- stderr is empty
**Pass Criteria:** exit 0 + output file contains markdown formatting

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-4: format::json produces JSON array output

**Goal:** Verify `format::json` exports the session as a JSON array that is valid JSON.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session `-default_topic` with at least 2 entries)
**Command:** `clg .export session_id::-default_topic format::json output::/tmp/export-test.json`
**Expected Output:** `/tmp/export-test.json` is created; file content is valid JSON (array of entry objects).
**Verification:**
- `/tmp/export-test.json` exists after command completes
- `python3 -m json.tool /tmp/export-test.json` exits with code 0 (valid JSON)
- the top-level JSON structure is an array
- array elements represent session entries
**Pass Criteria:** exit 0 + output file is valid JSON array

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-5: format::text produces plain transcript

**Goal:** Verify `format::text` exports the session as a plain text transcript with no markdown or JSON syntax.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session `-default_topic` with known user/assistant messages)
**Command:** `clg .export session_id::-default_topic format::text output::/tmp/export-test.txt`
**Expected Output:** `/tmp/export-test.txt` is created; file content is plain text without markdown heading characters or JSON braces.
**Verification:**
- `/tmp/export-test.txt` exists after command completes
- file content does not begin with `{` or `[` (not JSON)
- file content does not contain lines starting with `#` used as markdown headings
- file is non-empty and readable as plain text
**Pass Criteria:** exit 0 + output file is plain text transcript

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-6: Output file is created at output:: path

**Goal:** Verify the export command creates the output file at exactly the path specified by `output::`.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session `-default_topic`); ensure `/tmp/export-new.md` does not exist before test.
**Command:** `clg .export session_id::-default_topic output::/tmp/export-new.md`
**Expected Output:** `/tmp/export-new.md` is created after command runs.
**Verification:**
- `/tmp/export-new.md` does not exist before command runs
- `/tmp/export-new.md` exists after command runs
- file is non-empty
- `$?` is `0`
**Pass Criteria:** exit 0 + file created at specified output path

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-7: Output file is overwritten if exists

**Goal:** Verify the export command silently overwrites an existing file at the `output::` path without error.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`; pre-create `/tmp/export-overwrite.md` with content `old content`.
**Command:** `clg .export session_id::-default_topic output::/tmp/export-overwrite.md`
**Expected Output:** `/tmp/export-overwrite.md` is overwritten with the exported session content; `old content` is gone.
**Verification:**
- command exits with `0` (no error for pre-existing file)
- `/tmp/export-overwrite.md` content is the exported session, not `old content`
- stderr is empty (no overwrite warning)
**Pass Criteria:** exit 0 + file overwritten silently with new content

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-8: Exit code 2 when output parent dir does not exist

**Goal:** Verify `.export` exits with code `2` when the parent directory of the `output::` path does not exist.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session `-default_topic` exists); `/tmp/nonexistent-dir/` does not exist.
**Command:** `clg .export session_id::-default_topic output::/tmp/nonexistent-dir/out.md`
**Expected Output:** Error message on stderr indicating the output directory does not exist; no file created.
**Verification:**
- `$?` is `2`
- stderr contains an error message referencing the write failure or missing directory
- `/tmp/nonexistent-dir/out.md` is not created
**Pass Criteria:** exit 2 + error message on stderr for unwritable output path

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-9: project:: selects session from named project

**Goal:** Verify `project::` scopes session lookup to the specified project when exporting.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project `alpha` with session `-default_topic`; project `beta` with a different session `-default_topic` containing different content)
**Command:** `clg .export session_id::-default_topic project::alpha output::/tmp/export-alpha.md`
**Expected Output:** `/tmp/export-alpha.md` contains content from project `alpha`'s `-default_topic` session, not `beta`'s.
**Verification:**
- `/tmp/export-alpha.md` exists and is non-empty
- content matches the fixture data for `alpha/-default_topic`, not `beta/-default_topic`
- stderr is empty
**Pass Criteria:** exit 0 + exported content is scoped to project `alpha`

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-10: Export succeeds with valid session and output path

**Goal:** Verify the full happy-path export succeeds with all required parameters provided.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project `alpha`, session `-default_topic` with 3 entries)
**Command:** `clg .export session_id::-default_topic output::/tmp/export-happy.md`
**Expected Output:** `/tmp/export-happy.md` created with non-empty session content; command exits cleanly.
**Verification:**
- `$?` is `0`
- `/tmp/export-happy.md` exists and is non-empty
- file contains recognizable session content (not an error message)
- stderr is empty
**Pass Criteria:** exit 0 + output file created with session content

**Source:** [commands.md](../../../../../docs/cli/commands.md)
