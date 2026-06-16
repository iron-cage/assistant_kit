# Test: Process Lifecycle

Acceptance tests for User Story 003. See [user_story/003_process_lifecycle.md](../../../../docs/cli/user_story/003_process_lifecycle.md) for specification.

### Scope

- **Purpose**: Verify process inspection and termination workflow.
- **Responsibility**: Acceptance criteria coverage for the process lifecycle scenario.
- **Commands:** `.processes`, `.processes.kill`
- **In Scope**: Process listing, JSON format, dry-run kill preview, actual kill, force kill, post-kill verification.
- **Out of Scope**: Version management (-> `02_version_upgrade.md`), settings (-> `04_settings_management.md`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| US-1 | `.processes` lists PIDs and working directories | Acceptance: list |
| US-2 | `.processes format::json` returns JSON array | Acceptance: JSON format |
| US-3 | `.processes.kill dry::1` previews without sending signals | Acceptance: dry-run |
| US-4 | `.processes.kill` sends SIGTERM then SIGKILL | Acceptance: graceful kill |
| US-5 | `.processes.kill force::1` sends SIGKILL directly | Acceptance: force kill |
| US-6 | `.processes` after kill returns empty list | Acceptance: post-kill |

## Test Coverage Summary

- Process listing: 1 test (US-1)
- JSON format: 1 test (US-2)
- Dry-run kill: 1 test (US-3)
- Graceful kill: 1 test (US-4)
- Force kill: 1 test (US-5)
- Post-kill verification: 1 test (US-6)

**Total:** 6 tests

---

### US-1: `.processes` lists PIDs and working directories

- **Given:** at least one Claude Code process is running
- **When:** `clv .processes`
- **Then:** exit 0; output contains PID and working directory for each running process
- **Exit:** 0
- **Source:** [user_story/003 -- AC bullet 1](../../../../docs/cli/user_story/003_process_lifecycle.md)

---

### US-2: `.processes format::json` returns JSON array

- **Given:** at least one Claude Code process is running
- **When:** `clv .processes format::json`
- **Then:** exit 0; valid JSON array with process entries
- **Exit:** 0
- **Source:** [user_story/003 -- AC bullet 2](../../../../docs/cli/user_story/003_process_lifecycle.md)

---

### US-3: `.processes.kill dry::1` previews without sending signals

- **Given:** at least one Claude Code process is running
- **When:** `clv .processes.kill dry::1`
- **Then:** exit 0; stdout shows which processes would be killed; no signals sent
- **Exit:** 0
- **Source:** [user_story/003 -- AC bullet 3](../../../../docs/cli/user_story/003_process_lifecycle.md)

---

### US-4: `.processes.kill` sends SIGTERM then SIGKILL

- **Given:** at least one Claude Code process is running
- **When:** `clv .processes.kill`
- **Then:** exit 0; SIGTERM sent first, waits 2 seconds, SIGKILL to survivors
- **Exit:** 0
- **Source:** [user_story/003 -- AC bullet 4](../../../../docs/cli/user_story/003_process_lifecycle.md)

---

### US-5: `.processes.kill force::1` sends SIGKILL directly

- **Given:** at least one Claude Code process is running
- **When:** `clv .processes.kill force::1`
- **Then:** exit 0; SIGKILL sent directly without SIGTERM
- **Exit:** 0
- **Source:** [user_story/003 -- AC bullet 5](../../../../docs/cli/user_story/003_process_lifecycle.md)

---

### US-6: `.processes` after kill returns empty list

- **Given:** all Claude Code processes were just killed
- **When:** `clv .processes`
- **Then:** exit 0; empty output (no running processes)
- **Exit:** 0
- **Source:** [user_story/003 -- AC bullet 6](../../../../docs/cli/user_story/003_process_lifecycle.md)

---

### Source Functions

| Function | File | Status |
|----------|------|--------|
| `us01_003_processes_exits_0` | `integration/user_story_test.rs` | ✅ |
| `us02_003_processes_json_format` | `integration/user_story_test.rs` | ✅ |
| `us03_003_processes_kill_dry_preview` | `integration/user_story_test.rs` | ✅ |
| `us04_003_processes_kill_graceful` | `integration/user_story_test.rs` | ✅ |
| `us05_003_processes_kill_force` | `integration/user_story_test.rs` | ✅ |
| `us06_003_processes_empty_after_kill` | `integration/user_story_test.rs` | ✅ |
