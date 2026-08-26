# Feature: Wire Protocol

### Scope

- **Purpose**: Define what a client sends the daemon and what it gets back, in a shape a client written in any language can produce and parse.
- **In Scope**: `Request`, `Response`, `SessionSummary`, `read_capped_line`, `MAX_IPC_LINE_BYTES`.
- **Out of Scope**: Transport setup (the caller opens the socket), the line cap's rationale (→ [invariant/001_capped_line_reads.md](../invariant/001_capped_line_reads.md)), session naming (→ [invariant/002_conversation_id_key.md](../invariant/002_conversation_id_key.md)).

### Framing

One JSON object per line, in both directions. Newline-delimited JSON is chosen over a length-prefixed frame because it is debuggable with `nc` and `jq`, and because the existing `clr query` client already speaks it.

Lines are read with `read_capped_line`, which refuses at `MAX_IPC_LINE_BYTES` (1 MiB) rather than growing without bound.

### Requests

Internally tagged on `method`, in `snake_case`:

| Method | Fields | Effect |
|--------|--------|--------|
| `ping` | — | Liveness probe; returns the daemon's version |
| `list_sessions` | — | Every hosted session, as `SessionSummary` values |
| `spawn` | `cwd`, `prompt` (optional) | Start an interactive session in `cwd`; deliver `prompt` once ready |
| `send` | `session_id`, `text` | Deliver `text` to the session's stdin, followed by a carriage return |
| `resize` | `session_id`, `rows`, `cols` | Change the session's terminal dimensions |
| `shutdown` | `session_id` | Stop and reap the session |

```json
{ "method" : "send", "session_id" : "…", "text" : "what changed in this file?" }
```

`prompt` carries `#[ serde( default ) ]`, so omitting it is equivalent to `null`.

### Responses

```json
{ "ok" : true,  "result" : { … } }
{ "ok" : false, "error"  : "no such session: …" }
```

The `ok` discriminant is explicit rather than an externally tagged enum, so a client written against the earlier `clr query` shape reads the daemon's replies unchanged. Preserving that shape costs two marker types (`OkTrue`, `OkFalse`) with hand-written `Serialize`/`Deserialize` impls; the alternative was breaking every existing client to save them.

`Response::ok( value )` and `Response::err( message )` are the constructors.

### `SessionSummary`

What `list_sessions` returns per session:

| Field | Type | Meaning |
|-------|------|---------|
| `session_id` | `String` | Conversation id — the handle for every other request |
| `pid` | `u32` | Current process id; changes across a `--fork-session` re-host |
| `cwd` | `PathBuf` | Working directory |
| `busy` | `bool` | Whether the daemon believes a turn is in flight |

`pid` is reported for diagnostics — correlating against `ps`, or against `claude_session_core`'s registry scan — and is never an address. See [invariant/002_conversation_id_key.md](../invariant/002_conversation_id_key.md).

### What This Generalizes

`clr query` established this response shape against a **per-PID socket**: one socket per session, so the session was identified by which socket you connected to. Two things changed:

1. **One socket, many sessions.** A single daemon hosts everything, so the session is named *inside* the request.
2. **Named by conversation id, not PID.** A re-host changes the PID; the conversation id is what a client can keep holding.

### Verification

```bash
# Round-trip every request and response variant through serde:
cargo test -p claude_daemon_core --test protocol_test

# By hand, against a running daemon:
printf '{"method":"ping"}\n' | nc -U "${CLAUDE_HOME:-$HOME/.claude}/-daemon/daemon.sock"
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/protocol.rs` | `Request`, `Response`, `SessionSummary` |
| source | `src/ipc.rs` | `read_capped_line` |
| doc | [invariant/001_capped_line_reads.md](../invariant/001_capped_line_reads.md) | Why reads are capped |
| doc | [invariant/002_conversation_id_key.md](../invariant/002_conversation_id_key.md) | Why sessions are named by conversation id |
| doc | [api/001_daemon_surface.md](../api/001_daemon_surface.md) | Full signature contract |
| test | `tests/protocol_test.rs` | Round-trips and the `ok` discriminant shape |
| test | `tests/ipc_test.rs` | Framing, the cap, and trailing-`\r` handling |
