# Parameter Group :: Session Filter

Interaction tests for the Session Filter group (`session::`, `agent::`, `min_entries::`). Tests verify auto-enable behavior, combined filter semantics, and `sessions::` override interactions.

**Source:** [parameter_groups.md#session-filter](../../../../../docs/cli/parameter_groups.md#session-filter)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | session:: alone auto-enables session display | Auto-Enable |
| CC-2 | agent:: alone auto-enables session display | Auto-Enable |
| CC-3 | min_entries:: alone auto-enables session display | Auto-Enable |
| CC-4 | sessions::0 suppresses display even with all three filters | Override Interaction |
| CC-5 | session:: + agent:: combined filters sessions by both | Combined Filter |
| CC-6 | session:: + min_entries:: combined filters by both criteria | Combined Filter |
| CD-1 | All three filters are AND-combined (not OR) | Filter Logic |

## Test Coverage Summary

- Auto-Enable: 3 tests (CC-1, CC-2, CC-3)
- Override Interaction: 1 test (CC-4)
- Combined Filter: 2 tests (CC-5, CC-6)
- Filter Logic: 1 test (CD-1)

## Test Cases

### CC-1: session:: alone auto-enables session display

**Goal:** Verify that providing `session::` without `sessions::1` automatically enables the session list in the output.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project having sessions `-commit` and `-default_topic`.
**Command:** `clg .list session::commit`
**Expected Output:** Project listing with sessions enabled; only `-commit` is shown under its project; `-default_topic` is absent.
**Verification:**
- Output contains a session section listing sessions
- `-commit` session is listed
- `-default_topic` is not listed (does not match "commit" filter)
**Pass Criteria:** exit 0 + session list auto-enabled and filtered by "commit"
**Source:** [parameter_groups.md#session-filter](../../../../../docs/cli/parameter_groups.md#session-filter)

### CC-2: agent:: alone auto-enables session display

**Goal:** Verify that providing `agent::1` without `sessions::1` automatically enables the session list in the output.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project having both main sessions and agent sessions (`agent-*.jsonl`).
**Command:** `clg .list agent::1`
**Expected Output:** Project listing with sessions enabled; only agent sessions are shown.
**Verification:**
- Output contains a session section
- Only agent sessions (`agent-*.jsonl`) are listed
- Main sessions are not included
**Pass Criteria:** exit 0 + session list auto-enabled and filtered to agent sessions only
**Source:** [parameter_groups.md#session-filter](../../../../../docs/cli/parameter_groups.md#session-filter)

### CC-3: min_entries:: alone auto-enables session display

**Goal:** Verify that providing `min_entries::N` without `sessions::1` automatically enables the session list in the output.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project having sessions of 2, 5, and 10 entries.
**Command:** `clg .list min_entries::5`
**Expected Output:** Project listing with sessions enabled; only sessions with 5 or more entries are shown.
**Verification:**
- Output contains a session section
- Session with 2 entries is excluded
- Sessions with 5 and 10 entries are included
**Pass Criteria:** exit 0 + session list auto-enabled and filtered by minimum entry count
**Source:** [parameter_groups.md#session-filter](../../../../../docs/cli/parameter_groups.md#session-filter)

### CC-4: sessions::0 suppresses display even with all three filters

**Goal:** Verify that `sessions::0` explicitly suppresses the session list even when all three filter params are provided.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project having agent sessions and varied entry counts.
**Command:** `clg .list sessions::0 session::commit agent::1 min_entries::2`
**Expected Output:** Project listing with no session section despite all filter parameters being present.
**Verification:**
- Output has no session section or session list
- Filter parameters are silently ignored (no error produced)
- Other project info (path, summary counts) may still appear
**Pass Criteria:** exit 0 + no session list in output
**Source:** [parameter_groups.md#session-filter](../../../../../docs/cli/parameter_groups.md#session-filter)

### CC-5: session:: + agent:: combined filters sessions by both

**Goal:** Verify that `session::` and `agent::` filters are applied together (AND logic) to narrow the result set.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project having: `-commit` (main session), `agent-commit-123` (agent session), `agent-other-456` (agent session).
**Command:** `clg .list session::commit agent::1`
**Expected Output:** Only `agent-commit-123` is listed under its project (matches "commit" substring AND is an agent session).
**Verification:**
- `agent-commit-123` is in the output
- `-commit` is not in the output (matches session filter but is a main session, not agent)
- `agent-other-456` is not in the output (is agent but does not match "commit")
**Pass Criteria:** exit 0 + only sessions matching both the substring and agent filters
**Source:** [parameter_groups.md#session-filter](../../../../../docs/cli/parameter_groups.md#session-filter)

### CC-6: session:: + min_entries:: combined filters by both criteria

**Goal:** Verify that `session::` and `min_entries::` filters are applied together (AND logic).
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project having: `-commit` (3 entries), `-commit-long` (10 entries), `-default_topic` (8 entries).
**Command:** `clg .list session::commit min_entries::5`
**Expected Output:** Only `-commit-long` is listed under its project (matches "commit" substring AND has ≥5 entries).
**Verification:**
- `-commit-long` is in the output (satisfies both filters)
- `-commit` is not in the output (matches substring but has only 3 entries)
- `-default_topic` is not in the output (has enough entries but no "commit" in ID)
**Pass Criteria:** exit 0 + only sessions satisfying both the substring and minimum entry count
**Source:** [parameter_groups.md#session-filter](../../../../../docs/cli/parameter_groups.md#session-filter)

### CD-1: All three filters are AND-combined (not OR)

**Goal:** Verify that when all three filter params are set, all must be satisfied simultaneously (AND semantics, not OR).
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project having: `agent-commit-5entries` (agent, contains "commit", 5 entries), `agent-other-5entries` (agent, no "commit", 5 entries), `-commit-5entries` (main, contains "commit", 5 entries).
**Command:** `clg .list session::commit agent::1 min_entries::5`
**Expected Output:** Only `agent-commit-5entries` is listed under its project (satisfies all three filters).
**Verification:**
- `agent-commit-5entries` appears in output (all 3 conditions met)
- `agent-other-5entries` does not appear (fails session:: filter — no "commit" in ID)
- `-commit-5entries` does not appear (fails agent:: filter — is a main session)
- A session with 2 entries matching the other filters would also not appear (fails min_entries::)
**Pass Criteria:** exit 0 + only sessions satisfying ALL three filters simultaneously
**Source:** [parameter_groups.md#session-filter](../../../../../docs/cli/parameter_groups.md#session-filter)
