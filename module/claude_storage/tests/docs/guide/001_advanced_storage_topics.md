# Guide :: Advanced Storage Topics

Contract tests for storage architecture behaviors documented in the advanced storage topics guide.

**Source:** [guide/001_advanced_storage_topics.md](../../../docs/guide/001_advanced_storage_topics.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| GD-1 | Agent entry `isSidechain: true` field present in agent JSONL | Agent Session Format |
| GD-2 | Agent entry `agentId` matches the agent filename suffix | Agent Session Format |
| GD-3 | Empty `.meta.json` sidecar is parsed without error | Agent Metadata |
| GD-4 | History entry `timestamp` field is milliseconds-since-epoch (> 10^12) | History Format |
| GD-5 | Session env directory is empty — contains no files | Session Environment |
| GD-6 | Agent `slug` field is identical across all agents from the same parent session | Agent Slug |
| GD-7 | Agent entries thread within their own session via `parentUuid`, not back to parent session | Agent Threading |

## Test Coverage Summary

- Agent Session Format: 2 tests (GD-1, GD-2)
- Agent Metadata: 1 test (GD-3)
- History Format: 1 test (GD-4)
- Session Environment: 1 test (GD-5)
- Agent Slug: 1 test (GD-6)
- Agent Threading: 1 test (GD-7)

**Total:** 7 guide contract cases

**Implementation target:** `tests/invariant_contracts_test.rs`

## Test Cases

---

### GD-1: Agent entry `isSidechain: true` field present in agent JSONL

- **Given:** a session JSONL file named `agent-{id}.jsonl` with at least one entry
- **When:** the first entry is parsed
- **Then:** the entry has `isSidechain: true`; absent or `false` would indicate a non-agent session

---

### GD-2: Agent entry `agentId` matches the agent filename suffix

- **Given:** an agent session file `agent-64bdad98.jsonl`
- **When:** the first entry is parsed and `agentId` is extracted
- **Then:** `agentId` equals `"64bdad98"` — matching the filename suffix exactly

---

### GD-3: Empty `.meta.json` sidecar is parsed without error

- **Given:** a `.meta.json` file with 0 bytes (empty file) alongside an agent session
- **When:** the parser attempts to read agent metadata from that file
- **Then:** no parse error is raised; metadata is treated as absent (empty struct or `None`)

---

### GD-4: History entry `timestamp` field is milliseconds-since-epoch

- **Given:** a `history.jsonl` entry with a `timestamp` field value
- **When:** the value is inspected numerically
- **Then:** the value is greater than `10^12` (i.e., milliseconds since UNIX epoch, not seconds)

---

### GD-5: Session env directory contains no files

- **Given:** a `session-env/{session-uuid}/` directory in the storage root
- **When:** the directory is listed
- **Then:** the directory is empty (contains no files or subdirectories); only `.` and `..` present

---

### GD-6: Agent `slug` field is identical across all agents from the same parent session

- **Given:** two agent JSONL files belonging to the same parent session (same `sessionId`)
- **When:** first entries of both agent files are parsed and `slug` fields are extracted
- **Then:** both `slug` values are identical; the slug is a session-family-level identifier, not per-agent

---

### GD-7: Agent entries thread within their own session via `parentUuid`, not back to parent session

- **Given:** an agent JSONL file with at least 2 entries; the first entry has `parentUuid: null`
- **When:** the second (and subsequent) entries are parsed and `parentUuid` is extracted
- **Then:** each `parentUuid` references a UUID present in the agent JSONL itself — NOT any UUID from the root session JSONL; agent threading is internal to the agent session only
