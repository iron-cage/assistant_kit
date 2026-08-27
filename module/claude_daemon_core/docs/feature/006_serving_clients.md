# Feature: Serving Clients

### Scope

- **Purpose**: Accept a client on the Unix socket, turn its request into an answer, and send that answer back.
- **In Scope**: `Listener`, `Daemon::dispatch`, `serve_connection`, `serve_once`, `client::request`, `client::call`.
- **Out of Scope**: The request and response types (→ [002_wire_protocol.md](002_wire_protocol.md)), the line framing they travel in (→ `src/ipc.rs`), what a session *is* (→ [003_session_table.md](003_session_table.md)), the daemon binary's main loop and its lifetime (→ `claude_daemon`).

### The Socket

`Listener::bind` takes the instance lock as an argument, not as a comment. Binding has to unlink whatever is already at the path — `bind(2)` fails with `AddrInUse` on an existing path whether or not anything is listening on it, and a daemon killed with `SIGKILL` leaves exactly that behind. Unlinking is only safe if no other daemon is listening, and the lock is the entire basis for believing so.

Which is why the lock is checked, not just accepted: a lock held over some other directory proves nothing about this socket, and `Error::LockMismatch` says so.

| Already at the path | What happens |
|---------------------|--------------|
| Nothing | Bind |
| A socket | Unlink, then bind — the crashed-daemon case |
| Anything else | `Error::Io`, and the file is left alone |

The last row is the limit of what the lock covers. It is evidence about processes; it says nothing about a regular file that happens to share the name, and deleting one is not a thing a function called `bind` should do.

`Listener` removes its own path on drop, because `UnixListener` does not. A socket file that outlives its daemon makes the next client fail with `ECONNREFUSED`, which reads as "the daemon is broken" rather than "the daemon is not running" — an hour of debugging the wrong thing.

The socket is chmod'ed to `0600` after binding. The runtime directory is the real boundary; this narrows what gets through it if that directory is wider than it should be.

### One Request Per Connection

Not a limitation to be worked around later. A single-threaded daemon serving *persistent* connections is a single-**client** daemon: whoever connects first decides when everybody else gets served, and one client that stops reading stops the whole daemon.

Closing after one request bounds a client's hold on the daemon to the request it actually sent. Connection setup on a Unix socket costs microseconds; a session's turn costs seconds to minutes. The ratio is not close.

### Dispatch

`Daemon::dispatch` is infallible by construction. Every failure inside it becomes a `Response::err`, because a client that sent a request is owed an answer either way — a dropped connection is indistinguishable, from the client's side, from a daemon that died.

| Method | Result |
|--------|--------|
| `ping` | `{ "version": … }` |
| `list_sessions` | An array of `SessionSummary` |
| `spawn` | `{ "session_id": … }`, after the session registers |
| `send` | `{ "cursor": … }` — where this turn's output begins |
| `read` | An `OutputSlice` |
| `resize` | `null` |
| `shutdown` | `{ "exit_code": … }` |
| `stop_daemon` | `{ "stopping": true }`, and `stop_requested()` becomes true |

`stop_daemon` sets a flag and nothing else. Tearing sessions down inside the request would spend an unbounded part of the client's wait on children that may be slow to die, and would leave the client unable to tell "stopping" from "hung". The main loop checks `stop_requested()` after `serve_once` returns — that is, after the answer is already on the wire — and only then calls `shutdown_all`.

`try_dispatch` has no catch-all arm. `Request` is `#[non_exhaustive]`, but that binds *downstream* crates — inside the crate that defines it the match is exhaustive, so a variant added later stops the build instead of silently reaching a default that answers it wrongly.

### Nothing Blocks on a Turn

`send` returns as soon as the text is queued. It does not wait for the session to finish answering, and it carries back the output cursor read immediately before the write.

That cursor is exact rather than approximate. The daemon is single-threaded, so no other request can have written to that session in between — the position taken before the write *is* the position this turn's output starts at. The client polls `read` from there and sees its own turn, from the first byte, with nothing of the previous one.

A `send` that waited for the turn to finish would be simpler to call and would freeze every other session for its duration.

### Submitting Is Not the Same as Typing

`send` writes the prompt, pauses 200ms, then writes the carriage return that submits it. The pause is the difference between a prompt that runs and a prompt that sits in the input box.

It was measured, not guessed. Against a real `claude`, with both writes issued back to back, prompts up to about 55 bytes submitted normally and everything longer silently did not — the text appeared in the session's input box and stayed there, with the *next* prompt landing underneath it on a second line. No error, on either side.

The two writes land in the pty's buffer together, so a reader that has not been scheduled in between sees one chunk of text-then-return. A terminal application reading a burst that size treats it as pasted input, and a newline inside a paste is a newline rather than a submission. That is correct of it — pasting a multi-line snippet should not fire off the first line — and exactly wrong for us. Under the threshold the burst was small enough to read as typing, which is why the failure looked like it was about length.

Sending the return as its own event, far enough behind the text that no arrival-rate heuristic can attach the two, is the whole fix. It blocks the daemon for that fifth of a second, deliberately: `send` is already the one request whose caller is waiting on the result.

Measured, with `SUBMIT_GAP` removed and restored, at prompt lengths 26 / 54 / 68 / 79 / 88 / 137 bytes:

| Prompt bytes | 26 | 54 | 68 | 79 | 88 | 137 |
|---|---|---|---|---|---|---|
| Without the gap | answered | answered | **no answer** | **no answer** | **no answer** | **no answer** |
| With the gap | answered | answered | answered | answered | answered | answered |

### Who Owns the Loop

`serve_once` is the body of a daemon's main loop. The loop itself is not here.

An `accept` loop inside a library is a loop the caller cannot end — no shutdown, no signal handling, no way to serve a bounded number of connections. Leaving it out costs the binary four lines and makes the whole path testable: a test serves exactly the connections it intends to and then gets its `Daemon` back to inspect.

```rust,ignore
loop
{
  serve_once( &listener, &mut daemon )?;
  if daemon.stop_requested() { break }
}
daemon.shutdown_all()?;
```

`serve_connection` handles the framing around one exchange, and it is deliberate about what counts as a failure:

| On the wire | Outcome |
|-------------|---------|
| A client that hangs up without sending | `Ok(())` — nothing read, nothing written |
| A line that is not a valid request | `Response::err` sent back |
| A request that fails | `Response::err` sent back |
| The response cannot be written | `Err` — nothing left to tell the client with |

### The Client Side

`client::request` is one connection, one line out, one line back. `client::call` adds the one thing every caller would otherwise write itself: turning `Response::Err` into `Error::Remote`, so a daemon-side failure arrives as an ordinary `Result` rather than as a successful call carrying bad news.

Both set a read and a write timeout — `DEFAULT_TIMEOUT` is 60 seconds. Without one, a client talking to a wedged daemon blocks forever on a socket that will never answer.

### Verification

```bash
cd module/claude_daemon_core && ./verb/test
```

Or the two suites directly, inside the container:

```bash
cargo nextest run -p claude_daemon_core --test listener_test
cargo nextest run -p claude_daemon_core --test serve_test
```

`tests/listener_test.rs` covers binding, cleanup on drop, binding over a stale socket, the foreign-lock refusal, the non-socket refusal, permissions, and accepting a real client.

`tests/serve_test.rs` runs a daemon on a thread with a real socket and real PTY-attached children, and drives it through the client: ping, listing, spawning, the send/read round trip, the cursor `send` reports, unknown sessions, a malformed line, shutdown, resize, a prompt carried by `spawn`, stopping the daemon, a child that never registers, and the submit gap.

Its server thread runs the loop above verbatim, so the ordering `stop_daemon` depends on is exercised rather than assumed.

The submit gap (srv13) is checked with a stopwatch rather than an observation, which is a real limit worth naming. What the gap guarantees is that the input handler reads the text and the return as two separate events — but the daemon is single-threaded, so while `send` is holding them apart it cannot answer a `read` that would catch it in the act. From outside, the pause has exactly one signature: latency. srv13 asserts that latency against a fixed floor rather than against `SUBMIT_GAP` itself, since a constant compared to itself passes at any value including zero. That the text goes out first is settled by the write order; that the pause sits between the two writes rather than after both is settled by reading `send`. The paste heuristic itself exists only in a real `claude`, so the end of that story has to be checked by hand against one, not asserted by this suite.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/listener.rs` | Binding, stale-socket removal, cleanup on drop |
| source | `src/serve.rs` | `Daemon`, `serve_connection`, `serve_once` |
| source | `src/client.rs` | `request`, `request_within`, `call` |
| doc | [001_single_instance.md](001_single_instance.md) | The lock the socket's safety rests on |
| doc | [002_wire_protocol.md](002_wire_protocol.md) | The request and response shapes |
| doc | [004_session_output.md](004_session_output.md) | The cursors `send` and `read` trade in |
| doc | [005_session_registration.md](005_session_registration.md) | Why `spawn` has to wait before it can answer |
| test | `tests/listener_test.rs` | The socket's lifecycle |
| test | `tests/serve_test.rs` | End-to-end dispatch over a real socket |
