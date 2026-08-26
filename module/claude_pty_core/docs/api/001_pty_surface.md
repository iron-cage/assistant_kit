# API: PTY Surface

### Scope

- **Purpose**: Pin the signature and error contract of every item `claude_pty_core` exports, so a consumer can depend on it without reading the source.
- **In Scope**: All items re-exported from `lib.rs`, plus the `env_scrub` module.
- **Out of Scope**: The private `ffi` module.

### Errors

Every fallible function returns `Result< T > = core::result::Result< T, Error >`.

| Variant | Meaning |
|---------|---------|
| `Error::Os { op, source }` | A libc call failed; `op` names which one |
| `Error::NonUtf8SlavePath` | `ptsname_r` returned bytes that are not UTF-8 |
| `Error::WriterFull` | Write queue at capacity — the child has stopped reading stdin |
| `Error::WriterGone` | The writer thread has exited |
| `Error::Spawn( io::Error )` | The child process could not be started |
| `Error::SessionClosed` | The master was closed by `shutdown`; the terminal is gone |

`Error::last_os( op )` constructs an `Os` variant from `errno` at the call site.

### `Pty`

| Signature | Contract |
|-----------|----------|
| `Pty::open() -> Result< Pty >` | Allocates a master/slave pair. Fails `Os` if any of the four allocation calls fails, `NonUtf8SlavePath` if the slave path is not UTF-8. |
| `Pty::master( &self ) -> &OwnedFd` | Borrows the master. The `Pty` owns it; clone with `try_clone()` to hand a descriptor elsewhere. |
| `Pty::slave_path( &self ) -> &str` | The slave's filesystem path, resolved once at `open`. |
| `Pty::open_slave( &self ) -> Result< File >` | Opens a *new* descriptor on the slave. Each call is independent — see [feature/002_session_spawn.md](../feature/002_session_spawn.md). |
| `Pty::resize( &self, size : WinSize ) -> Result< () >` | Issues `TIOCSWINSZ`; the kernel delivers `SIGWINCH` to the child. |

`WinSize::new( rows, cols )` is `const`. `WinSize::default()` is 24×80.

### `SessionConfig`

A consuming builder — every method takes and returns `self`.

| Signature | Contract |
|-----------|----------|
| `SessionConfig::new( program : impl Into< OsString > ) -> Self` | Program to run; window size defaults to 24×80, queue capacity to `DEFAULT_QUEUE_CAPACITY`. |
| `.arg( impl Into< OsString > ) -> Self` | Appends one argument. |
| `.env( key, value ) -> Self` | Sets one variable, applied *after* scrubbing — so it can reinstate a scrubbed name. |
| `.cwd( impl Into< PathBuf > ) -> Self` | Child's working directory. |
| `.win_size( WinSize ) -> Self` | `const`. Applied before the child starts, so it never renders at the default first. |
| `.queue_capacity( usize ) -> Self` | `const`. Depth of the writer queue. |

### `PtySession`

| Signature | Contract |
|-----------|----------|
| `PtySession::spawn( &SessionConfig ) -> Result< PtySession >` | Allocates, sizes, scrubs, and starts the child as session leader with the PTY as controlling terminal. Fails `Os` on allocation, `Spawn` on process start. |
| `.write( &self, bytes : &[ u8 ] ) -> Result< () >` | Queues bytes for the child's stdin. Never blocks. Fails `WriterFull` or `WriterGone`. |
| `.resize( &self, size : WinSize ) -> Result< () >` | Delegates to `Pty::resize`. Fails `SessionClosed` after `shutdown`. |
| `.take_reader( &mut self ) -> Option< File >` | Yields the master read half **once**; a second call returns `None`. |
| `.pid( &self ) -> u32` | The child's process id. |
| `.slave_path( &self ) -> &str` | The slave path, for correlating against an external registry. Still readable after `shutdown`. |
| `.try_wait( &mut self ) -> Result< Option< ExitStatus > >` | Non-blocking exit check. `Ok( None )` means still running. |
| `.shutdown( &mut self ) -> Result< ExitStatus >` | Closes every master descriptor it owns, then blocks for the child's exit. Idempotent. |

**Caller obligation:** the reader returned by `take_reader` must be drained, or the child eventually blocks writing output. The library does not read on the caller's behalf, because who consumes the output is the consumer's decision.

**Caller obligation:** a reader taken with `take_reader` is a master descriptor the session no longer owns. `shutdown` cannot close it, so the child's stdin stays open and `shutdown` blocks until the caller drops it.

### `WriterHandle`

| Signature | Contract |
|-----------|----------|
| `WriterHandle::spawn< W : Write + Send + 'static >( sink : W, capacity : usize ) -> Self` | Starts the drain thread; takes ownership of `sink`. |
| `.send( &self, bytes : &[ u8 ] ) -> Result< () >` | Copies onto the queue. `WriterFull` at capacity, `WriterGone` if the thread has exited. |
| `.shutdown( &mut self )` | Closes the queue and joins the thread. |

Dropping without `shutdown` closes the queue and detaches — the thread drains what is queued and exits, but is not joined.

### `env_scrub`

| Item | Contract |
|------|----------|
| `TERMINAL_IDENTITY_VARS : &[ &str ]` | The eight terminal-emulator variables removed from the child. |
| `CLAUDE_MARKER_PREFIX : &str` | `"CLAUDE_"` — every variable with this prefix is removed. |
| `CHILD_TERM : &str` | `"xterm-256color"`, set as the child's `TERM`. |
| `scrub_list< 'a, I >( source : I ) -> Vec< String >` | Given variable names, returns those that must be removed. |
| `is_scrubbed( name : &str ) -> bool` | Whether one name would be removed. |

### Verification

```bash
cd module/claude_pty_core && cargo doc --no-deps --all-features
```

The workspace sets `missing_docs = "warn"` and this crate additionally sets `#![ deny( missing_docs ) ]`, so an undocumented public item fails the build rather than appearing here as a gap.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/lib.rs` | The re-export list this documents |
| doc | [feature/001_pty_allocation.md](../feature/001_pty_allocation.md) | Behavior behind `Pty` |
| doc | [feature/002_session_spawn.md](../feature/002_session_spawn.md) | Behavior behind `PtySession` |
| doc | [feature/003_writer_thread.md](../feature/003_writer_thread.md) | Behavior behind `WriterHandle` |
| test | `tests/pty_test.rs` | Allocation and resize |
| test | `tests/session_test.rs` | Spawn, write, read, shutdown |
| test | `tests/writer_test.rs` | Queue capacity and shutdown ordering |
