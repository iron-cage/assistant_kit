# Feature: Process Lifecycle

### Scope

- **Purpose**: Document the process listing and kill commands for running Claude Code sessions.
- **Responsibility**: Describe process detection via `/proc`, SIGTERM/SIGKILL signal sequence, force kill mode, and post-kill verification.
- **In Scope**: `.processes`, `.processes.kill`, `/proc` scanning, signal sequence, force::1 behavior, post-kill verification.
- **Out of Scope**: Version management (→ `feature/001_version_management.md`), hot-swap during install (→ `feature/001_version_management.md`).

### Design

**Process detection:** `.processes` scans `/proc/{pid}/cmdline` for entries where `basename == "claude"` (exact match, not substring). The scanner's own PID is excluded. Unreadable `/proc` entries are skipped non-fatally. Linux-only: uses the `/proc` filesystem.

**Kill sequence — normal mode:**
1. Send SIGTERM to all detected claude processes
2. Sleep 2 seconds
3. Send SIGKILL to any survivors
4. Sleep 500ms
5. Verify: if any processes still survive, return exit code 2

**Kill sequence — force mode (`force::1`):**
1. Send SIGKILL directly (skip SIGTERM, skip 2s wait)
2. Sleep 500ms
3. Verify: if any processes still survive, return exit code 2

Signal delivery uses `Command::new("kill")` (no `libc`, enforced by `unsafe-code = "deny"` workspace lint).

**Post-kill verification:** After the kill sequence completes, the process list is re-scanned. Any surviving processes cause exit code 2. This verification applies to both normal and force kill modes.

**Dry-run:** `dry::1` prints `[dry-run] would kill N process(es)` without sending any signals.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [feature/004_dry_run.md](004_dry_run.md) | dry::1 preview mode for .processes.kill |
| doc | [feature/005_cli_design.md](005_cli_design.md) | CLI routing and exit code mapping |
| source | `../../src/commands.rs` | Process command routines |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | FR-08, FR-09, Command Inventory (commands 7-8), Known Limitations (Linux-only, kill via Command::new) |
