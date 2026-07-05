# Invariant :: Session Family

Direct contract tests for the session family grouping behavioral invariant.

**Source:** [invariant/002_session_family.md](../../../docs/invariant/002_session_family.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IN-1 | Flat layout: agents discovered by `agent-*.jsonl` filename at project root | Flat Layout (B7) |
| IN-2 | Flat layout: family membership established via `sessionId` field in first agent entry | Flat Layout (B7) |
| IN-3 | Hierarchical layout: agents discovered in `{uuid}/subagents/` subdirectory | Hierarchical Layout (B13) |
| IN-4 | Hierarchical layout: family membership established by directory structure, not `sessionId` | Hierarchical Layout (B13) |
| IN-5 | Both layouts coexist in same project directory without conflict | Coexistence |
| IN-6 | Empty `.meta.json` sidecar is parsed without error | Agent Metadata |
| IN-7 | Agent `slug` field is identical across all agents from the same parent session | Agent Slug |
| IN-8 | Agent entries thread within their own session via `parentUuid`, not back to parent session | Agent Threading |

## Test Coverage Summary

- Flat Layout (B7): 2 tests (IN-1, IN-2)
- Hierarchical Layout (B13): 2 tests (IN-3, IN-4)
- Coexistence: 1 test (IN-5)
- Agent Metadata: 1 test (IN-6)
- Agent Slug: 1 test (IN-7)
- Agent Threading: 1 test (IN-8)

**Total:** 8 invariant contract cases

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

---

### IN-6: Empty `.meta.json` sidecar is parsed without error

- **Given:** a `.meta.json` file with 0 bytes (empty file) alongside an agent session
- **When:** the parser attempts to read agent metadata from that file
- **Then:** no parse error is raised; metadata is treated as absent (empty struct or `None`)

---

### IN-7: Agent `slug` field is identical across all agents from the same parent session

- **Given:** two agent JSONL files belonging to the same parent session (same `sessionId`)
- **When:** first entries of both agent files are parsed and `slug` fields are extracted
- **Then:** both `slug` values are identical; the slug is a session-family-level identifier, not per-agent

---

### IN-8: Agent entries thread within their own session via `parentUuid`, not back to parent session

- **Given:** an agent JSONL file with at least 2 entries; the first entry has `parentUuid: null`
- **When:** the second (and subsequent) entries are parsed and `parentUuid` is extracted
- **Then:** each `parentUuid` references a UUID present in the agent JSONL itself — NOT any UUID from the root session JSONL; agent threading is internal to the agent session only
