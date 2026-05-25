# Parameter Group :: Session Filter

Interaction tests for the Session Filter group (`session::`, `agent::`, `min_entries::`). Tests verify auto-enable behavior, combined filter semantics, and `sessions::` override interactions.

**Source:** [param_group/04_session_filter.md](../../../../docs/cli/param_group/04_session_filter.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | session:: alone auto-enables session display | Auto-Enable |
| CC-2 | agent:: alone auto-enables session display | Auto-Enable |
| CC-3 | min_entries:: alone auto-enables session display | Auto-Enable |
| CC-4 | sessions::0 suppresses display even with all three filters | Override Interaction |
| CC-5 | session:: + agent:: combined filters sessions by both | Combined Filter |
| CC-6 | session:: + min_entries:: combined filters by both criteria | Combined Filter |
| CC-7 | All three filters are AND-combined (not OR) | Filter Logic |

## Test Coverage Summary

- Auto-Enable: 3 tests (CC-1, CC-2, CC-3)
- Override Interaction: 1 test (CC-4)
- Combined Filter: 2 tests (CC-5, CC-6)
- Filter Logic: 1 test (CC-7)

## Test Cases

---

### CC-1: session:: alone auto-enables session display

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project having sessions `-commit` and `-default_topic`.
- **When:** `clg .list session::commit`
- **Then:** Project listing with sessions enabled; only `-commit` is shown under its project; `-default_topic` is absent.; session list auto-enabled and filtered by "commit"
- **Exit:** 0
- **Source:** [param_group/04_session_filter.md](../../../../docs/cli/param_group/04_session_filter.md)

---

### CC-2: agent:: alone auto-enables session display

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project having both main sessions and agent sessions (`agent-*.jsonl`).
- **When:** `clg .list agent::1`
- **Then:** Project listing with sessions enabled; only agent sessions are shown.; session list auto-enabled and filtered to agent sessions only
- **Exit:** 0
- **Source:** [param_group/04_session_filter.md](../../../../docs/cli/param_group/04_session_filter.md)

---

### CC-3: min_entries:: alone auto-enables session display

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project having sessions of 2, 5, and 10 entries.
- **When:** `clg .list min_entries::5`
- **Then:** Project listing with sessions enabled; only sessions with 5 or more entries are shown.; session list auto-enabled and filtered by minimum entry count
- **Exit:** 0
- **Source:** [param_group/04_session_filter.md](../../../../docs/cli/param_group/04_session_filter.md)

---

### CC-4: sessions::0 suppresses display even with all three filters

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project having agent sessions and varied entry counts.
- **When:** `clg .list sessions::0 session::commit agent::1 min_entries::2`
- **Then:** Project listing with no session section despite all filter parameters being present.; no session list in output
- **Exit:** 0
- **Source:** [param_group/04_session_filter.md](../../../../docs/cli/param_group/04_session_filter.md)

---

### CC-5: session:: + agent:: combined filters sessions by both

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project having: `-commit` (main session), `agent-commit-123` (agent session), `agent-other-456` (agent session).
- **When:** `clg .list session::commit agent::1`
- **Then:** Only `agent-commit-123` is listed under its project (matches "commit" substring AND is an agent session).; only sessions matching both the substring and agent filters
- **Exit:** 0
- **Source:** [param_group/04_session_filter.md](../../../../docs/cli/param_group/04_session_filter.md)

---

### CC-6: session:: + min_entries:: combined filters by both criteria

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project having: `-commit` (3 entries), `-commit-long` (10 entries), `-default_topic` (8 entries).
- **When:** `clg .list session::commit min_entries::5`
- **Then:** Only `-commit-long` is listed under its project (matches "commit" substring AND has ≥5 entries).; only sessions satisfying both the substring and minimum entry count
- **Exit:** 0
- **Source:** [param_group/04_session_filter.md](../../../../docs/cli/param_group/04_session_filter.md)

---

### CC-7: All three filters are AND-combined (not OR)

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project having: `agent-commit-5entries` (agent, contains "commit", 5 entries), `agent-other-5entries` (agent, no "commit", 5 entries), `-commit-5entries` (main, contains "commit", 5 entries).
- **When:** `clg .list session::commit agent::1 min_entries::5`
- **Then:** Only `agent-commit-5entries` is listed under its project (satisfies all three filters).; only sessions satisfying ALL three filters simultaneously
- **Exit:** 0
- **Source:** [param_group/04_session_filter.md](../../../../docs/cli/param_group/04_session_filter.md)
