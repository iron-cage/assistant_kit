# Invariant: Session Mismatch Detection

### Scope

- **Purpose**: Document the diagnostic warning emitted when the session resumed by `claude -c` differs from the session that was expected based on pre-spawn storage inspection.
- **Responsibility**: State when the warning fires, what it means, what it does NOT do (never fails the run), and the implementation mechanism.
- **In Scope**: `expected_session_id` capture in `build_claude_command()`, `extract_session_id()` call on success path in `run_print_mode()`, stderr warning format, non-fatal semantics.
- **Out of Scope**: The `-c` injection decision (→ `invariant/001_default_flags.md`), continuation detection algorithm (→ `claude_storage_core/docs/feature/004_continuation_detection.md`), BUG-320 root cause analysis (closed — → `task/claude_runner/bug/320_clr_dir_daemon_env_session_contamination.md`).

### Invariant Statement

When `clr run` or `clr ask` injects `-c` (i.e. `expected_session_id` is `Some`), the runner MUST compare the actual `session_id` field from claude's JSON result envelope with the expected UUID on the success path. If they differ, the runner MUST emit a `[Runner] warning` line to stderr before printing the result. The run MUST NOT fail — the warning is diagnostic only.

| Condition | Behavior |
|-----------|----------|
| `expected_session_id` is `None` (no `-c` injected) | No comparison, no warning |
| `expected_session_id` is `Some(uuid)` AND actual matches | Silent success |
| `expected_session_id` is `Some(uuid)` AND actual differs | `[Runner] warning: session mismatch — expected {uuid}, got {actual} (BUG-320 detected)` on stderr; result still printed; exit 0 |
| `expected_session_id` is `Some(uuid)` AND result has no `session_id` field | Silent — non-JSON output or `--output-format text` mode; comparison skipped |

### Rationale

BUG-320 confirmed root cause (H6 — closed 2026-06-29): a daemon inherited `CLR_DIR=<wrong-project>` from its launching shell (`pr_review/run.sh:78 export CLR_DIR`); `parse.rs:313` substituted `CLR_DIR` as the effective directory override; `session_exists()` found the wrong project's session; `cmd.current_dir()` redirected the subprocess; `claude -c` resumed the wrong project's session. The fix for H6 is in wplan_daemon (BUG-1129 — `execute_job_sync()` must sanitize subprocess env before spawn).

The diagnostic warning (this invariant) remains independently valuable: it detects any session mismatch regardless of cause — env contamination, filesystem race, or unknown future mechanisms — and surfaces it in stderr without failing the run. By the time the mismatch is detected, claude has already responded using the wrong session; aborting post-hoc would discard a completed result.

### Enforcement Mechanism

Three components collaborate to enforce this invariant:

1. **`builder.rs` — capture expected UUID:**
   `session_exists()` returns `Option<SessionId>` (the UUID of the most-recently-modified `.jsonl` file in the session storage directory) instead of `bool`. `build_claude_command()` stores this as `expected_session_id` and returns `(ClaudeCommand, Option<SessionId>)`. The `-c` injection condition is unchanged: `expected_session_id.is_some()`.

2. **`summary.rs` — extract actual UUID:**
   `extract_session_id(stdout: &str) -> Option<String>` parses the `session_id` field from a CLR result envelope. Guards on `"type":"result"` per invariant/008 — returns `None` for non-result types (tool use, thinking blocks).

3. **`execution.rs` — compare and warn:**
   On the success path in `run_print_mode()`, after capturing `raw_stdout` and before fence-stripping:
   ```rust
   if let Some( expected ) = expected_session_id
   {
     if let Some( actual ) = super::summary::extract_session_id( &output.stdout )
     {
       if actual.as_str() != expected.as_str()
       {
         eprintln!( "[Runner] warning: session mismatch — expected {}, got {} (BUG-320 detected)",
           expected.as_str(), actual );
       }
     }
   }
   ```

### Warning Format

```
[Runner] warning: session mismatch — expected <expected-uuid>, got <actual-uuid> (BUG-320 detected)
```

The `(BUG-320 detected)` suffix makes the warning grep-able and links it to the documented bug. The warning is emitted on stderr. It appears before the result output so it is visible even when stdout is captured to `--output-file`.

### Non-Fatal Design Decision

The warning is intentionally non-fatal. Reasons:
- The wrong session has already responded; aborting discards a completed response.
- The mismatch does not indicate data corruption — only a wrong session was loaded.
- The user can retry with `--new-session` if the wrong context is detected.
- Future work may introduce a pre-spawn probe (dry-run `-c` to verify UUID before commit), but this would require a second invocation and is YAGNI until the warning confirms the bug is reproducible.

### Permanent Design Decision — `-c` Always Preferred Over `--resume <uuid>`

`clr` always uses `with_continue_conversation(true)` / `claude -c` for session continuation. Using `with_resume(Some(uuid))` / `claude -r <uuid>` was evaluated as an alternative for BUG-320 and **explicitly rejected. Do not revisit this decision.**

Reasons:
- `-c` is semantically natural and matches `claude --help`: "continue the most recent conversation in the current directory"
- `-c` is resilient: gracefully falls through to a fresh session when UUID files are missing, stale, or deleted; `--resume <uuid>` fails hard if the session file is gone
- BUG-320's confirmed root cause (H6) was daemon env contamination in wplan_daemon — a defect outside claude_runner, not in the `-c` mechanism itself
- This diagnostic warning provides the observability needed without altering the continuation mechanism

`most_recent_session_id()` (added by TSK-334) is used exclusively for UUID capture pre-spawn and comparison post-execution. It does **not** drive `--resume`. See `task/claude_runner/bug/320_clr_dir_daemon_env_session_contamination.md § Fix Location`.

### Fixed Defects

**BUG-320 (diagnostic hardening — closed 2026-06-29) — `clr -c` resumed wrong session via daemon `CLR_DIR` env contamination (H6):**
Daemon inherited `CLR_DIR=<wrong-project>` from launching shell; `parse.rs:313` substituted it as effective dir override; `claude -c` resumed the wrong project's session. Fix is in wplan_daemon (BUG-1129). This warning provides independent observability for any future mismatch regardless of cause. Root cause: `task/claude_runner/bug/320_clr_dir_daemon_env_session_contamination.md`.

### Sources

| File | Notes |
|------|-------|
| `../../src/cli/builder.rs` | `session_exists()` returning `Option<SessionId>`; `build_claude_command()` returning `(ClaudeCommand, Option<SessionId>)` |
| `../../src/cli/summary.rs` | `extract_session_id(stdout) -> Option<String>` |
| `../../src/cli/execution.rs` | Comparison + warning emission on success path in `run_print_mode()` |

### Tests

| File | Notes |
|------|-------|
| `../../tests/session_verification_test.rs` | `sv1`–`sv4`: match/mismatch/no-session/non-result-type coverage |
| `../../tests/summary_unit_test.rs` | `extract_session_id` unit tests (result type, non-result type, absent field) |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_default_flags.md](001_default_flags.md) | `-c` injection decision that gates this check |
| [invariant/008_render_summary_gate.md](008_render_summary_gate.md) | `"type":"result"` gate that `extract_session_id()` inherits |

### Features

| File | Relationship |
|------|--------------|
| `module/claude_storage_core/docs/feature/004_continuation_detection.md` | Upstream API spec: `most_recent_session_id()` / `most_recent_session_in_dir()` that supply `Option<SessionId>` used here |
