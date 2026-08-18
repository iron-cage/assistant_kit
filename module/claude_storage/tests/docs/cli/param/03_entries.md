# Parameter :: `show_entries::`

Edge case tests for the `show_entries::` parameter. Tests validate boolean enforcement and its three distinct contexts in `.show`: a no-op in session-detail content mode, a nested entry-list toggle inside session-detail metadata mode, and a rendering toggle in project-overview mode.

**Source:** [param/03_entries.md](../../../../docs/cli/param/03_entries.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Value 0, session-detail content mode → full formatted content | Content-Mode No-Op |
| EC-2 | Value 1, session-detail content mode → still full formatted content | Content-Mode No-Op |
| EC-3 | Value "yes" rejected | Type Validation |
| EC-4 | show_metadata::1 + show_entries::0 (default) → metadata only, no entry list | Metadata-Mode Behavior |
| EC-5 | show_metadata::1 + show_entries::1 → metadata + raw entry list | Metadata-Mode Behavior |
| EC-6 | show_metadata::1 + show_entries::1 entry list includes UUID and timestamp | Output Format |
| EC-7 | show_entries::0 (default), project overview → formatted tail messages | Project-Overview Behavior |
| EC-8 | show_entries::1, project overview → tail window as raw list | Project-Overview Behavior |

## Test Coverage Summary

- Content-Mode No-Op: 2 tests (EC-1, EC-2)
- Type Validation: 1 test (EC-3)
- Metadata-Mode Behavior: 2 tests (EC-4, EC-5)
- Output Format: 1 test (EC-6)
- Project-Overview Behavior: 2 tests (EC-7, EC-8)

**Total:** 8 edge cases

**Behavioral Divergence Pair:** EC-2 (content mode, no-op) ↔ EC-5 (metadata mode, appends list) — same `show_entries::1` value, different effect depending on `show_metadata::`.

**Behavioral Divergence Pair:** EC-7 (project overview, formatted) ↔ EC-8 (project overview, raw list)

## Test Cases

---

### EC-1: Value 0, session-detail content mode → full formatted content

- **Commands:** `.show`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session `-default_topic` with 4 known entries)
- **When:** `clg .show session_id::-default_topic show_entries::0`
- **Then:** stdout contains the full conversation as formatted chat content (`[timestamp] Role:` + message body for all 4 entries) — content mode's baseline, unaffected by `show_entries::`
- **Exit:** 0
- **Source:** [param/03_entries.md](../../../../docs/cli/param/03_entries.md)

---

### EC-2: Value 1, session-detail content mode → still full formatted content

- **Commands:** `.show`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session `-default_topic` with 4 known entries)
- **When:** `clg .show session_id::-default_topic show_entries::1`
- **Then:** stdout is byte-identical to EC-1's output — `show_entries::` has no effect in content mode (no `show_metadata::1`); content mode always shows full formatted entry content regardless of this flag
- **Exit:** 0
- **Source:** [param/03_entries.md](../../../../docs/cli/param/03_entries.md)

---

### EC-3: Value "yes" rejected

- **Commands:** `.show`
- **Given:** clean environment
- **When:** `clg .show session_id::-default_topic show_entries::yes`
- **Then:** stderr contains an error indicating `entries` must be 0 or 1
- **Exit:** 1
- **Source:** [param/03_entries.md](../../../../docs/cli/param/03_entries.md)

---

### EC-4: show_metadata::1 + show_entries::0 (default) → metadata only, no entry list

- **Commands:** `.show`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session `-default_topic` with 4 known entries)
- **When:** `clg .show session_id::-default_topic show_metadata::1`
- **Then:** stdout contains metadata fields (entry count, session type, timestamps) only; no per-entry list, no message content
- **Exit:** 0
- **Source:** [param/03_entries.md](../../../../docs/cli/param/03_entries.md)

---

### EC-5: show_metadata::1 + show_entries::1 → metadata + raw entry list

- **Commands:** `.show`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session `-default_topic` with 4 known entries)
- **When:** `clg .show session_id::-default_topic show_metadata::1 show_entries::1`
- **Then:** stdout contains the metadata fields from EC-4, followed by a raw numbered list of all 4 entries; no formatted message content
- **Exit:** 0
- **Source:** [param/03_entries.md](../../../../docs/cli/param/03_entries.md)

---

### EC-6: show_metadata::1 + show_entries::1 entry list includes UUID and timestamp

- **Commands:** `.show`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session `-default_topic` with 4 known entries)
- **When:** `clg .show session_id::-default_topic show_metadata::1 show_entries::1`
- **Then:** Each line of the appended entry list includes a UUID-format string, an entry type, and a timestamp string
- **Exit:** 0
- **Source:** [param/03_entries.md](../../../../docs/cli/param/03_entries.md)

---

### EC-7: show_entries::0 (default), project overview → formatted tail messages

- **Commands:** `.show`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project with 3 sessions, run from its cwd)
- **When:** `clg .show`
- **Then:** Summary block followed by the last `tail::` messages rendered as formatted chat content (default — `show_entries::0`)
- **Exit:** 0
- **Source:** [param/03_entries.md](../../../../docs/cli/param/03_entries.md)

---

### EC-8: show_entries::1, project overview → tail window as raw list

- **Commands:** `.show`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project with 3 sessions, run from its cwd)
- **When:** `clg .show show_entries::1`
- **Then:** Summary block followed by the same `tail::`-windowed entries, rendered as a raw UUID/type/timestamp list instead of formatted chat content
- **Exit:** 0
- **Source:** [param/03_entries.md](../../../../docs/cli/param/03_entries.md)
