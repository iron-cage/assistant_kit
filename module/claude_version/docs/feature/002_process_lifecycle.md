# Feature: Process Lifecycle

### Scope

- **Purpose**: Document the process listing and kill commands for running Claude Code sessions.
- **Responsibility**: Describe process detection via `/proc`, rich table output, SIGTERM/SIGKILL signal sequence, targeted kill via pid::, force kill mode, and post-kill verification.
- **In Scope**: `.ps`, `.ps.kill`, `/proc` scanning, rich table output, signal sequence, pid:: targeted kill, force::1 behavior, post-kill verification.
- **Out of Scope**: Version management (→ `feature/001_version_management.md`), hot-swap during install (→ `feature/001_version_management.md`).

### Design

**Process detection:** `.ps` scans `/proc/{pid}/cmdline` for entries where `basename == "claude"` (exact match, not substring). The scanner's own PID is excluded. Unreadable `/proc` entries are skipped non-fatally. Linux-only: uses the `/proc` filesystem.

**Process table output (`.ps`, v::1):** Renders a rich table with columns: #, PID, Elapsed, CPU%, RAM, State, Mode, Path, Task. Path is shortened to `~/...` using `$PRO` or `$HOME` prefix. Task column shows a preview extracted from the active JSONL conversation file, or `—` if unavailable. Processes are sorted oldest-first. Rendering is shared with `claude_runner` via a module in `claude_runner_core`.

**Kill sequence — bulk mode (no `pid::`) — normal:**
1. Send SIGTERM to all detected claude processes
2. Sleep 2 seconds
3. Send SIGKILL to any survivors
4. Sleep 500ms
5. Verify: if any processes still survive, return exit code 2

**Kill sequence — targeted mode (`pid::PID`) — normal:**
1. Validate that PID is a running Claude Code process; exit 1 if not
2. Send SIGTERM to the target PID
3. Sleep 2 seconds
4. Send SIGKILL to the target PID if still alive
5. Sleep 500ms
6. Verify: if the target PID still survives, return exit code 2

**Kill sequence — force mode (`force::1`):**
1. Send SIGKILL directly (skip SIGTERM, skip 2s wait); applies to bulk and targeted modes
2. Sleep 500ms
3. Verify: if any target processes still survive, return exit code 2

Signal delivery uses `Command::new("kill")` (no `libc`, enforced by `unsafe-code = "deny"` workspace lint).

**Kill isolation invariant:** Kill signals are delivered only when a user explicitly invokes `.ps.kill`. The `.version.guard` and `.version.install` flows interact with running processes exclusively via `hot_swap_binary()` (moving the binary path aside), which allows running sessions to continue from their open file descriptor. No automatic path — guard, install, daemon, or interval-watch mode — ever reaches `send_kill_signals()` or any `libc::kill` call.

**Post-kill verification:** After the kill sequence completes, the process list is re-scanned. Any surviving processes cause exit code 2. This verification applies to both normal and force kill modes.

**Dry-run:** `dry::1` prints `[dry-run] would kill N process(es)` without sending any signals.

### Features

| File | Relationship |
|------|-------------|
| [feature/004_dry_run.md](004_dry_run.md) | dry::1 preview mode for .ps.kill |
| [feature/005_cli_design.md](005_cli_design.md) | CLI routing and exit code mapping |

### Sources

| File | Relationship |
|------|-------------|
| `../../src/commands/process.rs` | `.ps` and `.ps.kill` command routines |

### Provenance

| Source | Notes |
|--------|-------|
| `spec.md` (deleted) | FR-08, FR-09, Command Inventory (commands 7-8), Known Limitations |

### Tests

| File | Relationship |
|------|-------------|
| [tests/docs/feature/002_process_lifecycle.md](../../tests/docs/feature/002_process_lifecycle.md) | Feature test spec |
| [tests/docs/cli/command/07_ps.md](../../tests/docs/cli/command/07_ps.md) | `.ps` command integration tests |
| [tests/docs/cli/command/08_ps_kill.md](../../tests/docs/cli/command/08_ps_kill.md) | `.ps.kill` command integration tests |
