# Parameter :: `entries::`

Edge case tests for the `entries::` parameter. Tests validate boolean enforcement and display impact in `.show`.

**Source:** [params.md#parameter--3-entries](../../../../../docs/cli/params.md#parameter--3-entries)

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

## Test Cases

### EC-1: Value 0 shows summary view

**Goal:** Verify that `entries::0` causes `.show` to display a summary view rather than expanding individual entries.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .show session_id::-default_topic entries::0`
**Expected Output:** stdout contains a concise summary of the session (entry count, timestamps) without listing each individual message record.
**Verification:**
- Exit code is 0
- Output does not list individual entry UUIDs or per-entry timestamps
- Output contains a summary (e.g., entry count, session ID)
**Pass Criteria:** exit 0 + summary output without per-entry records
**Source:** [params.md](../../../../../docs/cli/params.md)

### EC-2: Value 1 shows all entry records

**Goal:** Verify that `entries::1` causes `.show` to list every entry record in the session.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .show session_id::-default_topic entries::1`
**Expected Output:** stdout lists each entry in the session with its record details (UUID, type, timestamp).
**Verification:**
- Exit code is 0
- Output contains multiple entry lines (one per session entry)
- Each entry line includes identifying information
**Pass Criteria:** exit 0 + individual entry records listed in output
**Source:** [params.md](../../../../../docs/cli/params.md)

### EC-3: Value "yes" rejected

**Goal:** Verify that `entries::yes` is rejected because only `0` and `1` are valid boolean values.
**Setup:** None
**Command:** `clg .show session_id::-default_topic entries::yes`
**Expected Output:** stderr contains an error indicating `entries` must be 0 or 1.
**Verification:**
- Exit code is 1
- stderr contains a message like `entries must be 0 or 1`
**Pass Criteria:** exit 1 + error message indicating non-boolean value rejected
**Source:** [params.md](../../../../../docs/cli/params.md)

### EC-4: Omitted defaults to 0 (summary view)

**Goal:** Verify that omitting `entries::` causes `.show` to use the summary view (same as `entries::0`).
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .show session_id::-default_topic`
**Expected Output:** stdout contains the session summary, identical to running with explicit `entries::0`.
**Verification:**
- Exit code is 0
- Output matches the result of `clg .show session_id::-default_topic entries::0`
- No per-entry records are expanded
**Pass Criteria:** exit 0 + summary view shown (default applied, no per-entry expansion)
**Source:** [params.md](../../../../../docs/cli/params.md)

### EC-5: entries::1 with small session shows all entries

**Goal:** Verify that for a session with a small number of entries, `entries::1` outputs the correct count of entry records.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture session with a known small number of entries, e.g., 3)
**Command:** `clg .show session_id::-default_topic entries::1`
**Expected Output:** stdout lists exactly the number of entry records that exist in the session (e.g., 3 records for a 3-entry session).
**Verification:**
- Exit code is 0
- Line count of entry records in output matches the known entry count of the test fixture session
**Pass Criteria:** exit 0 + entry record count in output equals the fixture session's actual entry count
**Source:** [params.md](../../../../../docs/cli/params.md)

### EC-6: entries::1 output includes UUID and timestamp per entry

**Goal:** Verify that the per-entry output format produced by `entries::1` includes both the UUID and a timestamp for each entry.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .show session_id::-default_topic entries::1`
**Expected Output:** stdout contains entry records where each line (or block) includes a UUID-format string and a timestamp string.
**Verification:**
- Exit code is 0
- At least one output line contains a UUID-format string (e.g., `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`)
- At least one output line contains a timestamp (ISO 8601 format or similar)
**Pass Criteria:** exit 0 + UUID and timestamp fields present in per-entry output
**Source:** [params.md](../../../../../docs/cli/params.md)
