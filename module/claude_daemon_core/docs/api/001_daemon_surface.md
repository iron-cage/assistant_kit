# API: Daemon Surface

### Scope

- **Purpose**: Pin the signature and error contract of every item `claude_daemon_core` exports.
- **In Scope**: All items re-exported from `lib.rs` — including `BackgroundReporting` and `BG_TASKS_REPORT_RUNNING_ENV`, re-exported from `claude_session_core` because `Daemon::with_background_reporting` takes one — plus the `baseline`, `client`, `context`, `ipc`, `listener`, `lock`, `output`, `paths`, `protocol`, `registration`, `serve`, and `table` modules.
- **Out of Scope**: The `error` module's internals; construct errors through the operations that return them.

### Errors

`Result< T > = core::result::Result< T, Error >`.

| Variant | Meaning |
|---------|---------|
| `Error::Io( io::Error )` | A filesystem or socket operation failed |
| `Error::AlreadyRunning { lock_path }` | Another daemon holds the instance lock |
| `Error::LockMismatch { lock_path, socket_path }` | The lock offered as evidence does not cover the socket being bound |
| `Error::LineTooLong` | A protocol line exceeded `MAX_IPC_LINE_BYTES` |
| `Error::NonUtf8Line` | A protocol line was not valid UTF-8 |
| `Error::Malformed( String )` | A line parsed as JSON but not as a `Request` |
| `Error::UnknownSession( String )` | No hosted session carries that conversation id |
| `Error::ReaderTaken` | A session's reader was already taken, so its output cannot be drained |
| `Error::NoRegistration { pid }` | A spawned process never published a conversation id |
| `Error::Remote( String )` | The daemon answered, and its answer was a failure |
| `Error::Pty( claude_pty_core::Error )` | An operation on the underlying terminal failed |
| `Error::Registry( claude_session_core::Error )` | Claude Code's session registry could not be read |
| `Error::Storage( claude_storage_core::Error )` | Claude Code's on-disk conversation storage could not be read |
| `Error::NoTranscript { session_id }` | The session has no readable transcript — its `cwd` will not encode, or it has not written one yet |
| `Error::Probe { reason }` | A baseline probe ran and then failed, or answered something unreadable. Distinct from `Io`, which means `claude` could not be run at all — a probe that half-worked must never be recorded as a floor of zero |

`From< io::Error >`, `From< claude_pty_core::Error >`, `From< claude_session_core::Error >`, and `From< claude_storage_core::Error >` are implemented, so `?` works across all four boundaries.

### `lock`

| Signature | Contract |
|-----------|----------|
| `acquire( lock_path : &Path ) -> Result< InstanceLock >` | Non-blocking `flock( LOCK_EX \| LOCK_NB )`. `AlreadyRunning` if held, `Io` if the file cannot be opened. |
| `InstanceLock::path( &self ) -> &Path` | The locked file's path. |

`acquire` creates the lock path's parent directory (`create_dir_all`) if it does not already exist, then opens or creates the lock file itself — the caller does not need to create the directory first.

The lock is released on drop, including on panic and on `SIGKILL` — see [feature/001_single_instance.md](../feature/001_single_instance.md).

### `paths`

| Signature | Contract |
|-----------|----------|
| `RUNTIME_DIR_NAME : &str` | `"-daemon"` — hyphen-prefixed, so git ignores it |
| `LOCK_FILE_NAME : &str` | `"instance.lock"` |
| `SOCKET_FILE_NAME : &str` | `"daemon.sock"` |
| `LOG_FILE_NAME : &str` | `"daemon.log"` |
| `DaemonPaths::new() -> Option< Self >` | `None` when `HOME` is not set |
| `DaemonPaths::with_home( home : &Path ) -> Self` | Explicit base — the form tests use |
| `.runtime_dir( &self ) -> &Path` | `<home>/-daemon/` |
| `.lock_file( &self ) -> PathBuf` | `<home>/-daemon/instance.lock` |
| `.socket_file( &self ) -> PathBuf` | `<home>/-daemon/daemon.sock` |
| `.log_file( &self ) -> PathBuf` | `<home>/-daemon/daemon.log` — where a detached daemon's output is appended |
| `.sessions_dir( &self ) -> &Path` | Claude Code's registry directory, for `claude_session_core::scan` |

### `ipc`

| Signature | Contract |
|-----------|----------|
| `MAX_IPC_LINE_BYTES : usize` | `1024 * 1024` |
| `read_capped_line< R : BufRead >( reader : &mut R ) -> Result< Option< String > >` | `Ok( None )` at clean EOF with nothing buffered; `Ok( Some( line ) )` otherwise, trailing `\r` trimmed and the newline consumed but not returned. `LineTooLong` before allocating past the cap; `NonUtf8Line` for invalid UTF-8. |

A partial line at EOF is returned rather than discarded — a peer that closes without a final newline still gets its last request served.

### `protocol`

| Signature | Contract |
|-----------|----------|
| `Request` | `#[ non_exhaustive ]`, internally tagged on `method` in `snake_case`. Variants: `Ping`, `ListSessions`, `Spawn { cwd, prompt }`, `Send { session_id, text }`, `Read { session_id, cursor }`, `ContextSummary { session_id }`, `Resize { session_id, rows, cols }`, `Shutdown { session_id }`, `StopDaemon`. `prompt` and `cursor` are `#[ serde( default ) ]`. |
| `Response::ok( result : serde_json::Value ) -> Self` | `const`. Serializes as `{ "ok" : true, "result" : … }`. |
| `Response::err( error : impl Into< String > ) -> Self` | Serializes as `{ "ok" : false, "error" : … }`. |
| `SessionSummary` | `#[ non_exhaustive ]`. Fields `session_id`, `pid`, `cwd`, `busy`. |

`Response` is `#[ serde( untagged ) ]` with marker types carrying hand-written impls, so the `ok` discriminant is a real field rather than an enum tag — preserving the shape existing clients already parse.

### `output`

| Signature | Contract |
|-----------|----------|
| `DEFAULT_OUTPUT_CAP : usize` | `256 * 1024` — retained bytes per session |
| `OutputBuffer::with_capacity( cap : usize ) -> Self` | Retains at most `cap` bytes, evicting from the front |
| `.push( &mut self, bytes : &[ u8 ] )` | Appends, evicting as needed |
| `.read_from( &mut self, cursor : u64 ) -> OutputSlice` | Non-destructive. A cursor past the end is clamped to it |
| `.end() / .dropped() / .capacity()` | Newest absolute position, evicted count, retention cap |
| `.mark_ended( &mut self ) / .has_ended( &self )` | Whether the stream has ended |
| `OutputPump::spawn( reader : File, cap : usize ) -> Self` | Starts a thread draining `reader` into a fresh buffer |
| `.read_from( &self, cursor : u64 ) -> OutputSlice` | As above, through the pump's shared buffer |
| `.end( &self ) / .has_ended( &self )` | As above |
| `.join( &mut self )` | Waits for the pump thread to finish |
| `OutputSlice` | Fields `text`, `cursor`, `missed`, `ended`. `Serialize`/`Deserialize` |

**Caller obligation:** `OutputPump` holds a clone of the PTY master, which keeps the session alive. Join it only after the child has exited — see [feature/004_session_output.md](../feature/004_session_output.md).

A poisoned buffer mutex is recovered from rather than propagated: a panic in one reader must not make a live session permanently unreadable.

### `context`

| Signature | Contract |
|-----------|----------|
| `summary( cwd : &Path, session_id : &str, baselines : Option< &Path > ) -> Result< serde_json::Value >` | Folds the session's transcript into a context summary. `NoTranscript` when `cwd` will not encode or no transcript exists yet; `Storage` when one exists but cannot be read. `baselines` names a directory a `baseline` measurement may be cached in; `None`, or no measurement matching this session's version and model, reports the overhead split as `null` |

Answers `Request::ContextSummary`. The daemon holds none of this state — it comes from the session's own transcript, read through `claude_storage_core`'s `ContextFold` (current context: deferred tools, agent and skill rosters, remaining budget, tasks) and `Session::stats` (token usage).

**Read-only.** The call never writes to, truncates, or re-creates the transcript, so it is safe to issue against a session with a turn in flight.

#### `tokens`

Different questions, different figures. They are not interchangeable, and the sums in particular are not a measure of fullness.

| Field | Source | Answers |
|-------|--------|---------|
| `remaining` | Harness reminder, parsed | What the harness says is left. Reported, never computed. `null` if no reminder has appeared |
| `context` | `Session::stats` — newest call | How full the conversation is **now**: one call's `input + cache_read + cache_creation` |
| `peak_context` | `Session::stats` — high-water mark | The largest that figure has ever been. Never falls, so after a compaction it describes a conversation that no longer exists |
| `window` | `remaining + context` | The model's usable window. `null` until a turn has both reported a budget and been billed for a prompt |
| `static_overhead` | Cached `baseline` measurement | How much of `context` was spent before the first word. `null` until a baseline has been measured for this session's version and model |
| `conversation` | `context - static_overhead` | The rest of `context` — what the conversation itself occupies. `null` on the same condition |
| `input` / `output` / `cache_read` / `cache_creation` | `Session::stats` — sums, deduplicated by `message.id` | What the session has **cost**. Every turn re-sends the whole conversation, so in a long session these run to many times the window |

The static system prompt never appears in the transcript as text, but its cost is inside `context` — that figure is what the API billed for the whole prompt, tools and system prompt included. This is why `window` is derivable without measuring anything.

`window` is the only figure here that is not in the transcript in any form: the window belongs to the model and the deployment, not to the conversation. It is reported only when both halves that bracket it are present — a budget alone would put the window at the budget, which is true only of a conversation that costs nothing to send.

The split is the one thing a transcript genuinely cannot supply. The overhead is charged identically on every call and so never varies within a session; separating it takes a conversation with no conversation in it, which is what `baseline` measures. Both halves of the cache key come from the session itself — `version` from its newest line, `model` from its first assistant turn — so a measurement taken against anything else is never applied.

### `baseline`

| Signature | Contract |
|-----------|----------|
| `StaticBaseline` | `version`, `model`, `prompt_tokens`, and the three fields it sums. `Serialize`/`Deserialize` |
| `.conversation_tokens( &self, context : u64 ) -> u64` | Saturating. A `context` below the floor means the floor moved, not a negative conversation |
| `probe_args( model : Option< &str > ) -> Vec< String >` | `--print --output-format json --no-session-persistence [--model M] "hi"` |
| `parse_probe( version : &str, stdout : &str ) -> Result< StaticBaseline >` | `Probe` when the output is not JSON, carries no `usage`, or names no model |
| `measure( claude : &Path, version : &str, model : Option< &str > ) -> Result< StaticBaseline >` | **Spends a real API call.** `Io` when `claude` cannot be run; `Probe` when it runs and fails, carrying its stderr |
| `cache_path( runtime_dir : &Path ) -> PathBuf` | `runtime_dir/baseline.json` |
| `load( runtime_dir, version, model ) -> Option< StaticBaseline >` / `load_all( runtime_dir ) -> BTreeMap<..>` | A cache that cannot be read or parsed yields nothing, never an error — it is a memo, and a corrupt one should cost a re-measurement |
| `store( runtime_dir : &Path, baseline : &StaticBaseline ) -> Result< () >` | Replaces any earlier measurement of the same version and model; creates `runtime_dir` |

`prompt_tokens` sums `input + cache_read + cache_creation` rather than reading `input_tokens` alone. The static prompt is identical on every call and so is precisely what prompt caching captures — `input_tokens` alone would put the floor at a few hundred tokens and swing by an order of magnitude between a cold and a warm cache.

**There is no `measure` request in the wire protocol, deliberately.** The daemon is single-threaded and serves one request at a time, so probing inside it would freeze every hosted session for a full API round trip — the same reason `send` does not wait for a turn. A probe also needs nothing the daemon holds, so whoever knows where `claude` is calls `measure` and `store` directly. The daemon's half is `load`, on a request already reading files, wired by `Daemon::with_baselines`.

See `claude_storage_core`'s [data_structure/004_session_context_state.md](../../../claude_storage_core/docs/data_structure/004_session_context_state.md).

`counters.has_unmodelled` reports whether the transcript held a line or attachment kind this build cannot model — when true, the rosters above may under-report, and a client cannot otherwise distinguish an empty roster from an unparsed one.

The JSON is projected field by field here rather than derived from `Serialize`: `claude_storage_core` has zero runtime dependencies and so does not serialize its own types. Keeping the projection on this side is what preserves that guarantee, and it leaves the wire shape owned by the protocol rather than by a struct layout.

### `registration`

| Signature | Contract |
|-----------|----------|
| `REGISTRATION_TIMEOUT : Duration` | 30 seconds |
| `await_session_id( sessions_dir : &Path, pid : u32, timeout : Duration, alive : impl FnMut() -> bool ) -> Result< String >` | Polls until a record names `pid`. `NoRegistration` when `alive()` reports the child gone or `timeout` elapses; `Registry` if the directory exists but cannot be read |
| `lookup( sessions_dir : &Path, pid : u32 ) -> Result< Option< String > >` | One scan, no waiting. A missing directory yields `Ok( None )` |

**Caller obligation:** `alive` must report the *spawned child's* liveness, from the handle the caller holds. The registry cannot distinguish "not yet" from "never".

### `table`

| Signature | Contract |
|-----------|----------|
| `SessionTable::new() -> Self` | Empty. Also `Default`. |
| `.len() / .is_empty()` | Hosted-session count |
| `.insert( &mut self, session : HostedSession ) -> Option< HostedSession >` | `#[ must_use ]`. Returns any entry it replaced — dropping one leaks a live child and its pump thread |
| `.get( &self, session_id : &str ) -> Result< &HostedSession >` | `UnknownSession` when absent |
| `.get_mut( &mut self, session_id : &str ) -> Result< &mut HostedSession >` | `UnknownSession` when absent |
| `.remove( &mut self, session_id : &str ) -> Result< HostedSession >` | `UnknownSession` when absent |
| `.summaries( &self ) -> Vec< SessionSummary >` | Sorted by conversation id, so repeated calls are stable |
| `.session_ids( &self ) -> Vec< String >` | Sorted |
| `HostedSession::adopt( session_id, cwd, pty : PtySession ) -> Result< Self >` | Takes the PTY's reader and starts a pump on it. `ReaderTaken` if it is already gone |
| `.session_id() / .cwd() / .pid() / .busy()` | Accessors; `pid` is diagnostic only |
| `.set_busy( &mut self, busy : bool )` | Record whether a turn is in flight |
| `.write( &self, bytes : &[ u8 ] ) -> Result< () >` | Deliver bytes to the terminal. `Pty` on failure |
| `.read_from( &self, cursor : u64 ) -> OutputSlice` | Output since `cursor` |
| `.output_end( &self ) -> u64` | Absolute position just past the newest byte |
| `.resize( &self, rows : u16, cols : u16 ) -> Result< () >` | `Pty` if the session is closed |
| `.summary( &self ) -> SessionSummary` | Snapshot of one session |
| `.shutdown( &mut self ) -> Result< ExitStatus >` | `Ctrl-D`, then `SIGKILL` after 5s, then join the pump and close the PTY. Idempotent |

`HostedSession`'s fields are private. The pump and the PTY have an invariant between them that public fields would let a caller break silently — see [feature/003_session_table.md](../feature/003_session_table.md).

`busy` is the daemon's belief, maintained from `claude_session_core`'s `TurnWatcher` — not read from the registry per request.

### `listener`

| Signature | Contract |
|-----------|----------|
| `Listener::bind( socket_path : &Path, lock : &InstanceLock ) -> Result< Self >` | `LockMismatch` unless `lock` lives beside `socket_path`. A socket already there is unlinked; anything else is `Io`. Mode set to `0600` after binding |
| `.path( &self ) -> &Path` | The bound path |
| `.accept( &self ) -> Result< UnixStream >` | Blocks for the next client. `Io` on failure |

**Caller obligation:** the parent directory must exist — `bind` creates nothing. Taking the lock creates it, so in practice it already does.

The socket is removed on drop. `UnixListener` does not do this, and the file outliving the daemon turns "not running" into `ECONNREFUSED`.

### `serve`

| Signature | Contract |
|-----------|----------|
| `Daemon::new( sessions_dir : impl Into< PathBuf >, spawner : S ) -> Self` | `S : FnMut( &Path ) -> Result< PtySession >` — the library does not decide what a session runs |
| `.with_registration_timeout( self, timeout : Duration ) -> Self` | `#[ must_use ]`. Overrides `REGISTRATION_TIMEOUT` |
| `.with_background_reporting( self, reporting : BackgroundReporting ) -> Self` | `const`, `#[ must_use ]`. Declares whether `spawner` sets `BG_TASKS_REPORT_RUNNING_ENV`. Defaults to `Unknown`; claiming `Enabled` falsely makes `busy` go false with background work outstanding |
| `.with_baselines( self, dir : impl Into< PathBuf > ) -> Self` | `#[ must_use ]`. Reads cached `baseline` measurements from `dir` when answering `context_summary`. Read-only — the daemon never measures. Absent, the overhead split is reported as `null` |
| `.sessions( &self ) -> &SessionTable` | `const`. What is currently hosted |
| `.stop_requested( &self ) -> bool` | `const`. Set by `StopDaemon`; checked by the main loop after the answer is written |
| `.dispatch( &mut self, request : Request ) -> Response` | Infallible: every failure becomes `Response::err` |
| `.shutdown_all( &mut self ) -> Result< () >` | Attempts every session, returns the first error |
| `serve_connection< H : FnOnce( Request ) -> Response >( stream : &UnixStream, handle : H ) -> Result< () >` | One line in, one line out. A client that hangs up first is `Ok`; an unparseable line gets an error response. `Io` only if the answer cannot be written |
| `serve_once( listener : &Listener, daemon : &mut Daemon< S > ) -> Result< () >` | Accept one client and serve its request |

`send` returns `{ "cursor" : … }` taken immediately before the write, and does not wait for the turn. The daemon is single-threaded, so that cursor is exactly where the turn's output begins.

`list_sessions` refreshes every hosted session's `busy` flag from the registry before answering, through one `TurnWatcher` per session. Not on a timer — the daemon is single-threaded and blocked in `accept` between requests, and this is the only request that reports the flag. A scan that fails leaves the last known state in place rather than failing the request. See [feature/008_turn_state.md](../feature/008_turn_state.md).

**Re-exported for this method's sake:** `BackgroundReporting` and `BG_TASKS_REPORT_RUNNING_ENV`, from `claude_session_core`. An argument type a caller cannot name without adding a dependency is not really public.

The accept *loop* is not here — a loop inside a library is one the caller cannot end.

### `client`

| Signature | Contract |
|-----------|----------|
| `DEFAULT_TIMEOUT : Duration` | 60 seconds |
| `request( socket_path : &Path, request : &Request ) -> Result< Response >` | One connection, one exchange, at `DEFAULT_TIMEOUT` |
| `request_within( socket_path : &Path, request : &Request, timeout : Duration ) -> Result< Response >` | As above, with an explicit read and write timeout |
| `call( socket_path : &Path, request : &Request ) -> Result< serde_json::Value >` | `request`, with `Response::Err` turned into `Error::Remote` |

### Not On This Surface — Rendering

`to_plain_text` is **not** exported here, and this crate does not depend on the
crate that exports it. Turning a session's raw terminal bytes into readable text
needs neither a daemon nor a pty — it is a function of a `&str` — so it lives at
Layer `*` in
[`claude_terminal_core`](../../../claude_terminal_core/docs/api/001_terminal_surface.md),
which a client depends on directly.

What this crate hands back is the raw stream, unrendered, by design: `read`
returns the bytes as they arrived, and rendering is the client's own step. That
is what makes `clr chat --raw` a flag rather than a second protocol request.

### Verification

```bash
cd module/claude_daemon_core && cargo doc --no-deps --all-features
```

`#![ deny( missing_docs ) ]` is set, so an undocumented public item fails the build.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/lib.rs` | The re-export list this documents |
| doc | [feature/001_single_instance.md](../feature/001_single_instance.md) | Behavior behind `acquire` |
| doc | [feature/002_wire_protocol.md](../feature/002_wire_protocol.md) | Behavior behind `Request`/`Response` |
| doc | [feature/003_session_table.md](../feature/003_session_table.md) | Behavior behind `SessionTable` |
| doc | [feature/004_session_output.md](../feature/004_session_output.md) | Behavior behind `OutputPump` |
| doc | [feature/005_session_registration.md](../feature/005_session_registration.md) | Behavior behind `await_session_id` |
| doc | [feature/006_serving_clients.md](../feature/006_serving_clients.md) | Behavior behind `Listener`, `Daemon`, and `client` |
| doc | [`claude_terminal_core` api/001](../../../claude_terminal_core/docs/api/001_terminal_surface.md) | `to_plain_text`, which this surface deliberately does not carry |
| doc | [feature/008_turn_state.md](../feature/008_turn_state.md) | Behavior behind `with_background_reporting` and `SessionSummary::busy` |
| test | `tests/lock_test.rs` | Exclusion and release |
| test | `tests/ipc_test.rs` | Framing and the cap |
| test | `tests/protocol_test.rs` | Serde round-trips |
| test | `tests/output_test.rs` | Cursors, eviction, character boundaries |
| test | `tests/registration_test.rs` | Waiting for a conversation id |
| test | `tests/table_test.rs` | Table operations and teardown |
| test | `tests/listener_test.rs` | The socket's lifecycle |
| test | `tests/serve_test.rs` | End-to-end dispatch over a real socket |
