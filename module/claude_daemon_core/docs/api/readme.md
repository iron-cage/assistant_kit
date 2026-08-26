# API Doc Entity

### Scope

- **Purpose**: Document the programmatic interface of the `claude_daemon_core` library surface.
- **Responsibility**: Index of API doc instances covering the lock, paths, protocol, framing, and table exports.
- **In Scope**: `acquire`, `InstanceLock`, `DaemonPaths`, `Request`, `Response`, `SessionSummary`, `read_capped_line`, `MAX_IPC_LINE_BYTES`, `SessionTable`, `HostedSession`, `Error`, `Result`.
- **Out of Scope**: CLI behavior (this crate has no binary — the executable lives in `claude_runner`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Daemon Surface](001_daemon_surface.md) | Signature contract for every exported item | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating API doc instances | ✅ |
