# Feature: Wire Protocol

### Scope

- **Purpose**: Define what a client sends the daemon and what it gets back, in a shape a client written in any language can produce and parse.
- **In Scope**: `Request`, `Response`, `SessionSummary`, `OutputSlice` on the wire, `read_capped_line`, `MAX_IPC_LINE_BYTES`.
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
| `read` | `session_id`, `cursor` (optional) | Everything the session produced since `cursor`, as an `OutputSlice` |
| `context_summary` | `session_id` | What the session's context currently holds, folded from its transcript |
| `resize` | `session_id`, `rows`, `cols` | Change the session's terminal dimensions |
| `shutdown` | `session_id` | Stop and reap the session |
| `stop_daemon` | — | Answer, then shut every session down and stop the daemon |

```json
{ "method" : "send", "session_id" : "…", "text" : "what changed in this file?" }
{ "method" : "read", "session_id" : "…", "cursor" : 8192 }
```

`prompt` and `cursor` carry `#[ serde( default ) ]`, so omitting them is equivalent to `null` and `0`. An omitted cursor means "everything still retained", which is what a client attaching to a session for the first time wants.

`stop_daemon` is a request rather than a signal because a signal tells the sender nothing. `SIGTERM` is fire-and-hope: it does not say whether the process it reached was this daemon, whether the daemon had already exited, or whether the sessions came down cleanly. A request is answered on the connection it arrived on, before the daemon acts on it — see [006_serving_clients.md](006_serving_clients.md).

`send` and `read` are separate calls, and deliberately so. A `send` that blocked until the turn settled would hold the daemon's accept loop for the whole duration of that turn, freezing every other session behind it. Instead `send` returns as soon as the text is queued, and the client polls `read` — which is also what makes the print-mode UX (prompt → output → prompt returns) something a client can build rather than something the protocol has to impose.

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

### `OutputSlice`

What `read` returns:

| Field | Type | Meaning |
|-------|------|---------|
| `text` | `String` | Output decoded since the requested cursor |
| `cursor` | `u64` | Where to read from next |
| `missed` | `u64` | Bytes evicted before the requested cursor reached them |
| `ended` | `bool` | Whether the session's output has ended |

Reads are non-destructive: two clients watching one session each hold their own cursor and neither takes the other's output. Buffering, eviction, and how a character split across a read boundary is handled are [004_session_output.md](004_session_output.md).

### What This Generalizes

`clr query` established this response shape against a **per-PID socket**: one socket per session, so the session was identified by which socket you connected to. Two things changed:

1. **One socket, many sessions.** A single daemon hosts everything, so the session is named *inside* the request.
2. **Named by conversation id, not PID.** A re-host changes the PID; the conversation id is what a client can keep holding.

### Verification

```bash
# Round-trip every request and response variant through serde:
cargo test -p claude_daemon_core --test protocol_test

# By hand, against a running daemon:
printf '{"method":"ping"}\n' | nc -U "$HOME/.claude/-daemon/daemon.sock"
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/protocol.rs` | `Request`, `Response`, `SessionSummary` |
| source | `src/output.rs` | `OutputSlice` |
| source | `src/ipc.rs` | `read_capped_line` |
| doc | [004_session_output.md](004_session_output.md) | What sits behind `read` |
| doc | [invariant/001_capped_line_reads.md](../invariant/001_capped_line_reads.md) | Why reads are capped |
| doc | [invariant/002_conversation_id_key.md](../invariant/002_conversation_id_key.md) | Why sessions are named by conversation id |
| doc | [api/001_daemon_surface.md](../api/001_daemon_surface.md) | Full signature contract |
| test | `tests/protocol_test.rs` | Round-trips and the `ok` discriminant shape |
| test | `tests/ipc_test.rs` | Framing, the cap, and trailing-`\r` handling |
