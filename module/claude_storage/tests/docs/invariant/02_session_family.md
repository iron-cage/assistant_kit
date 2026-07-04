# Invariant :: Session Family

Direct contract tests for the session family grouping behavioral invariant.

**Source:** [invariant/02_session_family.md](../../../docs/invariant/02_session_family.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IN-1 | Flat layout: agents discovered by `agent-*.jsonl` filename at project root | Flat Layout (B7) |
| IN-2 | Flat layout: family membership established via `sessionId` field in first agent entry | Flat Layout (B7) |
| IN-3 | Hierarchical layout: agents discovered in `{uuid}/subagents/` subdirectory | Hierarchical Layout (B13) |
| IN-4 | Hierarchical layout: family membership established by directory structure, not `sessionId` | Hierarchical Layout (B13) |
| IN-5 | Both layouts coexist in same project directory without conflict | Coexistence |

## Test Coverage Summary

- Flat Layout (B7): 2 tests (IN-1, IN-2)
- Hierarchical Layout (B13): 2 tests (IN-3, IN-4)
- Coexistence: 1 test (IN-5)

**Total:** 5 invariant contract cases

**Implementation target:** `tests/invariant_contracts_test.rs`

## Test Cases

---

### IN-1: Flat layout: agents discovered by `agent-*.jsonl` filename at project root

- **Given:** project directory containing `{uuid}.jsonl` (root session) and `agent-64bdad98.jsonl` (agent session) at the same level
- **When:** session discovery runs on the project directory
- **Then:** `agent-64bdad98.jsonl` is discovered as an agent session belonging to the project

---

### IN-2: Flat layout: family membership established via `sessionId` field

- **Given:** flat project directory; `agent-64bdad98.jsonl` first entry has `sessionId: "9425242b-..."`; a root session `9425242b-....jsonl` exists
- **When:** family grouping is applied
- **Then:** the agent is associated with root session `9425242b-...`; the two form one Session Family

---

### IN-3: Hierarchical layout: agents discovered in `{uuid}/subagents/` subdirectory

- **Given:** project directory with `{uuid}.jsonl` at root and `{uuid}/subagents/agent-ac9afcb5.jsonl`
- **When:** session discovery runs on the project directory
- **Then:** `agent-ac9afcb5.jsonl` is discovered as an agent session under `{uuid}`

---

### IN-4: Hierarchical layout: family membership by directory structure

- **Given:** `{uuid}/subagents/agent-ac9afcb5.jsonl` exists; the `sessionId` field in agent entries may reference a different UUID
- **When:** family grouping is applied
- **Then:** agent is associated with `{uuid}` root session by directory structure, not by `sessionId` field value

---

### IN-5: Both layouts coexist in same project directory

- **Given:** project directory with flat agent `agent-abc123.jsonl` at root AND hierarchical agent `{uuid2}/subagents/agent-def456.jsonl` for a different root session
- **When:** session discovery runs
- **Then:** both agents are discovered; each is associated with its correct root session; no collision or omission occurs
