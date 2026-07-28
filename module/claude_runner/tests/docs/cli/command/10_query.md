# Test: `query`

Integration test planning for the `query` command. See [command/10_query.md](../../../../docs/cli/command/10_query.md) for specification.

`query` starts a PID-addressed, detached bidirectional control session (start form) or
dispatches one of 25 camelCase control methods against an already-running session
(dispatch form). Tests verify session start, `ps`/`kill` interoperability, the
interrupt/not-found control cycle, help text listing all 25 methods, unknown-method
validation, a `run` regression guard, and dispatch of the 24 remaining methods. The
`fake_claude_control` fixture (`tests/fixtures/fake_claude_control.rs`) is a compiled ELF
binary speaking the real bidirectional control-session wire protocol, so
`find_claude_processes()` can discover it and `query.rs`'s daemon can complete a real
`spawn_control_session()` handshake against it — no mocking.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `clr query "<msg>"` → prints a PID, exits 0 (QT-1) | Session start |
| IT-2 | `clr ps` lists the query session, `Mode` column == `query` (QT-2) | Session registry |
| IT-3 | `interrupt` on a live session, then on an already-gone PID (QT-3, QT-4) | Control cycle + Not found |
| IT-4 | `clr kill <pid>` terminates a query session (QT-5) | Termination |
| IT-5 | `clr query --help` lists `query` and all 25 method names (QT-6) | Documentation |
| IT-6 | `clr query <pid> notARealMethod` → exit 1, lists all 25 valid names (QT-7) | Validation |
| IT-7 | `clr run --help` / `clr --dry-run` unaffected by `query` addition (QT-8) | Regression guard |
| IT-8 | Remaining 24 methods each dispatch successfully against one shared session (QT-9–QT-32) | Control cycle |

## Test Coverage Summary

- Session start: 1 test (IT-1)
- Session registry: 1 test (IT-2)
- Control cycle + Not found: 1 test (IT-3, covers 2 scenarios)
- Termination: 1 test (IT-4)
- Documentation: 1 test (IT-5)
- Validation: 1 test (IT-6)
- Regression guard: 1 test (IT-7)
- Control cycle (remaining methods): 1 test (IT-8, covers 24 scenarios)

**Total:** 8 test functions covering 32 Test Matrix scenarios (QT-1–QT-32)

---

### IT-1: Start form prints PID and exits 0

- **Command:** `clr query "hello from query_command_test"` (against `fake_claude_control`)
- **Expected behavior:** stdout is a single line parsing as `u32`; the daemon's Unix socket exists at `<CLR_QUERY_DIR>/<pid>.sock` before this process exits
- **Exit:** 0
- **Source:** [command/10_query.md](../../../../docs/cli/command/10_query.md)
- **Platform:** Linux/Unix only (`find_claude_processes()` + Unix domain sockets)

---

### IT-2: `clr ps` distinguishes a query session

- **Setup:** start a query session (IT-1's helper)
- **Command:** `clr ps --columns pid,mode --pid <pid>`
- **Expected behavior:** exit 0; stdout contains the PID and the literal string `query` in the `Mode` column
- **Exit:** 0
- **Source:** [command/10_query.md](../../../../docs/cli/command/10_query.md), [command/06_ps.md](../../../../docs/cli/command/06_ps.md)

---

### IT-3: `interrupt` control cycle, then not-found contract

- **Setup:** start a query session
- **Command 1:** `clr query <pid> interrupt` — expect exit 0
- **Command 2 (after killing the session):** `clr query 999999 interrupt` — PID guaranteed not a running session
- **Expected behavior:** command 2 exits 1; stderr contains `"PID 999999 is not a running Claude Code session"` (same not-found contract `clr kill` uses) and a `"clr ps"` hint
- **Exit:** 0 (command 1) / 1 (command 2)
- **Source:** [command/10_query.md](../../../../docs/cli/command/10_query.md), [command/07_kill.md](../../../../docs/cli/command/07_kill.md)

---

### IT-4: `clr kill` terminates a query session

- **Setup:** start a query session
- **Command:** `clr kill <pid>`
- **Expected behavior:** exit 0; after a brief delay, `clr ps --pid <pid>` no longer lists the PID (or reports no active sessions)
- **Exit:** 0
- **Source:** [command/07_kill.md](../../../../docs/cli/command/07_kill.md), [command/10_query.md](../../../../docs/cli/command/10_query.md)

---

### IT-5: `--help` lists all 25 methods

- **Command:** `clr query --help`
- **Expected behavior:** exit 0; stdout contains all 25 camelCase method tokens (`interrupt`, `rewindFiles`, `setPermissionMode`, `setModel`, `setMaxThinkingTokens`, `applyFlagSettings`, `initializationResult`, `reinitialize`, `supportedCommands`, `supportedModels`, `supportedAgents`, `mcpServerStatus`, `accountInfo`, `reconnectMcpServer`, `toggleMcpServer`, `setMcpServers`, `streamInput`, `stopTask`, `setMcpPermissionModeOverride`, `getContextUsage`, `readFile`, `reloadPlugins`, `reloadSkills`, `seedReadState`, `backgroundTasks`)
- **Exit:** 0
- **Source:** [command/10_query.md](../../../../docs/cli/command/10_query.md)

---

### IT-6: Unknown method lists all 25 valid names

- **Setup:** start a query session
- **Command:** `clr query <pid> notARealMethod`
- **Expected behavior:** exit 1; stderr contains all 25 valid method tokens
- **Exit:** 1
- **Source:** [command/10_query.md](../../../../docs/cli/command/10_query.md)

---

### IT-7: `run` subcommand unaffected by `query` addition

- **Command 1:** `clr run --help` — expect exit 0, stdout still documents `run`
- **Command 2:** `clr --dry-run "a message"` — expect exit 0, existing preview output unaffected
- **Expected behavior:** both commands behave identically to their pre-task baseline
- **Exit:** 0
- **Source:** [command/01_run.md](../../../../docs/cli/command/01_run.md), [command/10_query.md](../../../../docs/cli/command/10_query.md)

---

### IT-8: Remaining 24 methods dispatch successfully

- **Setup:** start one shared query session
- **Command:** `clr query <pid> <method> [args...]` for each of the 24 remaining methods (`rewindFiles`, `setPermissionMode`, `setModel`, `setMaxThinkingTokens`, `applyFlagSettings`, `initializationResult`, `reinitialize`, `supportedCommands`, `supportedModels`, `supportedAgents`, `mcpServerStatus`, `accountInfo`, `reconnectMcpServer`, `toggleMcpServer`, `setMcpServers`, `streamInput`, `stopTask`, `setMcpPermissionModeOverride`, `getContextUsage`, `readFile`, `reloadPlugins`, `reloadSkills`, `seedReadState`, `backgroundTasks`)
- **Expected behavior:** every dispatch exits 0 against the `fake_claude_control` fixture, which accepts any request shape and responds per-subtype (or `null` for fire-and-forget subtypes); exact wire-shape mapping is TSK-415's own test surface, not this command's
- **Exit:** 0
- **Source:** [command/10_query.md](../../../../docs/cli/command/10_query.md)
