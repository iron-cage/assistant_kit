# Parameter :: `metadata::`

Edge case tests for the `metadata::` parameter. Tests validate boolean enforcement and content suppression in `.show`.

**Source:** [params.md#parameter--6-metadata](../../../../../docs/cli/params.md#parameter--6-metadata)

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

## Test Cases

### EC-1: Value 0 shows conversation content

**Goal:** Verify that `metadata::0` causes `.show` to display the conversation content (messages) rather than suppressing it.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .show session_id::-default_topic metadata::0`
**Expected Output:** stdout contains conversation message content (user and assistant message text from the session).
**Verification:**
- Exit code is 0
- Output contains message text content from the session
- Content is not suppressed or replaced by technical fields only
**Pass Criteria:** exit 0 + conversation content present in output
**Source:** [params.md](../../../../../docs/cli/params.md)

### EC-2: Value 1 suppresses content, shows metadata

**Goal:** Verify that `metadata::1` suppresses conversation content and instead shows only technical session metadata.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .show session_id::-default_topic metadata::1`
**Expected Output:** stdout contains technical metadata fields (session ID, entry count, timestamps, token usage) without conversation message text.
**Verification:**
- Exit code is 0
- Output contains metadata fields (e.g., entry count, session ID, timestamps)
- Output does not contain conversation message body text from the session
**Pass Criteria:** exit 0 + metadata fields present, conversation content absent
**Source:** [params.md](../../../../../docs/cli/params.md)

### EC-3: Value "true" rejected

**Goal:** Verify that `metadata::true` is rejected because only `0` and `1` are valid boolean values.
**Setup:** None
**Command:** `clg .show session_id::-default_topic metadata::true`
**Expected Output:** stderr contains an error indicating `metadata` must be 0 or 1.
**Verification:**
- Exit code is 1
- stderr contains a message like `metadata must be 0 or 1`
**Pass Criteria:** exit 1 + error message indicating non-boolean value rejected
**Source:** [params.md](../../../../../docs/cli/params.md)

### EC-4: Omitted defaults to 0 (show content)

**Goal:** Verify that omitting `metadata::` causes `.show` to display conversation content (same as `metadata::0`).
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .show session_id::-default_topic`
**Expected Output:** stdout contains conversation message content, identical to running with explicit `metadata::0`.
**Verification:**
- Exit code is 0
- Output matches the result of `clg .show session_id::-default_topic metadata::0`
- Conversation content is present (not suppressed)
**Pass Criteria:** exit 0 + conversation content shown (default applied)
**Source:** [params.md](../../../../../docs/cli/params.md)

### EC-5: metadata::1 output includes entry count

**Goal:** Verify that the metadata output produced by `metadata::1` includes the session's entry count.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .show session_id::-default_topic metadata::1`
**Expected Output:** stdout contains a field showing the number of entries in the session (e.g., `entries: 12` or similar).
**Verification:**
- Exit code is 0
- Output contains a numeric entry count value
- The count is consistent with the fixture session's actual entry count
**Pass Criteria:** exit 0 + entry count present in metadata output
**Source:** [params.md](../../../../../docs/cli/params.md)

### EC-6: metadata::1 output includes session timestamps

**Goal:** Verify that the metadata output produced by `metadata::1` includes first and last entry timestamps.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .show session_id::-default_topic metadata::1`
**Expected Output:** stdout contains timestamp fields for the session's first and last entries (ISO 8601 format or similar).
**Verification:**
- Exit code is 0
- Output contains at least one timestamp value in a recognizable date/time format
- First and/or last timestamp fields are present
**Pass Criteria:** exit 0 + timestamp fields present in metadata output
**Source:** [params.md](../../../../../docs/cli/params.md)
