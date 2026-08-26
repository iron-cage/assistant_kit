# Feature: Session Output

### Scope

- **Purpose**: Keep every hosted session's terminal drained, and let any number of clients read what it produced without consuming it from one another.
- **In Scope**: `OutputBuffer`, `OutputPump`, `OutputSlice`, `DEFAULT_OUTPUT_CAP`, `HostedSession::read_from`, the `read` request.
- **Out of Scope**: Allocating the PTY and cloning its master (→ `claude_pty_core`), deciding a turn has ended (→ `claude_session_core`), how a client renders a slice (→ `claude_runner`).

### Why This Exists

A PTY master that nobody reads is not idle — it fills. Once the kernel's buffer is full the child blocks on its next write, and a blocked child is indistinguishable from a thinking one: no output, no error, no exit. So a hosted session must be drained continuously, whether or not a client is currently interested.

That forces the two halves of this feature apart. Draining has to happen on the daemon's schedule; reading happens on the client's. Between them sits a buffer.

### Behavior

| Piece | Contract |
|-------|----------|
| `OutputPump::spawn( reader, cap )` | Starts a thread reading the master until it ends, appending into a buffer retaining `cap` bytes |
| `OutputBuffer::push( bytes )` | Appends, evicting from the front once `cap` is exceeded |
| `read_from( cursor )` | Returns an `OutputSlice`; does not consume |
| `mark_ended()` | Records that the stream ended; every later read reports it |

Reads are **non-destructive**. A cursor is a position, not a claim: two clients watching one session each hold their own, and neither takes the other's output. Re-reading the same cursor returns the same bytes; reading from the cursor a slice returned yields only what arrived since.

### `OutputSlice`

| Field | Type | Meaning |
|-------|------|---------|
| `text` | `String` | Output decoded since the requested cursor |
| `cursor` | `u64` | Where to read from next |
| `missed` | `u64` | Bytes evicted before the requested cursor could reach them |
| `ended` | `bool` | Whether the stream has ended |

`missed` is on the wire rather than inferred client-side, because only the daemon knows how much it dropped. A client that renders a gap as continuous output is worse than one that prints a warning: the transcript looks whole and is not.

Cursors are absolute byte positions in the session's whole output, counting evicted bytes. A cursor past the newest byte is corrected to the end rather than trusted — it arrives from a client that saved one across a daemon restart, and treating a large number as an offset would panic.

### Bounded Retention

`DEFAULT_OUTPUT_CAP` is 256 KiB per session. The daemon hosts many sessions and runs indefinitely; unbounded retention makes memory a function of uptime. The cap is what makes it a function of session count instead.

Eviction is reported, never silent — that is what `missed` is for.

### Character Boundaries

A terminal emits bytes; a slice carries text. Two things can cut a multi-byte character in half, and they need opposite answers:

| Cut by | Answer | Why |
|--------|--------|-----|
| A chunk boundary (the rest has not been read yet) | Withhold the trailing bytes | They complete on the next read; replacing them corrupts undamaged text |
| Eviction (the leading bytes are gone for good) | Emit `U+FFFD` and step past | Nothing can complete them, so withholding stalls the cursor forever |

Getting these backwards produces mojibake that appears only under load.

### Draining Outlives the Reader

The pump holds a *clone* of the PTY master, obtained through `PtySession::take_reader`. A session ends when the **last** master descriptor closes, so while the pump lives the child never sees a hangup — and `PtySession::shutdown` waits for a child that is waiting for it.

This is why `HostedSession`'s fields are private and its teardown is a fixed ladder (→ [003_session_table.md](003_session_table.md)). The pump's descriptor is released only when its read ends, which happens when the child's own descriptors close.

### Verification

```bash
cd module/claude_daemon_core && ./verb/test
```

Or the two suites directly, inside the container:

```bash
cargo nextest run -p claude_daemon_core --test output_test   # buffer arithmetic, no threads
cargo nextest run -p claude_daemon_core --test table_test    # the pump against a real child
```

`output_test.rs` pins cursors, eviction accounting, and both character-boundary cases exactly. `table_test.rs` covers the pump end-to-end, including that output survives a round trip through a cursor and that a read after shutdown reports `ended`.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/output.rs` | `OutputBuffer`, `OutputPump`, `OutputSlice` |
| source | `src/table.rs` | `HostedSession::read_from`, the teardown ladder |
| doc | [002_wire_protocol.md](002_wire_protocol.md) | The `read` request and the slice on the wire |
| doc | [003_session_table.md](003_session_table.md) | Why the session owns its pump |
| doc | [../../../claude_pty_core/docs/feature/002_session_spawn.md](../../../claude_pty_core/docs/feature/002_session_spawn.md) | Why a held clone of the master keeps a session alive |
| test | `tests/output_test.rs` | Cursors, eviction, UTF-8 boundaries |
| test | `tests/table_test.rs` | Pump, cursor round-trip, end of stream |
