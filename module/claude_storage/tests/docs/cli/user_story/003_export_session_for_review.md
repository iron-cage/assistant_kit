# User Story :: 3. Export Session for Review

Acceptance criteria tests for the developer persona exporting a session transcript.
Source: [003_export_session_for_review.md](../../../../docs/cli/user_story/003_export_session_for_review.md)

## Test Case Index

| ID | Test Name | Criteria |
|----|-----------|---------|
| RWS-1 | Export as markdown writes output file | AC: export a specific session by ID to a named file |
| RWS-2 | Export as JSON produces JSONL output | AC: choose between markdown, JSON, and plain text |
| RWS-3 | Export as text produces plain text transcript | AC: choose between markdown, JSON, and plain text |
| RWS-4 | Missing session_id exits with error | AC: required params enforced |
| RWS-5 | Missing output exits with error | AC: required params enforced |

---

### RWS-1: Export as markdown writes output file

**Scenario:** Developer exports a session to a markdown file for offline review.

**Fixture:** One project with a session containing 2 user and 2 assistant entries.

**Command:**
```bash
clg .export session_id::-default_topic output::/tmp/session-test.md
```

**Expected:**
- `/tmp/session-test.md` is created
- File contains markdown-formatted session content
- User and assistant entries are distinguishable in the output

**Exit:** `0`

---

### RWS-2: Export as JSON produces JSONL output

**Scenario:** Developer exports a session as JSON for programmatic processing.

**Fixture:** One project with a session containing at least 1 entry.

**Command:**
```bash
clg .export session_id::-default_topic format::json output::/tmp/session-test.json
```

**Expected:**
- `/tmp/session-test.json` is created
- File is valid JSONL (one JSON object per line)
- Each line is parseable with `jq`

**Exit:** `0`

---

### RWS-3: Export as text produces plain text transcript

**Scenario:** Developer exports a session as plain text for piping to other tools.

**Fixture:** One project with a session containing 1 user and 1 assistant entry.

**Command:**
```bash
clg .export session_id::-default_topic format::text output::/tmp/session-test.txt
```

**Expected:**
- `/tmp/session-test.txt` is created
- File contains plain text without markdown formatting
- Content is human-readable transcript

**Exit:** `0`

---

### RWS-4: Missing session_id exits with error

**Scenario:** Developer accidentally omits the required `session_id::` parameter.

**Fixture:** Any project in storage.

**Command:**
```bash
clg .export output::/tmp/out.md
```

**Expected:**
- Stderr contains an error message indicating `session_id::` is required
- No output file is created

**Exit:** `1`

---

### RWS-5: Missing output exits with error

**Scenario:** Developer accidentally omits the required `output::` parameter.

**Fixture:** One project with a session.

**Command:**
```bash
clg .export session_id::-default_topic
```

**Expected:**
- Stderr contains an error message indicating `output::` is required
- No file is written

**Exit:** `1`
