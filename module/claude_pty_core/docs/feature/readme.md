# Feature Doc Entity

### Scope

- **Purpose**: Document user-facing capabilities of the `claude_pty_core` library for consumers that need to host an interactive terminal program.
- **Responsibility**: Index of feature doc instances covering PTY pair allocation, spawning a child onto the slave side, and the writer thread that decouples callers from a blocking master descriptor.
- **In Scope**: `Pty` allocation and resize, `SessionConfig`/`PtySession` spawning, `WriterHandle` queueing, environment scrubbing applied to the child.
- **Out of Scope**: Deciding *what* to run on the PTY or when (→ `claude_daemon_core/docs/feature/`), reading a session's liveness or turn state (→ `claude_session_core/docs/feature/`), invariant constraints (→ `invariant/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [PTY Allocation](001_pty_allocation.md) | Allocate a master/slave pair and control its window size | ✅ |
| 002 | [Session Spawn](002_session_spawn.md) | Put a child process on the slave side as its controlling terminal | ✅ |
| 003 | [Writer Thread](003_writer_thread.md) | Queue writes so a full terminal buffer never blocks the caller | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating feature doc instances | ✅ |
