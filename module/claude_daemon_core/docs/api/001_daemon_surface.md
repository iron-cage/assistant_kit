# API: Daemon Surface

### Scope

- **Purpose**: Pin the signature and error contract of every item `claude_daemon_core` exports.
- **In Scope**: All items re-exported from `lib.rs`, plus the `ipc`, `lock`, `paths`, `protocol`, and `table` modules.
- **Out of Scope**: The `error` module's internals; construct errors through the operations that return them.

### Errors

`Result< T > = core::result::Result< T, Error >`.

| Variant | Meaning |
|---------|---------|
| `Error::Io( io::Error )` | A filesystem or socket operation failed |
| `Error::AlreadyRunning { lock_path }` | Another daemon holds the instance lock |
| `Error::LineTooLong` | A protocol line exceeded `MAX_IPC_LINE_BYTES` |
| `Error::NonUtf8Line` | A protocol line was not valid UTF-8 |
| `Error::Malformed( String )` | A line parsed as JSON but not as a `Request` |
| `Error::UnknownSession( String )` | No hosted session carries that conversation id |
| `Error::Pty( claude_pty_core::Error )` | An operation on the underlying terminal failed |

`From< io::Error >` and `From< claude_pty_core::Error >` are implemented, so `?` works across both boundaries.

### `lock`

| Signature | Contract |
|-----------|----------|
| `acquire( lock_path : &Path ) -> Result< InstanceLock >` | Non-blocking `flock( LOCK_EX \| LOCK_NB )`. `AlreadyRunning` if held, `Io` if the file cannot be opened. |
| `InstanceLock::path( &self ) -> &Path` | The locked file's path. |

**Caller obligation:** the lock's parent directory must exist. `acquire` does not create it — creating a directory as a side effect of taking a lock hides a filesystem write inside an operation that reads as a mutual-exclusion primitive.

The lock is released on drop, including on panic and on `SIGKILL` — see [feature/001_single_instance.md](../feature/001_single_instance.md).

### `paths`

| Signature | Contract |
|-----------|----------|
| `RUNTIME_DIR_NAME : &str` | `"-daemon"` — hyphen-prefixed, so git ignores it |
| `LOCK_FILE_NAME : &str` | `"instance.lock"` |
| `SOCKET_FILE_NAME : &str` | `"daemon.sock"` |
| `DaemonPaths::new() -> Option< Self >` | `None` when neither `CLAUDE_HOME` nor `HOME` is set |
| `DaemonPaths::with_home( home : &Path ) -> Self` | Explicit base — the form tests use |
| `.runtime_dir( &self ) -> &Path` | `<home>/-daemon/` |
| `.lock_file( &self ) -> PathBuf` | `<home>/-daemon/instance.lock` |
| `.socket_file( &self ) -> PathBuf` | `<home>/-daemon/daemon.sock` |
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
| `Request` | `#[ non_exhaustive ]`, internally tagged on `method` in `snake_case`. Variants: `Ping`, `ListSessions`, `Spawn { cwd, prompt }`, `Send { session_id, text }`, `Resize { session_id, rows, cols }`, `Shutdown { session_id }`. |
| `Response::ok( result : serde_json::Value ) -> Self` | `const`. Serializes as `{ "ok" : true, "result" : … }`. |
| `Response::err( error : impl Into< String > ) -> Self` | Serializes as `{ "ok" : false, "error" : … }`. |
| `SessionSummary` | `#[ non_exhaustive ]`. Fields `session_id`, `pid`, `cwd`, `busy`. |

`Response` is `#[ serde( untagged ) ]` with marker types carrying hand-written impls, so the `ok` discriminant is a real field rather than an enum tag — preserving the shape existing clients already parse.

### `table`

| Signature | Contract |
|-----------|----------|
| `SessionTable::new() -> Self` | Empty. Also `Default`. |
| `.len() / .is_empty()` | Hosted-session count |
| `.insert( &mut self, session : HostedSession )` | Replaces any entry with the same conversation id |
| `.get_mut( &mut self, session_id : &str ) -> Result< &mut HostedSession >` | `UnknownSession` when absent |
| `.remove( &mut self, session_id : &str ) -> Result< HostedSession >` | `UnknownSession` when absent |
| `.summaries( &self ) -> Vec< SessionSummary >` | Sorted by conversation id, so repeated calls are stable |
| `HostedSession::summary( &self ) -> SessionSummary` | Snapshot of one session |

`HostedSession` fields are public: `session_id`, `cwd`, `pty`, `busy`. `busy` is the daemon's belief, maintained from `claude_session_core`'s `TurnWatcher` — not read from the registry per request.

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
| test | `tests/lock_test.rs` | Exclusion and release |
| test | `tests/ipc_test.rs` | Framing and the cap |
| test | `tests/protocol_test.rs` | Serde round-trips |
| test | `tests/table_test.rs` | Table operations |
