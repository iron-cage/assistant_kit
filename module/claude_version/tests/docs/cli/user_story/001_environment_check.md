# Test: Environment Check

Acceptance tests for User Story 001. See [user_story/001_environment_check.md](../../../../docs/cli/user_story/001_environment_check.md) for specification.

### Scope

- **Purpose**: Verify that `cm .status` provides a complete environment snapshot in one command.
- **Responsibility**: Acceptance criteria coverage for the environment check workflow.
- **Commands:** `.status`, `.help`
- **In Scope**: Single-command status output, JSON format, verbosity levels, error handling.
- **Out of Scope**: Version install (-> `002_version_upgrade.md`), process management (-> `003_process_lifecycle.md`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| US-1 | `.status` outputs version, session count, and account | Acceptance: core output |
| US-2 | `.status format::json` returns same fields as JSON | Acceptance: JSON format |
| US-3 | `.status v::2` shows additional diagnostic context | Acceptance: verbose mode |
| US-4 | Missing HOME exits 2 | Acceptance: error handling |

## Test Coverage Summary

- Core output: 1 test (US-1)
- JSON format: 1 test (US-2)
- Verbose mode: 1 test (US-3)
- Error handling: 1 test (US-4)

**Total:** 4 tests

---

### US-1: `.status` outputs version, session count, and account

- **Given:** Claude Code is installed; HOME is set
- **When:** `cm .status`
- **Then:** exit 0; output contains version string, session count, and active account in a single view
- **Exit:** 0
- **Source:** [user_story/001 -- AC bullet 1](../../../../docs/cli/user_story/001_environment_check.md)

---

### US-2: `.status format::json` returns same fields as JSON

- **Given:** Claude Code is installed; HOME is set
- **When:** `cm .status format::json`
- **Then:** exit 0; valid JSON containing version, session count, and account fields
- **Exit:** 0
- **Source:** [user_story/001 -- AC bullet 2](../../../../docs/cli/user_story/001_environment_check.md)

---

### US-3: `.status v::2` shows additional diagnostic context

- **Given:** Claude Code is installed; HOME is set
- **When:** `cm .status v::2`
- **Then:** exit 0; output contains additional diagnostic context beyond default verbosity
- **Exit:** 0
- **Source:** [user_story/001 -- AC bullet 3](../../../../docs/cli/user_story/001_environment_check.md)

---

### US-4: Missing HOME exits 2

- **Given:** HOME environment variable is unset
- **When:** `cm .status`
- **Then:** exit 2; error message indicates missing HOME
- **Exit:** 2
- **Source:** [user_story/001 -- AC bullet 4](../../../../docs/cli/user_story/001_environment_check.md)

---

### Source Functions

| Function | File | Status |
|----------|------|--------|
| -- | -- | ⏳ (pending implementation) |
