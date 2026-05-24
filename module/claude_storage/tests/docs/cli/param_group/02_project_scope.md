# Parameter Group :: Project Scope

Interaction tests for the Project Scope group (`project::`). Tests verify consistent project resolution across commands using this group.

**Source:** [003_parameter_groups.md#project-scope](../../../../docs/cli/003_parameter_groups.md#project-scope)

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

---

### CC-1: project:: resolves same project in .show and .search

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with project at `/home/testuser/myproject` having a known session `VALID-UUID` and entries containing "hello".
- **When:** `clg .show session_id::VALID-UUID project::/home/testuser/myproject` and `clg .search query::hello project::/home/testuser/myproject`
- **Then:** `.show` returns session content from `myproject`; `.search` returns results scoped to `myproject` only.; in both commands + results scoped to the specified project
- **Exit:** 0
- **Source:** [003_parameter_groups.md#project-scope](../../../../docs/cli/003_parameter_groups.md#project-scope)

---

### CC-2: project:: with absolute path format works in .export

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with project at `/home/testuser/myproject` having session `VALID-UUID`.
- **When:** `clg .export session_id::VALID-UUID project::/home/testuser/myproject`
- **Then:** Session exported as valid JSONL to stdout.; valid JSONL output for the correct session
- **Exit:** 0
- **Source:** [003_parameter_groups.md#project-scope](../../../../docs/cli/003_parameter_groups.md#project-scope)

---

### CC-3: project:: with UUID format works in .count

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project stored under UUID directory `PROJ-UUID` containing 10 known entries.
- **When:** `clg .count project::PROJ-UUID`
- **Then:** Entry count of 10 for the UUID-identified project.; correct count for the UUID-identified project
- **Exit:** 0
- **Source:** [003_parameter_groups.md#project-scope](../../../../docs/cli/003_parameter_groups.md#project-scope)

---

### CC-4: Absent project:: defaults to cwd in .show

- **Given:** `cd /home/testuser/myproject && export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with session `VALID-UUID` in that project.
- **When:** `clg .show session_id::VALID-UUID` (no `project::` param)
- **Then:** Session content from the cwd-matched project.
- **Exit:** 0
- **Source:** [003_parameter_groups.md#project-scope](../../../../docs/cli/003_parameter_groups.md#project-scope)

---

### CC-5: Absent project:: defaults to cwd in .export

- **Given:** `cd /home/testuser/myproject && export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with session `VALID-UUID` in that project.
- **When:** `clg .export session_id::VALID-UUID` (no `project::` param)
- **Then:** Session exported as JSONL from the cwd-matched project.; valid export from the cwd-resolved project
- **Exit:** 0
- **Source:** [003_parameter_groups.md#project-scope](../../../../docs/cli/003_parameter_groups.md#project-scope)

---

### CC-6: Same project:: value returns same project in all 5 commands

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with project at `/home/testuser/myproject` having 2 sessions and 10 entries.
- **When:** `.show`, `.search`, `.export`, `.count`, `.list` all with `project::/home/testuser/myproject`
- **Then:** All commands operate on the same project; counts and session lists are consistent.; in all commands + consistent project resolution across all 5 commands
- **Exit:** 0
- **Source:** [003_parameter_groups.md#project-scope](../../../../docs/cli/003_parameter_groups.md#project-scope)
