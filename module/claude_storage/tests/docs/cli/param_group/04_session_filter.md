# Parameter Group :: Session Filter

Interaction tests for the Session Filter group (`session::`, `agent::`, `min_entries::`). Tests verify auto-enable behavior, combined filter semantics, and `sessions::` override interactions.

**Source:** [parameter_groups.md#session-filter](../../../../docs/cli/parameter_groups.md#session-filter)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | session:: alone auto-enables session display | Auto-Enable |
| EC-2 | agent:: alone auto-enables session display | Auto-Enable |
| EC-3 | min_entries:: alone auto-enables session display | Auto-Enable |
| EC-4 | sessions::0 suppresses display even with all three filters | Override Interaction |
| EC-5 | session:: + agent:: combined filters sessions by both | Combined Filter |
| EC-6 | session:: + min_entries:: combined filters by both criteria | Combined Filter |
| EC-7 | All three filters are AND-combined (not OR) | Filter Logic |

## Test Coverage Summary

- Auto-Enable: 3 tests (EC-1, EC-2, EC-3)
- Override Interaction: 1 test (EC-4)
- Combined Filter: 2 tests (EC-5, EC-6)
- Filter Logic: 1 test (EC-7)

## Test Cases

---

### EC-1: session:: alone auto-enables session display

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project having sessions `-commit` and `-default_topic`.
- **When:** `clg .list session::commit`
- **Then:** Project listing with sessions enabled; only `-commit` is shown under its project; `-default_topic` is absent.; session list auto-enabled and filtered by "commit"
- **Exit:** 0
- **Source:** [parameter_groups.md#session-filter](../../../../docs/cli/parameter_groups.md#session-filter)

---

### EC-2: agent:: alone auto-enables session display

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project having both main sessions and agent sessions (`agent-*.jsonl`).
- **When:** `clg .list agent::1`
- **Then:** Project listing with sessions enabled; only agent sessions are shown.; session list auto-enabled and filtered to agent sessions only
- **Exit:** 0
- **Source:** [parameter_groups.md#session-filter](../../../../docs/cli/parameter_groups.md#session-filter)

---

### EC-3: min_entries:: alone auto-enables session display

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project having sessions of 2, 5, and 10 entries.
- **When:** `clg .list min_entries::5`
- **Then:** Project listing with sessions enabled; only sessions with 5 or more entries are shown.; session list auto-enabled and filtered by minimum entry count
- **Exit:** 0
- **Source:** [parameter_groups.md#session-filter](../../../../docs/cli/parameter_groups.md#session-filter)

---

### EC-4: sessions::0 suppresses display even with all three filters

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project having agent sessions and varied entry counts.
- **When:** `clg .list sessions::0 session::commit agent::1 min_entries::2`
- **Then:** Project listing with no session section despite all filter parameters being present.; no session list in output
- **Exit:** 0
- **Source:** [parameter_groups.md#session-filter](../../../../docs/cli/parameter_groups.md#session-filter)

---

### EC-5: session:: + agent:: combined filters sessions by both

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project having: `-commit` (main session), `agent-commit-123` (agent session), `agent-other-456` (agent session).
- **When:** `clg .list session::commit agent::1`
- **Then:** Only `agent-commit-123` is listed under its project (matches "commit" substring AND is an agent session).; only sessions matching both the substring and agent filters
- **Exit:** 0
- **Source:** [parameter_groups.md#session-filter](../../../../docs/cli/parameter_groups.md#session-filter)

---

### EC-6: session:: + min_entries:: combined filters by both criteria

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project having: `-commit` (3 entries), `-commit-long` (10 entries), `-default_topic` (8 entries).
- **When:** `clg .list session::commit min_entries::5`
- **Then:** Only `-commit-long` is listed under its project (matches "commit" substring AND has ≥5 entries).; only sessions satisfying both the substring and minimum entry count
- **Exit:** 0
- **Source:** [parameter_groups.md#session-filter](../../../../docs/cli/parameter_groups.md#session-filter)

---

### EC-7: All three filters are AND-combined (not OR)

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project having: `agent-commit-5entries` (agent, contains "commit", 5 entries), `agent-other-5entries` (agent, no "commit", 5 entries), `-commit-5entries` (main, contains "commit", 5 entries).
- **When:** `clg .list session::commit agent::1 min_entries::5`
- **Then:** Only `agent-commit-5entries` is listed under its project (satisfies all three filters).; only sessions satisfying ALL three filters simultaneously
- **Exit:** 0
- **Source:** [parameter_groups.md#session-filter](../../../../docs/cli/parameter_groups.md#session-filter)
