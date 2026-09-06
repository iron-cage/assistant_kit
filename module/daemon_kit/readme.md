# daemon_kit

Generic single-instance Unix-socket daemon skeleton. Knows nothing about what
a daemon built on it is for.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `Cargo.toml` | Crate manifest and dependency configuration |
| `src/lock.rs` | Advisory-`flock` single-instance guarantee |
| `src/listener.rs` | Socket bind, stale-socket cleanup, accept |
| `src/ipc.rs` | Capped newline-delimited line framing |
| `src/response.rs` | The `{ok, result}` / `{ok, error}` answer envelope |
| `src/client.rs` | One-connection-per-request client, generic over the request type |
| `src/serve.rs` | `serve_connection`/`serve_once`, and the waker that gives a blocking accept loop a clock |
| `src/error.rs` | Crate-wide error type |
| `tests/` | Lock exclusion, socket lifecycle, framing, and envelope round-trips |

## overview

Extracted from `claude_daemon_core` (see its `docs/feature/001_single_instance.md`,
`002_wire_protocol.md`, and `006_serving_clients.md` for the full behavioral
rationale — this crate is the mechanism those documents describe, minus
anything Claude-specific). A daemon built on this crate supplies its own
request type and dispatch closure; everything else — the lock, the socket, the
framing, the client, the loop body — is here.

## composes into

`claude_daemon_core` depends on this crate for the parts of its daemon that
have nothing to do with hosting Claude Code sessions specifically.
