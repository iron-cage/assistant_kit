# Feature Doc Entity

### Scope

- **Purpose**: Document user-facing capabilities of the `claude_daemon_core` library for consumers building a client or an executable around the session daemon.
- **Responsibility**: Index of feature doc instances covering the single-instance guarantee, the line-framed wire protocol, the hosted-session table, session output, learning a session's conversation id, answering clients on the socket, rendering a session's output readably, and reporting whether a turn is in flight.
- **In Scope**: `InstanceLock` and `DaemonPaths`, `Request`/`Response` and `read_capped_line`, `SessionTable` and `HostedSession`, `OutputPump` and `OutputSlice`, `await_session_id`, `Listener` and `Daemon` and `client`, `render::to_plain_text`, `Daemon::with_background_reporting`.
- **Out of Scope**: Terminal mechanics (→ `claude_pty_core/docs/feature/`), liveness and turn boundaries (→ `claude_session_core/docs/feature/`), the CLI surface that drives the daemon (→ `claude_runner/docs/`), invariant constraints (→ `invariant/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Single Instance](001_single_instance.md) | Exactly one daemon, enforced by an advisory lock | ✅ |
| 002 | [Wire Protocol](002_wire_protocol.md) | One JSON object per line, in both directions | ✅ |
| 003 | [Session Table](003_session_table.md) | Hosted sessions, addressed by conversation id | ✅ |
| 004 | [Session Output](004_session_output.md) | Output kept drained, retained in bounds, read by cursor | ✅ |
| 005 | [Session Registration](005_session_registration.md) | Learning the conversation id of a session just spawned | ✅ |
| 006 | [Serving Clients](006_serving_clients.md) | The socket, one request per connection, and what each means | ✅ |
| 007 | [Readable Output](007_readable_output.md) | A terminal's output stream, rendered as a transcript | ✅ |
| 008 | [Turn State](008_turn_state.md) | Whether a turn is in flight, and the guarantee `idle` needs | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating feature doc instances | ✅ |
