# CLI Command: query

### Description

Start a persistent, PID-addressed bidirectional control session, or dispatch one of 25
SDK control methods against an already-running session. Use `clr query "<message>"` to
start a session (prints a PID immediately, like `clr run`'s backgrounded default), then
`clr query <pid> <method>` to interrupt it, change its model or permission mode, inspect
its context usage, and more — all without blocking on the session's lifetime.

-- **Parameters:** `<MESSAGE>` / `<PID> <METHOD> [ARGS...]`, `--dir`
-- **Exit Codes:** 0 (success) | 1 (error)
-- **Forms:** start (prints PID, returns immediately) | dispatch (PID + method call)

### Syntax

```sh
clr query "<MESSAGE>" [--dir <PATH>]
clr query <PID> <METHOD> [ARGS...]
```

### Parameters

| # | Name | Required | Purpose |
|---|------|----------|---------|
| 1 | `<MESSAGE>` | Yes (start form) | Initial message sent to the new control session |
| 2 | `--dir` | No | Working directory for the spawned `claude` subprocess |
| 3 | `<PID>` | Yes (dispatch form) | PID of an already-running query session (from the start form's stdout, or `clr ps`) |
| 4 | `<METHOD>` | Yes (dispatch form) | One of 25 camelCase control-method tokens (see Methods below) |
| 5 | `[ARGS...]` | Method-dependent | Positional arguments for the chosen method |

**Form detection:** the first token after `query` is parsed as a PID (`u32`); if it parses
successfully, the dispatch form is used (second token required as `<METHOD>`). Otherwise
it is treated as the start form's `<MESSAGE>`.

**Methods (25, camelCase, matching the SDK's own spelling — not `ControlSession`'s Rust
snake_case names):**

`interrupt`, `rewindFiles`, `setPermissionMode`, `setModel`, `setMaxThinkingTokens`,
`applyFlagSettings`, `initializationResult`, `reinitialize`, `supportedCommands`,
`supportedModels`, `supportedAgents`, `mcpServerStatus`, `accountInfo`,
`reconnectMcpServer`, `toggleMcpServer`, `setMcpServers`, `streamInput`, `stopTask`,
`setMcpPermissionModeOverride`, `getContextUsage`, `readFile`, `reloadPlugins`,
`reloadSkills`, `seedReadState`, `backgroundTasks`.

Five (`initializationResult`, `supportedCommands`, `supportedModels`, `supportedAgents`,
`accountInfo`) are cached accessors — they return the daemon's cached initialize-result
state rather than performing a fresh wire round-trip. `streamInput` is write-only: it
resolves on successful write, with no control-response wire shape to await.

**Algorithm — start form (4 steps):**
1. Spawn a detached `__query_daemon` child process (piped stdout/stderr, null stdin).
2. Read exactly one line from the child's stdout — the underlying `claude` subprocess's PID, discovered by diffing `find_claude_processes()` before/after spawning the control session.
3. Print that PID and exit 0; the daemon keeps running after this process exits (reparented to init).
4. On startup failure, relay the daemon's stderr and exit 1.

**Algorithm — dispatch form (4 steps):**
1. Validate `<METHOD>` against the 25-name table; unknown method → exit 1 listing all valid names.
2. Connect to the Unix socket at `<query-dir>/<PID>.sock`; connection failure → exit 1 with the same not-a-running-session contract `clr kill` uses.
3. Send `{"method":..., "args":[...]}`, read back one `{"ok":bool,...}` response line.
4. On `"ok":true`, pretty-print `result` and exit 0; on `"ok":false` or a malformed/missing response, print the error and exit 1.

### Examples

```sh
# Start a query session
clr query "Refactor the auth module"
# -> prints a PID, e.g.: 82931

# Start in a specific working directory
clr query "Fix the failing test" --dir /path/to/project

# Interrupt the session's current turn
clr query 82931 interrupt

# Switch permission mode mid-session
clr query 82931 setPermissionMode acceptEdits

# Check context usage
clr query 82931 getContextUsage

# List active sessions, including query sessions
clr ps

# Terminate a query session the same way as any other
clr kill 82931

# Show query-specific help (lists all 25 methods)
clr query --help
```

### Notes

**Every query session starts with `PermissionMode::BypassPermissions` hardcoded** — the
same permission mode `spawn_control_session()`'s captured reference invocation uses
(`tests/fixtures/sdk_control_capture/argv.json`, TSK-415 Phase 0). `setPermissionMode` can
change this after the fact, but the session begins with all permission prompts bypassed.

Each query session is served by a detached `__query_daemon` process — a hidden
subcommand of the same `clr` binary, not user-invocable directly — that holds the one
`ControlSession` for the session's entire lifetime and relays method calls over a
PID-keyed Unix domain socket. The socket directory defaults to `<temp-dir>/clr-query/`
and can be overridden with the `CLR_QUERY_DIR` environment variable (mirrors `clr ps`'s
`CLR_GATE_DIR` convention) — used in tests to isolate socket I/O from real system state.

The daemon binds its socket **before** printing the session's PID, specifically so a
caller that immediately dispatches `clr query <pid> <method>` right after reading the
printed PID cannot race the daemon's own startup.

A background liveness watchdog polls every 500ms; once the underlying `claude` subprocess
is no longer live (per `find_claude_processes()`, the same `/proc` scan `ps`/`kill` use),
the daemon removes its socket file and exits — a query session cannot outlive its
subprocess as an orphaned, unreachable socket.

`query` relies on `find_claude_processes()` (Linux `/proc` scanning, same as `ps`/`kill`)
and Unix domain sockets (`std::os::unix::net`), so it is not available on non-Linux
platforms.

Exact per-method argument shapes beyond the bare method token were finalized against
TSK-415's Phase 0 captured wire trace, not invented independently of it — see
`module/claude_runner_core/tests/fixtures/sdk_control_capture/` for the ground truth.

**Error messages:**
- `Error: 'clr query' requires a message or a PID + method.` — No arguments given.
- `Error: 'clr query <pid>' requires a method name.` — PID given without a method.
- `Error: unknown query method '<name>'.` — Method not in the 25-name table; stderr lists all valid names.
- `Error: PID <N> is not a running Claude Code session.` — Same not-found contract as `clr kill`.
- `Error: failed to start control session: <reason>` / `Error: query daemon failed to start[: <detail>]` — Session startup failed.
- `Error: no response from PID <N>.` / `Error: malformed response from PID <N>.` — Daemon connection or protocol failure.

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 1 | [`ps`](06_ps.md) | Lists query sessions, distinguishable from `run`/`ask` sessions |
| 2 | [`kill`](07_kill.md) | Terminates a query session the same way as any other session |
| 3 | [`run`](01_run.md) | Shares the backgrounded-by-default session model query's start form mirrors |

### Referenced Parameter Groups

None. `--dir` here is `query`'s own minimal, independently-parsed flag (not routed through
the Runner Control group's shared argument parser `run`/`ask`/`isolated` use) — see
[`--dir`](../param/008_dir.md) for the conceptually equivalent Runner Control parameter.

### Referenced User Stories

*None — dedicated bidirectional-control user story not yet filed; task 418 introduces the
CLI surface for TSK-415's control-session capability.*

---

**Category:** Session management
**Complexity:** 20
**API Requirement:** Write
**Idempotent:** No
**Risk Level:** High
