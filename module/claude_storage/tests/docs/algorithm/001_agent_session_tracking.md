# Algorithm :: Agent Session Tracking

Direct contract tests for agent session detection behaviors documented in the agent session tracking algorithm.

**Source:** [algorithm/001_agent_session_tracking.md](../../../docs/algorithm/001_agent_session_tracking.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| AL-1 | Agent entry `isSidechain: true` field present in agent JSONL | Agent Session Format |
| AL-2 | Agent entry `agentId` matches the agent filename suffix | Agent Session Format |

## Test Coverage Summary

- Agent Session Format: 2 tests (AL-1, AL-2)

**Total:** 2 algorithm contract cases

**Implementation target:** `tests/invariant_contracts_test.rs`

## Test Cases

---

### AL-1: Agent entry `isSidechain: true` field present in agent JSONL

- **Given:** a session JSONL file named `agent-{id}.jsonl` with at least one entry
- **When:** the first entry is parsed
- **Then:** the entry has `isSidechain: true`; absent or `false` would indicate a non-agent session

---

### AL-2: Agent entry `agentId` matches the agent filename suffix

- **Given:** an agent session file `agent-64bdad98.jsonl`
- **When:** the first entry is parsed and `agentId` is extracted
- **Then:** `agentId` equals `"64bdad98"` — matching the filename suffix exactly
