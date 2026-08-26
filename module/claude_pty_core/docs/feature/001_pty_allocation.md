# Feature: PTY Allocation

### Scope

- **Purpose**: Allocate a pseudo-terminal pair so a program that refuses to run without a terminal can be hosted programmatically.
- **In Scope**: `Pty::open`, `Pty::master`, `Pty::slave_path`, `Pty::open_slave`, `Pty::resize`, `WinSize`.
- **Out of Scope**: Spawning a child onto the pair (→ [002_session_spawn.md](002_session_spawn.md)), writing to the master (→ [003_writer_thread.md](003_writer_thread.md)).

### Behavior

`Pty::open()` performs the four-call POSIX allocation sequence and hands back an owned master descriptor plus the slave's filesystem path:

1. `posix_openpt( O_RDWR | O_NOCTTY )` — allocate a master. `O_NOCTTY` matters: without it the calling process could acquire the new terminal as its own controlling terminal, which is the opposite of what a supervising process wants.
2. `grantpt( master )` — set the slave's ownership and permissions for the calling user.
3. `unlockpt( master )` — release the slave so it can be opened.
4. `ptsname_r( master, buf, len )` — resolve the slave path into a caller-supplied buffer.

`ptsname_r` is used rather than `ptsname` because `ptsname` returns a pointer into a static buffer shared across the process. Two threads allocating PTYs concurrently would each see whichever path was written last, and neither would know it.

### Window Size

A terminal program reads its window size from the kernel, not from an environment variable. `Pty::resize( WinSize )` issues `TIOCSWINSZ` against the master; the kernel then reports the new size to the child and delivers `SIGWINCH`.

`WinSize::default()` is 24 rows by 80 columns — the historical VT100 default, and what a program sees when nothing sets a size. `SessionConfig::win_size` applies a size at spawn time so the child never renders one frame at the default before being corrected.

### Ownership

`Pty` holds the master as an `OwnedFd`, so the descriptor closes when the `Pty` drops. The slave is *not* held open: `slave_path()` returns the path and `open_slave()` opens a fresh descriptor on demand. This is deliberate — see [002_session_spawn.md](002_session_spawn.md) for why the parent must not retain a slave descriptor after spawning.

The master is opened with `O_CLOEXEC`, and that flag is load-bearing rather than hygienic. Without it the descriptor survives `exec`, so every child spawned afterwards — including the pty's own — inherits a copy of the master to its own terminal. Once that happens, closing every descriptor the parent holds no longer produces `EOF` on the slave: the child is keeping its own terminal alive, so a child blocked reading stdin never exits and `PtySession::shutdown` waits forever. Descriptors made by `try_clone` are already close-on-exec (`F_DUPFD_CLOEXEC`), and the slave descriptors are *meant* to reach the child — they get there through `dup2`, which clears the flag by design. So the flag matters on exactly one descriptor: this one.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/pty.rs` | `Pty` and `WinSize` |
| source | `src/ffi.rs` | The four `extern "C"` declarations and their wrappers |
| doc | [invariant/001_unsafe_containment.md](../invariant/001_unsafe_containment.md) | Why every FFI call lives in one module |
| doc | [api/001_pty_surface.md](../api/001_pty_surface.md) | Full signature contract |
| test | `tests/pty_test.rs` | Real allocation, resize, and slave-path round-trip |
