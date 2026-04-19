# Parameter Group :: Session Identification

Interaction tests for the Session Identification group (`session_id::`). Tests verify direct session access behavior in `.show` and `.export`.

**Source:** [parameter_groups.md#session-identification](../../../../../docs/cli/parameter_groups.md#session-identification)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | session_id:: in .show displays session content | Cross-Command |
| CC-2 | session_id:: in .export exports the same session | Cross-Command |
| CC-3 | Same session_id:: value resolves same session in both commands | Cross-Command Consistency |
| CC-4 | session_id:: required in .export, optional in .show | Required vs Optional |
| CD-1 | session_id:: depends on project:: for scoping | Dependency |
| CD-2 | session_id:: without project:: resolves via cwd | Dependency |

## Test Coverage Summary

- Cross-Command: 2 tests (CC-1, CC-2)
- Cross-Command Consistency: 1 test (CC-3)
- Required vs Optional: 1 test (CC-4)
- Dependency: 2 tests (CD-1, CD-2)

## Test Cases

### CC-1: session_id:: in .show displays session content

**Goal:** Verify that `session_id::` in `.show` successfully retrieves and displays session content.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project at cwd containing session `test-session-uuid`.
**Command:** `clg .show session_id::test-session-uuid`
**Expected Output:** Conversation content from the specified session with user/assistant entries.
**Verification:**
- Output is non-empty
- Output contains conversation entries from the specified session
- Session ID or file reference appears in the output
**Pass Criteria:** exit 0 + session content displayed
**Source:** [parameter_groups.md#session-identification](../../../../../docs/cli/parameter_groups.md#session-identification)

### CC-2: session_id:: in .export exports the same session

**Goal:** Verify that `session_id::` in `.export` successfully exports the session as JSONL.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project at cwd containing session `test-session-uuid`.
**Command:** `clg .export session_id::test-session-uuid`
**Expected Output:** Valid JSONL content from the specified session written to stdout.
**Verification:**
- Output is valid JSONL (each line parses as a JSON object)
- JSONL entries contain `sessionId` field matching `test-session-uuid`
- Output is non-empty
**Pass Criteria:** exit 0 + valid JSONL output for the specified session
**Source:** [parameter_groups.md#session-identification](../../../../../docs/cli/parameter_groups.md#session-identification)

### CC-3: Same session_id:: value resolves same session in both commands

**Goal:** Verify that a single `session_id::` value consistently resolves to the same session in both `.show` and `.export`.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project at cwd containing session `test-session-uuid` with a known entry count of 4.
**Command:** `clg .show session_id::test-session-uuid` and `clg .export session_id::test-session-uuid`
**Expected Output:** Both commands operate on the same session; `.export` JSONL line count matches entry count visible in `.show`.
**Verification:**
- Both commands exit 0
- `.export` parsed as JSONL has the same number of entries (4) as shown in `.show`
- Both commands reference the same session file
**Pass Criteria:** exit 0 in both + consistent session resolution and entry counts
**Source:** [parameter_groups.md#session-identification](../../../../../docs/cli/parameter_groups.md#session-identification)

### CC-4: session_id:: required in .export, optional in .show

**Goal:** Verify that `.export` errors without `session_id::` while `.show` succeeds without it.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project at cwd.
**Command:** `clg .export` (no `session_id::`) and `clg .show` (no `session_id::`)
**Expected Output:** `.export` exits 1 with error about missing required `session_id::`; `.show` exits 0 with project-level output.
**Verification:**
- `clg .export` exits 1 with an error message referencing `session_id::` as required
- `clg .show` exits 0 without error
- `.show` output is non-empty (shows project summary or prompts for session selection)
**Pass Criteria:** `.export` exits 1 + missing-required error; `.show` exits 0 + valid output
**Source:** [parameter_groups.md#session-identification](../../../../../docs/cli/parameter_groups.md#session-identification)

### CD-1: session_id:: depends on project:: for scoping

**Goal:** Verify that `session_id::` resolves within the project specified by `project::`, not globally.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with two projects — `/home/testuser/project-a` and `/home/testuser/project-b` — each having a session named `-default_topic`. Run from a directory that is not either project.
**Command:** `clg .show session_id::-default_topic project::/home/testuser/project-a`
**Expected Output:** Session content from `project-a`'s `-default_topic` session, not from `project-b`.
**Verification:**
- Output is from `project-a`'s session (contains messages written to that project)
- Output is not from `project-b`'s session
- Command exits 0
**Pass Criteria:** exit 0 + session resolved within the specified project scope
**Source:** [parameter_groups.md#session-identification](../../../../../docs/cli/parameter_groups.md#session-identification)

### CD-2: session_id:: without project:: resolves via cwd

**Goal:** Verify that without `project::`, `session_id::` resolves within the project matching the current working directory.
**Setup:** `cd /home/testuser/project-a && export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with project at `/home/testuser/project-a` having session `-default_topic`.
**Command:** `clg .show session_id::-default_topic` (no `project::`)
**Expected Output:** Session content from `project-a`'s `-default_topic` session.
**Verification:**
- Output contains entries from `project-a`'s `-default_topic` session
- No error about ambiguous or missing project
- Result is equivalent to `clg .show session_id::-default_topic project::/home/testuser/project-a`
**Pass Criteria:** exit 0 + session resolved via cwd-matched project
**Source:** [parameter_groups.md#session-identification](../../../../../docs/cli/parameter_groups.md#session-identification)
