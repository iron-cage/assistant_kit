# Command :: `.show`

Integration tests for the `.show` command. Tests verify project view, session view, location-aware behavior, and display modes.

**Source:** [command/03_show.md](../../../../docs/cli/command/03_show.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| INT-1 | No args shows current project's overview | Location-Aware |
| INT-2 | session_id:: shows conversation content | Session View |
| INT-3 | project:: selects explicit project | Project View |
| INT-4 | session_id:: + project:: shows session in named project | Combined |
| INT-5 | show_metadata::1 suppresses content, shows metadata | Display Mode |
| INT-6 | show_metadata::1 + show_entries::1 shows raw entry list | Display Mode |
| INT-7 | Exit code 1 when cwd has no project | Exit Codes |
| INT-8 | project:: with path-encoded ID | Project View |
| INT-12 | show_entries::1 alone is a no-op in session-detail content mode | Display Mode |
| INT-13 | fields::single-field shows just that field for every entry | Field Projection |
| INT-14 | fields::multi-field shows requested fields in request order | Field Projection |
| INT-15 | fields::all shows every one of the 18 fields, including ones content mode drops | Field Projection |
| INT-16 | fields:: with an invalid token is rejected | Exit Codes |
| INT-17 | index::N narrows session-detail rendering to exactly one message | Message Selection |
| INT-18 | index:: beyond the entry count is rejected | Exit Codes |
| INT-19 | fields:: + index:: composed project one message's requested attributes | Field Projection |
| INT-20 | fields:: applies to the project-overview tail window, not just session-detail | Field Projection |
| INT-24 | A user entry holding only a successful tool result is named, not left blank | Content Rendering |

## Test Coverage Summary

- Location-Aware: 1 test (INT-1)
- Session View: 1 test (INT-2)
- Project View: 2 tests (INT-3, INT-8)
- Combined: 1 test (INT-4)
- Display Mode: 3 tests (INT-5, INT-6, INT-12)
- Exit Codes: 3 tests (INT-7, INT-16, INT-18)
- Field Projection: 5 tests (INT-13, INT-14, INT-15, INT-19, INT-20)
- Content Rendering: 1 test (INT-24)
- Message Selection: 1 test (INT-17)

> **Known gap:** this catalog's `INT-N` numbering stops short of the `T01`–`T18` and later cases actually present in `tests/cli_cmd_show_test.rs` (added by tasks 513/525/526) — pre-existing staleness, not introduced by the `fields::`/`index::` additions below (`INT-13`–`INT-20`). Out of scope for this change; flagged here rather than silently left implicit.

## Test Cases

---

### INT-1: No args shows current project's overview

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .show
```

**Expected behavior:**
- Fixture: a project whose path-encoding matches the test's cwd, with 3 sessions; run from that cwd
- A summary block (project path, storage dir, session counts by type, total entries) followed by the full list of all sessions in the project (one line per session: ID, entry count, last-activity timestamp) — unconditional, no tail-window capping, no per-session-list gating
- Exit code: 0
- **Source:** [command/03_show.md](../../../../docs/cli/command/03_show.md)

---

### INT-2: session_id:: shows conversation content

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .show session_id::-default_topic
```

**Expected behavior:**
- Fixture: project `alpha` with session `-default_topic` containing known messages
- Session summary or content for session `-default_topic`; includes session ID in output
- Exit code: 0
- **Source:** [command/03_show.md](../../../../docs/cli/command/03_show.md)

---

### INT-3: project:: selects explicit project

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .show project::alpha
```

**Expected behavior:**
- Fixture: projects `alpha` and `beta`; run from a cwd that does not correspond to either project
- Session list for project `alpha`; no sessions from `beta` or any cwd-resolved project
- Exit code: 0
- **Source:** [command/03_show.md](../../../../docs/cli/command/03_show.md)

---

### INT-4: session_id:: + project:: shows session in named project

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .show session_id::s1 project::alpha
```

**Expected behavior:**
- Fixture: project `alpha` with session `s1`; project `beta` with a different session `s1`
- Content or summary for session `s1` from project `alpha` specifically, not `s1` from `beta`
- Exit code: 0
- **Source:** [command/03_show.md](../../../../docs/cli/command/03_show.md)

---

### INT-5: show_metadata::1 suppresses content, shows metadata only

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .show session_id::-default_topic show_metadata::1
```

**Expected behavior:**
- Fixture: session `-default_topic` with known user/assistant messages
- Metadata fields (e.g., entry count, session type, timestamps) present; actual message text absent
- Exit code: 0
- **Source:** [command/03_show.md](../../../../docs/cli/command/03_show.md)

---

### INT-6: show_metadata::1 + show_entries::1 shows raw entry list

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .show session_id::-default_topic show_metadata::1 show_entries::1
```

**Expected behavior:**
- Fixture: session `-default_topic` with 4 known entries: 2 user, 2 assistant
- Metadata fields (entry count, session type, timestamps) present, followed by a raw numbered list of all 4 entries (UUID, type, timestamp) — no formatted message content
- Exit code: 0
- **Source:** [command/03_show.md](../../../../docs/cli/command/03_show.md)

---

### INT-7: Exit code 1 when cwd has no project

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .show
```

**Expected behavior:**
- Fixture: run from a directory (e.g., `/tmp`) that has no matching storage project
- Error message on stderr indicating the current directory has no project in storage
- Exit code: 1
- **Source:** [command/03_show.md](../../../../docs/cli/command/03_show.md)

---

### INT-8: project:: with path-encoded ID

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .show project::-home-alice-projects-alpha
```

**Expected behavior:**
- Fixture: project stored with path-encoded ID `-home-alice-projects-alpha`, with 3 sessions
- Summary block followed by the full list of all sessions for the project with path-encoded ID `-home-alice-projects-alpha`
- Exit code: 0
- **Source:** [command/03_show.md](../../../../docs/cli/command/03_show.md)

---

### INT-12: show_entries::1 alone is a no-op in session-detail content mode

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .show session_id::-default_topic show_entries::1
```

**Expected behavior:**
- Fixture: session `-default_topic` with 4 known entries: 2 user, 2 assistant
- Full formatted conversation content for all 4 entries — identical output to the same command without `show_entries::1` (content mode always shows all entries regardless of this flag; it only has effect nested inside `show_metadata::1`, see INT-6)
- Exit code: 0
- **Source:** [command/03_show.md](../../../../docs/cli/command/03_show.md)

---

### INT-13: fields::single-field shows just that field for every entry

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .show session_id::-default_topic fields::timestamp
```

**Expected behavior:**
- Fixture: session `-default_topic` with 3 known entries at distinct timestamps
- Output shows exactly one `timestamp` line per entry (plus the entry header identifying which message it is); no message text, no other attribute
- Exit code: 0
- **Source:** [command/03_show.md](../../../../docs/cli/command/03_show.md); [param/32_fields.md](../param/32_fields.md) EC-1

---

### INT-14: fields::multi-field shows requested fields in request order

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .show session_id::-default_topic fields::model,uuid
```

**Expected behavior:**
- Fixture: session `-default_topic` with a known assistant entry (known `model`, known `uuid`)
- Each entry's block lists the `model` line before the `uuid` line — matching request order, not canonical vocabulary order
- Exit code: 0
- **Source:** [command/03_show.md](../../../../docs/cli/command/03_show.md); [param/32_fields.md](../param/32_fields.md) EC-2

---

### INT-15: fields::all shows every one of the 18 fields, including ones content mode drops

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .show session_id::-default_topic fields::all
```

**Expected behavior:**
- Fixture: session `-default_topic` with one assistant entry carrying a `tool_use` block (known `id`/`name`/`input`) and one successful `tool_result` block, plus one user entry with `thinking_metadata` present
- Output includes every one of the 18 canonical field lines for that entry, including `parent_uuid`, `cwd`, `version`, `git_branch`, `request_id`, the user entry's `thinking_level`/`thinking_disabled`, the tool_use block's `id` and full `input` JSON, and the successful tool_result's `content` — none of which the default chat-log content mode ever prints (see [`../../../../docs/cli/readme.md` § Local Style Conventions](../../../../docs/cli/readme.md))
- Exit code: 0
- **Source:** [command/03_show.md](../../../../docs/cli/command/03_show.md); [param/32_fields.md](../param/32_fields.md) EC-3

---

### INT-16: fields:: with an invalid token is rejected

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .show session_id::-default_topic fields::bogus
```

**Expected behavior:**
- Fixture: any valid session
- stderr contains `unknown field 'bogus' — valid fields: uuid, parent_uuid, timestamp, entry_type, cwd, session_id, version, git_branch, user_type, is_sidechain, content, thinking_level, thinking_disabled, model, message_id, stop_reason, stop_sequence, request_id, or all`; stdout empty
- Exit code: 1
- **Source:** [command/03_show.md](../../../../docs/cli/command/03_show.md); [param/32_fields.md](../param/32_fields.md) EC-4

---

### INT-17: index::N narrows session-detail rendering to exactly one message

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .show session_id::-default_topic index::2
```

**Expected behavior:**
- Fixture: session `-default_topic` with 4 known entries, each with distinguishable content
- Output shows only the 2nd entry's chat-log content; entries 1, 3, 4 absent entirely
- Exit code: 0
- **Source:** [command/03_show.md](../../../../docs/cli/command/03_show.md); [param/33_index.md](../param/33_index.md) EC-1

---

### INT-18: index:: beyond the entry count is rejected

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .show session_id::-default_topic index::99
```

**Expected behavior:**
- Fixture: session `-default_topic` with 4 known entries
- stderr contains `index out of range: 99 (4 entries)`; stdout empty
- Exit code: 1
- **Source:** [command/03_show.md](../../../../docs/cli/command/03_show.md); [param/33_index.md](../param/33_index.md) EC-6

---

### INT-19: fields:: + index:: composed project one message's requested attributes

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .show session_id::-default_topic fields::uuid,model index::3
```

**Expected behavior:**
- Fixture: session `-default_topic` with 4 known entries, entry 3 a known assistant message
- Output shows only entry 3's `uuid` and `model` lines — no other entry, no other field
- Exit code: 0
- **Source:** [command/03_show.md](../../../../docs/cli/command/03_show.md); [param/32_fields.md](../param/32_fields.md) EC-12; [param/33_index.md](../param/33_index.md) EC-9

---

### INT-20: fields:: applies to the project-overview tail window, not just session-detail

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .show fields::timestamp last::5
```

**Expected behavior:**
- Fixture: cwd-resolved project, most-recently-active session with ≥5 known entries
- Project summary block unchanged, followed by field-projection blocks (not chat-log content) for the last 5 entries — proves `fields::` is not session-detail-only
- Exit code: 0
- **Source:** [command/03_show.md](../../../../docs/cli/command/03_show.md); [param/32_fields.md](../param/32_fields.md) EC-10

---

### INT-24: A user entry holding only a successful tool result is named, not left blank

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .show last::2
```

**Expected behavior:**
- Fixture: a raw two-record session — an assistant `tool_use` and the user `tool_result` answering it
- The user entry's `TIMESTAMP · User:` header is followed by `↳ tool result`; the output contains no header standing over a blank line
- Exit code: 0
- `.tail` folds a successful `tool_result` onto the `⚙` line of the call it answers, so it never prints one alone. `.show`'s chat-log view has no call to fold onto — suppressing the block there left two lines that said nothing, and no way to distinguish an empty entry from a broken one
- **Source:** [command/03_show.md](../../../../docs/cli/command/03_show.md)
