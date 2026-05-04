# Parameter Group :: Project Scope

Interaction tests for the Project Scope group (`project::`). Tests verify consistent project resolution across commands using this group.

**Source:** [parameter_groups.md#project-scope](../../../../docs/cli/parameter_groups.md#project-scope)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | project:: resolves same project in .show and .search | Cross-Command Consistency |
| EC-2 | project:: with absolute path format works in .export | Format Resolution |
| EC-3 | project:: with UUID format works in .count | Format Resolution |
| EC-4 | Absent project:: defaults to cwd in .show | Default Resolution |
| EC-5 | Absent project:: defaults to cwd in .export | Default Resolution |
| EC-6 | Same project:: value returns same project in all 5 commands | Cross-Command Consistency |

## Test Coverage Summary

- Cross-Command Consistency: 2 tests (EC-1, EC-6)
- Format Resolution: 2 tests (EC-2, EC-3)
- Default Resolution: 2 tests (EC-4, EC-5)

## Test Cases

---

### EC-1: project:: resolves same project in .show and .search

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with project at `/home/testuser/myproject` having a known session `VALID-UUID` and entries containing "hello".
- **When:** `clg .show session_id::VALID-UUID project::/home/testuser/myproject` and `clg .search query::hello project::/home/testuser/myproject`
- **Then:** `.show` returns session content from `myproject`; `.search` returns results scoped to `myproject` only.; in both commands + results scoped to the specified project
- **Exit:** 0
- **Source:** [parameter_groups.md#project-scope](../../../../docs/cli/parameter_groups.md#project-scope)

---

### EC-2: project:: with absolute path format works in .export

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with project at `/home/testuser/myproject` having session `VALID-UUID`.
- **When:** `clg .export session_id::VALID-UUID project::/home/testuser/myproject`
- **Then:** Session exported as valid JSONL to stdout.; valid JSONL output for the correct session
- **Exit:** 0
- **Source:** [parameter_groups.md#project-scope](../../../../docs/cli/parameter_groups.md#project-scope)

---

### EC-3: project:: with UUID format works in .count

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a project stored under UUID directory `PROJ-UUID` containing 10 known entries.
- **When:** `clg .count project::PROJ-UUID`
- **Then:** Entry count of 10 for the UUID-identified project.; correct count for the UUID-identified project
- **Exit:** 0
- **Source:** [parameter_groups.md#project-scope](../../../../docs/cli/parameter_groups.md#project-scope)

---

### EC-4: Absent project:: defaults to cwd in .show

- **Given:** `cd /home/testuser/myproject && export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with session `VALID-UUID` in that project.
- **When:** `clg .show session_id::VALID-UUID` (no `project::` param)
- **Then:** Session content from the cwd-matched project.
- **Exit:** 0
- **Source:** [parameter_groups.md#project-scope](../../../../docs/cli/parameter_groups.md#project-scope)

---

### EC-5: Absent project:: defaults to cwd in .export

- **Given:** `cd /home/testuser/myproject && export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with session `VALID-UUID` in that project.
- **When:** `clg .export session_id::VALID-UUID` (no `project::` param)
- **Then:** Session exported as JSONL from the cwd-matched project.; valid export from the cwd-resolved project
- **Exit:** 0
- **Source:** [parameter_groups.md#project-scope](../../../../docs/cli/parameter_groups.md#project-scope)

---

### EC-6: Same project:: value returns same project in all 5 commands

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with project at `/home/testuser/myproject` having 2 sessions and 10 entries.
- **When:** `.show`, `.search`, `.export`, `.count`, `.list` all with `project::/home/testuser/myproject`
- **Then:** All commands operate on the same project; counts and session lists are consistent.; in all commands + consistent project resolution across all 5 commands
- **Exit:** 0
- **Source:** [parameter_groups.md#project-scope](../../../../docs/cli/parameter_groups.md#project-scope)
