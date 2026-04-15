# Feature: Account Rotation Daemon

### Scope

- **Purpose**: Automate credential rotation in the background so the user never has to manually rotate when their 5-hour rate-limit window fills up.
- **Responsibility**: Documents the `.credentials.rotation.start`, `.credentials.rotation.stop`, and `.credentials.rotation.status` commands and the `--bg-rotation-daemon` background process (FR-19).
- **In Scope**: Daemon lifecycle (spawn, stop, status), PID file management, rate-limit polling, rotation trigger threshold, log format.
- **Out of Scope**: Account selection strategy (→ 008_auto_rotate.md), rate-limit data source (→ 013_account_limits.md), credential switching (→ 004_account_switch.md).

### Design

The rotation daemon is a detached background process that polls rate-limit utilization every 5 minutes and calls `account::auto_rotate()` when the 5-hour window is ≥ 90% consumed.

**Daemon lifecycle:**

1. `.credentials.rotation.start` — `rotation_background_spawn()` checks for a live PID file, removes stale ones, then re-invokes the current binary with `--bg-rotation-daemon`. The flag is intercepted in `run_cli()` before unilang dispatch and routes to `rotation_run()`.
2. `rotation_run()` creates `~/.claude/.transient/` if absent, opens the log file (append), writes the PID file atomically via `O_CREAT | O_EXCL`, installs a `PidGuard` RAII drop handler, then enters the infinite poll loop.
3. `.credentials.rotation.stop` — sends `SIGTERM` to the recorded PID, then polls `process_is_alive()` up to 10 × 200 ms (2 s total). Removes the PID file only after the process exits; returns an error and leaves the file in place if the timeout is exceeded.
4. `.credentials.rotation.status` — reads the PID file, checks liveness, and scans the log for the last `[unix_secs] Auto-rotation triggered` line to compute how long ago the last rotation occurred.

**Files on disk:**

| Path | Purpose |
|------|---------|
| `~/.claude/.transient/.rotation.pid` | PID of the running daemon; removed on clean exit or after confirmed stop |
| `~/.claude/.transient/rotation.log` | Append-only log; rotation events prefixed `[unix_secs]` for machine-parseable timestamps |

**Atomic PID file creation:** `pid_file_create()` uses `OpenOptions::new().write(true).create_new(true)` (`O_CREAT | O_EXCL`) so existence check and creation are a single syscall with no TOCTOU race.

**Stale PID file handling:** Both `rotation_background_spawn()` and `rotation_stop()` use `process_is_alive()` (`kill -0`) to distinguish a live daemon from a crashed one. A stale file is removed and a warning is printed before proceeding.

**Poll interval:** 5 minutes (`Duration::from_secs(5 * 60)`). The threshold for triggering rotation is `utilization_5h >= 0.9`.

**Internal flag:** `--bg-rotation-daemon` is not registered as a unilang command and does not appear in `.help` output. It is only ever passed by `rotation_background_spawn()` when re-invoking the current executable.

**Exit codes:**
- 0: success (start: daemon spawned; stop: daemon terminated; status: always 0)
- 2: runtime error (PID file conflict, spawn failure, SIGTERM failure, termination timeout)

### Acceptance Criteria

- **AC-01**: `.credentials.rotation.start` spawns a background process and creates `~/.claude/.transient/.rotation.pid`.
- **AC-02**: `.credentials.rotation.start` returns an error if the daemon is already running; does not spawn a second instance.
- **AC-03**: `.credentials.rotation.start` removes a stale PID file (process gone) and re-spawns without error.
- **AC-04**: `.credentials.rotation.stop` sends SIGTERM, waits for the process to exit, removes the PID file, and exits 0.
- **AC-05**: `.credentials.rotation.stop` returns an error and leaves the PID file in place if the process does not exit within 2 s.
- **AC-06**: `.credentials.rotation.status` reports `running (pid N)` when the daemon is alive, `stopped` otherwise.
- **AC-07**: `.credentials.rotation.status` reports the time since last rotation (e.g. `3h 12m ago`) or `never` when no rotation has occurred.
- **AC-08**: After a rotation event, `rotation.log` contains a `[unix_secs] Auto-rotation triggered` line with a valid timestamp.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/rotation.rs` | Daemon loop, PID file management, spawn/stop/status helpers |
| source | `src/commands.rs` | `credentials_enable_auto_rotation_routine`, `credentials_disable_auto_rotation_routine`, `credentials_rotation_status_routine` |
| source | `src/lib.rs` | `--bg-rotation-daemon` flag interception in `run_cli()`; command registration |
| doc | [008_auto_rotate.md](008_auto_rotate.md) | `auto_rotate()` — account selection called by the daemon on each rotation |
| doc | [013_account_limits.md](013_account_limits.md) | Rate-limit utilization data source polled by the daemon loop |
| doc | [004_account_switch.md](004_account_switch.md) | Credential switching primitive invoked via `auto_rotate()` |
