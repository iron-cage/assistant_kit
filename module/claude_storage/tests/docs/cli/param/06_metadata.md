# Parameter :: `metadata::`

Edge case tests for the `metadata::` parameter. Tests validate boolean enforcement and content suppression in `.show`.

**Source:** [params.md#parameter--6-metadata](../../../../docs/cli/params.md#parameter--6-metadata)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Value 0 shows conversation content | Behavior |
| EC-2 | Value 1 suppresses content, shows metadata | Behavior |
| EC-3 | Value "true" rejected | Type Validation |
| EC-4 | Omitted defaults to 0 (show content) | Default |
| EC-5 | metadata::1 output includes entry count | Output Format |
| EC-6 | metadata::1 output includes session timestamps | Output Format |

## Test Coverage Summary

- Behavior: 2 tests (EC-1, EC-2)
- Type Validation: 1 test (EC-3)
- Default: 1 test (EC-4)
- Output Format: 2 tests (EC-5, EC-6)

**Total:** 6 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

## Test Cases

---

### EC-1: Value 0 shows conversation content

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .show session_id::-default_topic metadata::0`
- **Then:** stdout contains conversation message content (user and assistant message text from the session).; conversation content present in output
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-2: Value 1 suppresses content, shows metadata

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .show session_id::-default_topic metadata::1`
- **Then:** stdout contains technical metadata fields (session ID, entry count, timestamps, token usage) without conversation message text.; metadata fields present, conversation content absent
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-3: Value "true" rejected

- **Given:** clean environment
- **When:** `clg .show session_id::-default_topic metadata::true`
- **Then:** stderr contains an error indicating `metadata` must be 0 or 1.; error message indicating non-boolean value rejected
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-4: Omitted defaults to 0 (show content)

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .show session_id::-default_topic`
- **Then:** stdout contains conversation message content, identical to running with explicit `metadata::0`.; conversation content shown (default applied)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-5: metadata::1 output includes entry count

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .show session_id::-default_topic metadata::1`
- **Then:** stdout contains a field showing the number of entries in the session (e.g., `entries: 12` or similar).; entry count present in metadata output
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-6: metadata::1 output includes session timestamps

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .show session_id::-default_topic metadata::1`
- **Then:** stdout contains timestamp fields for the session's first and last entries (ISO 8601 format or similar).; timestamp fields present in metadata output
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)
