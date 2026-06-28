# Invariant: Session Mismatch Detection

### Scope

- **Purpose**: Document the diagnostic warning emitted when the session resumed by `claude -c` differs from the session that was expected based on pre-spawn storage inspection.
- **Responsibility**: State when the warning fires, what it means, what it does NOT do (never fails the run), and the implementation mechanism.
- **In Scope**: `expected_session_id` capture in `build_claude_command()`, `extract_session_id()` call on success path in `run_print_mode()`, stderr warning format, non-fatal semantics.
- **Out of Scope**: The `-c` injection decision (→ `invariant/001_default_flags.md`), continuation detection algorithm (→ `claude_storage_core/docs/feature/004_continuation_detection.md`), BUG-320 root cause analysis (→ `task/claude_runner/bug/unverified/320_continue_flag_selects_global_not_cwd.md`).

### Invariant Statement

When `clr run` or `clr ask` injects `-c` (i.e. `expected_session_id` is `Some`), the runner MUST compare the actual `session_id` field from claude's JSON result envelope with the expected UUID on the success path. If they differ, the runner MUST emit a `[Runner] warning` line to stderr before printing the result. The run MUST NOT fail — the warning is diagnostic only.

| Condition | Behavior |
|-----------|----------|
| `expected_session_id` is `None` (no `-c` injected) | No comparison, no warning |
| `expected_session_id` is `Some(uuid)` AND actual matches | Silent success |
| `expected_session_id` is `Some(uuid)` AND actual differs | `[Runner] warning: session mismatch — expected {uuid}, got {actual} (BUG-320 detected)` on stderr; result still printed; exit 0 |
| `expected_session_id` is `Some(uuid)` AND result has no `session_id` field | Silent — non-JSON output or `--output-format text` mode; comparison skipped |

### Rationale

BUG-320 documented that `clr` can resume the wrong session when a JSONL file transiently appears in the expected session storage directory (e.g. from a concurrent Claude Code invocation with an overlapping project path). By the time `-c` is injected the storage directory contains a UUID; once Claude Code starts, it may load a different, more-recently-modified session than the one observed during pre-spawn detection.

The warning surfaces this condition without failing the run: by the time the mismatch is detected, Claude has already responded using the wrong session. Aborting post-hoc would discard a completed response. The warning provides the signal needed to diagnose the bug in production without requiring a dry-run probe.

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

### Fixed Defects

**BUG-320 (diagnostic hardening) — `clr -c` can resume wrong session:**
Pre-spawn storage inspection can observe UUID-A; Claude Code loads UUID-B because a different session file became more-recently-modified between inspection and spawn. This warning detects the symptom at result time. Root cause analysis in `task/claude_runner/bug/unverified/320_continue_flag_selects_global_not_cwd.md`.

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
