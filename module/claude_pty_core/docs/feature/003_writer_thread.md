# Feature: Writer Thread

### Scope

- **Purpose**: Let a caller send input to a hosted child without ever blocking its own event loop on that child.
- **In Scope**: `WriterHandle::spawn`, `WriterHandle::send`, `WriterHandle::shutdown`, `DEFAULT_QUEUE_CAPACITY`, `Error::WriterFull`, `Error::WriterGone`.
- **Out of Scope**: Reading from the master (the caller owns the reader — see [002_session_spawn.md](002_session_spawn.md)), what to send (→ `claude_daemon_core`).

### Behavior

`WriterHandle::spawn( sink, capacity )` starts a thread that drains a bounded queue into `sink` — normally a duplicate of the PTY master. `send( bytes )` copies the bytes onto the queue and returns immediately.

### Why a thread

Writing to a PTY master blocks once the kernel's input buffer fills, and that buffer fills whenever the child stops draining stdin — because it is busy, stopped, or hung. A synchronous `write_all` from a caller's event loop stalls the whole loop on an unresponsive child: no reads, no timers, no shutdown. With a dedicated thread, only that thread blocks.

This is the same conclusion reached independently by other supervisors of interactive Claude Code sessions: the writer thread is not an optimization, it is what keeps one wedged session from taking the supervisor down with it.

### Why the queue is bounded

An unbounded queue converts a stalled child into unbounded memory growth — the same outage, arriving later and harder to diagnose. At capacity, `send` returns `Error::WriterFull` and the caller decides what that means: drop the input, surface backpressure to its own client, or kill the session. The library does not choose on the caller's behalf, because the right answer differs between a keystroke and a queued command.

`DEFAULT_QUEUE_CAPACITY` is 256 messages, sized for interactive typing and paste bursts rather than bulk transfer. A child that has not drained 256 queued writes is not slow; it is stuck.

### Failure Modes

| Condition | Result |
|-----------|--------|
| Queue at capacity | `Error::WriterFull` — child has stopped reading stdin |
| Writer thread has exited | `Error::WriterGone` — the sink closed or the thread was joined |
| Handle dropped without `shutdown` | Queue closes, thread drains and exits, not joined |
| `shutdown()` called | Queue closes, thread drains and exits, joined |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/writer.rs` | `WriterHandle` and the queue |
| source | `src/session.rs` | `PtySession::write` delegates here |
| doc | [002_session_spawn.md](002_session_spawn.md) | Where the sink descriptor comes from |
| doc | [api/001_pty_surface.md](../api/001_pty_surface.md) | Full signature contract |
| test | `tests/writer_test.rs` | Delivery, capacity rejection, and shutdown ordering |
