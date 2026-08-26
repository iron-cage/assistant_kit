# Feature Doc Entity

### Scope

- **Purpose**: Document user-facing capabilities of the `claude_daemon_core` library for consumers building a client or an executable around the session daemon.
- **Responsibility**: Index of feature doc instances covering the single-instance guarantee, the line-framed wire protocol, and the hosted-session table.
- **In Scope**: `InstanceLock` and `DaemonPaths`, `Request`/`Response` and `read_capped_line`, `SessionTable` and `HostedSession`.
- **Out of Scope**: Terminal mechanics (→ `claude_pty_core/docs/feature/`), liveness and turn boundaries (→ `claude_session_core/docs/feature/`), the CLI surface that drives the daemon (→ `claude_runner/docs/`), invariant constraints (→ `invariant/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Single Instance](001_single_instance.md) | Exactly one daemon, enforced by an advisory lock | ✅ |
| 002 | [Wire Protocol](002_wire_protocol.md) | One JSON object per line, in both directions | ✅ |
| 003 | [Session Table](003_session_table.md) | Hosted sessions, addressed by conversation id | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating feature doc instances | ✅ |
