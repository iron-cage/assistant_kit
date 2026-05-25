# Parameter :: `entries::`

Edge case tests for the `entries::` parameter. Tests validate boolean enforcement and display impact in `.show`.

**Source:** [param/03_entries.md](../../../../docs/cli/param/03_entries.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Value 0 shows summary view | Behavior |
| EC-2 | Value 1 shows all entry records | Behavior |
| EC-3 | Value "yes" rejected | Type Validation |
| EC-4 | Omitted defaults to 0 (summary view) | Default |
| EC-5 | entries::1 with small session shows all entries | Behavior |
| EC-6 | entries::1 output includes UUID and timestamp per entry | Output Format |

## Test Coverage Summary

- Behavior: 3 tests (EC-1, EC-2, EC-5)
- Type Validation: 1 test (EC-3)
- Default: 1 test (EC-4)
- Output Format: 1 test (EC-6)

**Total:** 6 edge cases

**Behavioral Divergence Pair:** EC-1 (entries::0, summary view) ↔ EC-2 (entries::1, full records)

## Test Cases

---

### EC-1: Value 0 shows summary view

- **Commands:** `.show`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .show session_id::-default_topic entries::0`
- **Then:** stdout contains a concise summary of the session (entry count, timestamps) without listing each individual message record.; summary output without per-entry records
- **Exit:** 0
- **Source:** [param/03_entries.md](../../../../docs/cli/param/03_entries.md)

---

### EC-2: Value 1 shows all entry records

- **Commands:** `.show`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .show session_id::-default_topic entries::1`
- **Then:** stdout lists each entry in the session with its record details (UUID, type, timestamp).; individual entry records listed in output
- **Exit:** 0
- **Source:** [param/03_entries.md](../../../../docs/cli/param/03_entries.md)

---

### EC-3: Value "yes" rejected

- **Commands:** `.show`
- **Given:** clean environment
- **When:** `clg .show session_id::-default_topic entries::yes`
- **Then:** stderr contains an error indicating `entries` must be 0 or 1.; error message indicating non-boolean value rejected
- **Exit:** 1
- **Source:** [param/03_entries.md](../../../../docs/cli/param/03_entries.md)

---

### EC-4: Omitted defaults to 0 (summary view)

- **Commands:** `.show`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .show session_id::-default_topic`
- **Then:** stdout contains the session summary, identical to running with explicit `entries::0`.; summary view shown (default applied, no per-entry expansion)
- **Exit:** 0
- **Source:** [param/03_entries.md](../../../../docs/cli/param/03_entries.md)

---

### EC-5: entries::1 with small session shows all entries

- **Commands:** `.show`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture session with a known small number of entries, e.g., 3)
- **When:** `clg .show session_id::-default_topic entries::1`
- **Then:** stdout lists exactly the number of entry records that exist in the session (e.g., 3 records for a 3-entry session).; entry record count in output equals the fixture session's actual entry count
- **Exit:** 0
- **Source:** [param/03_entries.md](../../../../docs/cli/param/03_entries.md)

---

### EC-6: entries::1 output includes UUID and timestamp per entry

- **Commands:** `.show`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .show session_id::-default_topic entries::1`
- **Then:** stdout contains entry records where each line (or block) includes a UUID-format string and a timestamp string.; UUID and timestamp fields present in per-entry output
- **Exit:** 0
- **Source:** [param/03_entries.md](../../../../docs/cli/param/03_entries.md)
