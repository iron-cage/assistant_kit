# Feature: Single Instance

### Scope

- **Purpose**: Guarantee that at most one daemon owns the hosted sessions at a time, and that a crashed daemon never blocks its own replacement.
- **In Scope**: `acquire`, `InstanceLock`, `DaemonPaths`, `Error::AlreadyRunning`.
- **Out of Scope**: What the daemon does once it holds the lock (→ [003_session_table.md](003_session_table.md)), the socket protocol (→ [002_wire_protocol.md](002_wire_protocol.md)).

### Behavior

`acquire( lock_path )` opens the lock file and takes an exclusive advisory lock on it with `flock( fd, LOCK_EX | LOCK_NB )`. Non-blocking is the point: a second daemon must fail immediately with `Error::AlreadyRunning` rather than wait for a lock that may be held for days.

The returned `InstanceLock` owns the file. Dropping it closes the descriptor, and the kernel releases the lock.

### Why Not a PID File

A PID file records an intention and enforces nothing:

| Failure | PID file | `flock` |
|---------|----------|---------|
| Daemon exits cleanly | File must be removed by the exit path | Released by the kernel |
| Daemon is `SIGKILL`ed | Stale file remains | Released by the kernel |
| PID recycled to another process | Next start adopts a stranger, or refuses to run | Not possible — the lock is on the file, not a number |
| Two daemons start simultaneously | Both read "no file", both write, both proceed | One wins the `flock`, the other gets `EWOULDBLOCK` |

The recycled-PID row is the one that matters most, and it is the same class of failure `claude_session_core` documents at length in [its liveness invariant](../../../claude_session_core/docs/invariant/001_liveness_four_clauses.md): **a bare PID number never identifies a process across time.** A PID file is that mistake written to disk.

`flock` is also atomic against a race between two starting daemons, which a read-then-write PID file is not.

### Paths

`DaemonPaths` resolves the three locations the daemon needs, all derived from Claude's home directory:

| Accessor | Path | Purpose |
|----------|------|---------|
| `runtime_dir()` | `<claude-home>/-daemon/` | Runtime state root |
| `lock_file()` | `<claude-home>/-daemon/instance.lock` | The advisory lock |
| `socket_file()` | `<claude-home>/-daemon/daemon.sock` | The listening socket |
| `sessions_dir()` | Claude Code's own sessions directory | Passed to `claude_session_core::scan` |

The runtime directory is hyphen-prefixed, so the workspace's global `-*` ignore rule keeps it out of version control. These files are machine-local: a lock file or socket path committed to a repository is meaningless on any other machine and actively confusing on the same one.

`DaemonPaths::new()` returns `None` when neither `CLAUDE_HOME` nor `HOME` is set. `DaemonPaths::with_home( path )` takes an explicit base — the form tests use, so a test never touches the developer's real Claude home.

### Verification

```bash
# Who holds the lock right now, if anyone:
fuser -v "${CLAUDE_HOME:-$HOME/.claude}/-daemon/instance.lock" 2>&1

# The second acquire fails rather than blocking:
cargo test -p claude_daemon_core --test lock_test
```

`tests/lock_test.rs` acquires a lock in a temporary directory, asserts a second `acquire` on the same path returns `AlreadyRunning`, then drops the first and asserts the second now succeeds.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/lock.rs` | `acquire` and `InstanceLock` |
| source | `src/paths.rs` | `DaemonPaths` |
| doc | [002_wire_protocol.md](002_wire_protocol.md) | What listens on the socket path |
| doc | [api/001_daemon_surface.md](../api/001_daemon_surface.md) | Full signature contract |
| test | `tests/lock_test.rs` | Exclusion, release-on-drop, and path resolution |
