# Feature: Session Spawn

### Scope

- **Purpose**: Start a child process on the slave side of a PTY, as the leader of its own session, with the terminal as its controlling terminal — the state an interactive program checks for before it will run.
- **In Scope**: `SessionConfig` (program, args, env, cwd, window size, queue capacity), `PtySession::spawn`, `PtySession::pid`, `PtySession::take_reader`, `PtySession::try_wait`, `PtySession::shutdown`, environment scrubbing.
- **Out of Scope**: Allocating the PTY itself (→ [001_pty_allocation.md](001_pty_allocation.md)), write queueing (→ [003_writer_thread.md](003_writer_thread.md)), deciding when the child has finished a turn (→ `claude_session_core`).

### Behavior

`PtySession::spawn` allocates a PTY, sizes it, opens three slave descriptors, scrubs the environment, and starts the child with the terminal already attached.

**Three slave descriptors, not one.** `stdin`, `stdout`, and `stderr` each receive their own `open_slave()` result. A single descriptor shared across all three would mean closing any one closes all three, since they would be the same open file description.

**The parent keeps none of them.** Each slave descriptor is moved into a `Stdio` and closed in the parent by `spawn`. This is load-bearing rather than tidy: while the parent still holds any slave open, the terminal has a writer, so reads from the master never reach EOF. An exited child would then be indistinguishable from a live one that simply has nothing to say — the reader would block forever waiting for output that can never come.

**Controlling terminal.** `attach_controlling_terminal` registers a `pre_exec` closure that calls `setsid()` and then `ioctl( 0, TIOCSCTTY, 0 )`. It targets file descriptor 0 rather than a captured slave descriptor because `std` performs the stdio `dup2` calls *before* running any `pre_exec` closure — by the time the closure runs, fd 0 already is the slave. Capturing the original descriptor would additionally have meant keeping it alive past the point where `Stdio::from` consumed it.

`setsid()` must come first: a process that is already a session leader cannot acquire a new controlling terminal, and a process still in the parent's session cannot either.

### Environment Scrubbing

The child inherits the parent's environment minus:

- every variable whose name begins with `CLAUDE_` — markers the parent set for itself must not be read by the child as its own
- terminal identity variables that describe the *parent's* terminal emulator, not the PTY: `COLORTERM`, `ITERM_SESSION_ID`, `TERM_PROGRAM`, `TERM_PROGRAM_VERSION`, `TMUX`, `TMUX_PANE`, `WEZTERM_PANE`, `WEZTERM_UNIX_SOCKET`

`TERM` is then set to `xterm-256color` — a description of the PTY the child actually has. Scrubbing must be broader than the `CLAUDE_` prefix alone: a program that finds `TMUX` set will address escape sequences to a multiplexer that is not there.

Caller-supplied `SessionConfig::env` entries are applied *after* scrubbing, so a caller can deliberately reinstate a scrubbed variable.

### Lifecycle

- `pid()` — the child's process id, for correlating against an external registry
- `take_reader()` — hands the master read half to the caller exactly once; a second call returns `None`
- `try_wait()` — non-blocking exit check
- `shutdown()` — closes every master descriptor the session owns, then blocks for the child's exit status

Shutdown is a descriptor problem, not a signal problem. The session hands out three clones of the master — one to the writer thread, one to its own reader slot, one held by the `Pty` itself — and the child's stdin reads only reach `EOF` once the *last* of them closes. Stopping the writer thread alone leaves two open, so a child blocked reading stdin never returns and the wait never completes. `shutdown` therefore drops all three in order, and a reader the caller took with `take_reader` is theirs to drop: while it lives, so does the session.

`resize` after shutdown returns `Error::SessionClosed` rather than a stale success — there is no master left to issue `TIOCSWINSZ` against. `slave_path()` keeps working, because a shutdown is precisely the event a log line needs to name.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/session.rs` | `SessionConfig` and `PtySession` |
| source | `src/env_scrub.rs` | The scrub list and `TERM` replacement |
| source | `src/ffi.rs` | `attach_controlling_terminal` |
| doc | [001_pty_allocation.md](001_pty_allocation.md) | The pair this spawns onto |
| doc | [api/001_pty_surface.md](../api/001_pty_surface.md) | Full signature contract |
| test | `tests/session_test.rs` | Spawns a real child and reads its output back |
| test | `tests/env_scrub_test.rs` | Scrub-list membership and `TERM` replacement |
