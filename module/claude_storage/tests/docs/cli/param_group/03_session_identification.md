# Parameter Group :: Session Identification

Interaction tests for the Session Identification group (`session_id::`). Tests verify direct session access behavior in `.show` and `.export`.

**Source:** [parameter_groups.md#session-identification](../../../../docs/cli/parameter_groups.md#session-identification)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | session_id:: in .show displays session content | Cross-Command |
| EC-2 | session_id:: in .export exports the same session | Cross-Command |
| EC-3 | Same session_id:: value resolves same session in both commands | Cross-Command Consistency |
| EC-4 | session_id:: required in .export, optional in .show | Required vs Optional |
| EC-5 | session_id:: depends on project:: for scoping | Dependency |
| EC-6 | session_id:: without project:: resolves via cwd | Dependency |

## Test Coverage Summary

- Cross-Command: 2 tests (EC-1, EC-2)
- Cross-Command Consistency: 1 test (EC-3)
- Required vs Optional: 1 test (EC-4)
- Dependency: 2 tests (EC-5, EC-6)

## Test Cases

---

### EC-1: session_id:: in .show displays session content

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project at cwd containing session `test-session-uuid`.
- **When:** `clg .show session_id::test-session-uuid`
- **Then:** Conversation content from the specified session with user/assistant entries.; session content displayed
- **Exit:** 0
- **Source:** [parameter_groups.md#session-identification](../../../../docs/cli/parameter_groups.md#session-identification)

---

### EC-2: session_id:: in .export exports the same session

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project at cwd containing session `test-session-uuid`.
- **When:** `clg .export session_id::test-session-uuid`
- **Then:** Valid JSONL content from the specified session written to stdout.; valid JSONL output for the specified session
- **Exit:** 0
- **Source:** [parameter_groups.md#session-identification](../../../../docs/cli/parameter_groups.md#session-identification)

---

### EC-3: Same session_id:: value resolves same session in both commands

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project at cwd containing session `test-session-uuid` with a known entry count of 4.
- **When:** `clg .show session_id::test-session-uuid` and `clg .export session_id::test-session-uuid`
- **Then:** Both commands operate on the same session; `.export` JSONL line count matches entry count visible in `.show`.; in both + consistent session resolution and entry counts
- **Exit:** 0
- **Source:** [parameter_groups.md#session-identification](../../../../docs/cli/parameter_groups.md#session-identification)

---

### EC-4: session_id:: required in .export, optional in .show

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project at cwd.
- **When:** `clg .export` (no `session_id::`) and `clg .show` (no `session_id::`)
- **Then:** `.export` exits 1 with error about missing required `session_id::`; `.show` exits 0 with project-level output.; `.export` exits 1 + missing-required error; `.show` exits 0 + valid output
- **Exit:** 0
- **Source:** [parameter_groups.md#session-identification](../../../../docs/cli/parameter_groups.md#session-identification)

---

### EC-5: session_id:: depends on project:: for scoping

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with two projects — `/home/testuser/project-a` and `/home/testuser/project-b` — each having a session named `-default_topic`. Run from a directory that is not either project.
- **When:** `clg .show session_id::-default_topic project::/home/testuser/project-a`
- **Then:** Session content from `project-a`'s `-default_topic` session, not from `project-b`.; session resolved within the specified project scope
- **Exit:** 0
- **Source:** [parameter_groups.md#session-identification](../../../../docs/cli/parameter_groups.md#session-identification)

---

### EC-6: session_id:: without project:: resolves via cwd

- **Given:** `cd /home/testuser/project-a && export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with project at `/home/testuser/project-a` having session `-default_topic`.
- **When:** `clg .show session_id::-default_topic` (no `project::`)
- **Then:** Session content from `project-a`'s `-default_topic` session.; session resolved via cwd-matched project
- **Exit:** 0
- **Source:** [parameter_groups.md#session-identification](../../../../docs/cli/parameter_groups.md#session-identification)
