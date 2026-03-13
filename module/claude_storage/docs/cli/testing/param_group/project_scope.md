# Parameter Group :: Project Scope

Interaction tests for the Project Scope group (`project::`). Tests verify consistent project resolution across commands using this group.

**Source:** [parameter_groups.md#project-scope](../../parameter_groups.md#project-scope)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | project:: resolves same project in .show and .search | Cross-Command Consistency |
| CC-2 | project:: with absolute path format works in .export | Format Resolution |
| CC-3 | project:: with UUID format works in .count | Format Resolution |
| CC-4 | Absent project:: defaults to cwd in .show | Default Resolution |
| CC-5 | Absent project:: defaults to cwd in .export | Default Resolution |
| CC-6 | Same project:: value returns same project in all 5 commands | Cross-Command Consistency |

## Test Coverage Summary

- Cross-Command Consistency: 2 tests (CC-1, CC-6)
- Format Resolution: 2 tests (CC-2, CC-3)
- Default Resolution: 2 tests (CC-4, CC-5)

## Test Cases

### CC-1: project:: resolves same project in .show and .search

**Goal:** Verify that a `project::` value resolves to the same project in both `.show` and `.search`.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with project at `/home/testuser/myproject` having a known session `VALID-UUID` and entries containing "hello".
**Command:** `clg .show session_id::VALID-UUID project::/home/testuser/myproject` and `clg .search query::hello project::/home/testuser/myproject`
**Expected Output:** `.show` returns session content from `myproject`; `.search` returns results scoped to `myproject` only.
**Verification:**
- `.show` returns content from a session in the target project
- `.search` results contain no entries from other projects
- Both commands exit 0
**Pass Criteria:** exit 0 in both commands + results scoped to the specified project
**Source:** [parameter_groups.md#project-scope](../../parameter_groups.md#project-scope)

### CC-2: project:: with absolute path format works in .export

**Goal:** Verify that `project::` accepts an absolute filesystem path in `.export`.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with project at `/home/testuser/myproject` having session `VALID-UUID`.
**Command:** `clg .export session_id::VALID-UUID project::/home/testuser/myproject`
**Expected Output:** Session exported as valid JSONL to stdout.
**Verification:**
- Command exits 0
- Output is valid JSONL (each line parses as a JSON object)
- Exported entries have `sessionId` matching `VALID-UUID`
**Pass Criteria:** exit 0 + valid JSONL output for the correct session
**Source:** [parameter_groups.md#project-scope](../../parameter_groups.md#project-scope)

### CC-3: project:: with UUID format works in .count

**Goal:** Verify that `project::` accepts a UUID project identifier in `.count`.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project stored under UUID directory `PROJ-UUID` containing 10 known entries.
**Command:** `clg .count project::PROJ-UUID`
**Expected Output:** Entry count of 10 for the UUID-identified project.
**Verification:**
- Command exits 0
- Reported count matches the known number of entries in the UUID-named project
- Output is a numeric count or structured count response
**Pass Criteria:** exit 0 + correct count for the UUID-identified project
**Source:** [parameter_groups.md#project-scope](../../parameter_groups.md#project-scope)

### CC-4: Absent project:: defaults to cwd in .show

**Goal:** Verify that omitting `project::` causes `.show` to default to the current working directory's project.
**Setup:** `cd /home/testuser/myproject && export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with session `VALID-UUID` in that project.
**Command:** `clg .show session_id::VALID-UUID` (no `project::` param)
**Expected Output:** Session content from the cwd-matched project.
**Verification:**
- Command exits 0 and returns session content
- Session is from the project matching `/home/testuser/myproject` path encoding
- Result is identical to `clg .show session_id::VALID-UUID project::/home/testuser/myproject`
**Pass Criteria:** exit 0 + content from the cwd-matched project
**Source:** [parameter_groups.md#project-scope](../../parameter_groups.md#project-scope)

### CC-5: Absent project:: defaults to cwd in .export

**Goal:** Verify that omitting `project::` causes `.export` to default to the current working directory's project.
**Setup:** `cd /home/testuser/myproject && export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with session `VALID-UUID` in that project.
**Command:** `clg .export session_id::VALID-UUID` (no `project::` param)
**Expected Output:** Session exported as JSONL from the cwd-matched project.
**Verification:**
- Command exits 0
- Exported JSONL contains entries from the cwd project's session
- Result is equivalent to `clg .export session_id::VALID-UUID project::/home/testuser/myproject`
**Pass Criteria:** exit 0 + valid export from the cwd-resolved project
**Source:** [parameter_groups.md#project-scope](../../parameter_groups.md#project-scope)

### CC-6: Same project:: value returns same project in all 5 commands

**Goal:** Verify that a single `project::` value resolves to the same project consistently across all commands that accept it.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with project at `/home/testuser/myproject` having 2 sessions and 10 entries.
**Command:** `.show`, `.search`, `.export`, `.count`, `.list` all with `project::/home/testuser/myproject`
**Expected Output:** All commands operate on the same project; counts and session lists are consistent.
**Verification:**
- `.count` reports 10 entries
- `.list` includes the project in results
- `.search` scopes results to that project only
- `.show` and `.export` access sessions from that project
**Pass Criteria:** exit 0 in all commands + consistent project resolution across all 5 commands
**Source:** [parameter_groups.md#project-scope](../../parameter_groups.md#project-scope)
