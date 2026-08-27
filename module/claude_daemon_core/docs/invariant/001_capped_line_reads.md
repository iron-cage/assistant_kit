# Invariant: Capped Line Reads

### Scope

- **Purpose**: Guarantee that a peer which never sends a newline cannot exhaust the daemon's memory.
- **Governs**: `read_capped_line` in `src/ipc.rs`, and every path that reads a protocol line.
- **In Scope**: All reads from a client socket.
- **Out of Scope**: The size of a *response* the daemon produces; output is bounded by what it has to say, not by a peer.

### Rule

No read on the protocol path may use `BufRead::read_line` or any other unbounded accumulator. Every line is read through `read_capped_line`, which refuses with `Error::LineTooLong` once the accumulated length would exceed `MAX_IPC_LINE_BYTES` (1 MiB).

The check happens **before** each `extend_from_slice`, against the length that would result — not after appending. Checking afterward means the offending bytes were already allocated, which is the outcome the cap exists to prevent.

### Rationale

The `clr query` prototype this generalizes reads its socket with a bare `read_line`, which grows until a newline arrives. That was defensible: each session had its own helper process, so a peer that never terminated a line took down one helper and nothing else.

Consolidation changed the blast radius, not the behavior. With one daemon hosting every session, an unbounded read is no longer one session's problem — a single malformed or malicious peer takes down every session at once. The invariant exists because **the same code became unacceptable when the process it runs in started mattering more.**

1 MiB is far above any legitimate request. The largest realistic line is a `send` carrying pasted text; a megabyte of pasted text into a terminal is not a request, it is an accident or an attack, and either way refusing it is correct.

### Two Details

**`fill_buf`/`consume`, not `read_line`.** The implementation inspects the buffered slice, decides how much to take, and consumes exactly that. This is what makes the cap checkable at all — `read_line` gives no opportunity to intervene mid-line.

**The newline is consumed but never stored.** It is a delimiter, not content. `finish` additionally trims a trailing `\r`, so a client sending CRLF gets the same result as one sending LF.

### Verification

```bash
cargo test -p claude_daemon_core --test ipc_test
```

Directly — a peer that opens the socket and never sends a newline should be refused, not tolerated:

```bash
# Sends 2 MiB with no newline; expect the daemon to answer with an error and
# close, not to grow.
head -c 2097152 /dev/zero | tr '\0' 'x' \
  | nc -U "$HOME/.claude/-daemon/daemon.sock"
```

`tests/ipc_test.rs` feeds a reader that produces bytes without a newline and asserts `Error::LineTooLong` rather than growth.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/ipc.rs` | `read_capped_line` and `MAX_IPC_LINE_BYTES` |
| doc | [feature/002_wire_protocol.md](../feature/002_wire_protocol.md) | The framing this bounds |
| doc | [api/001_daemon_surface.md](../api/001_daemon_surface.md) | Signature contract |
| test | `tests/ipc_test.rs` | Cap enforcement, CRLF trimming, EOF handling |
