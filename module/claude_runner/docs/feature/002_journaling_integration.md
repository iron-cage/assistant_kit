# Feature: Journaling Integration

### Scope

- **Purpose**: Document the integration of `claude_journal` into `clr` for automatic event journaling.
- **Responsibility**: Define how `clr` calls `JournalWriter` at execution boundaries to record events.
- **In Scope**: Journal writer initialization, event emission points, journal level control, truncation behavior.
- **Out of Scope**: Journal file format (-> `claude_journal/docs/feature/001_event_journaling.md`), viewer CLI (-> `claude_journal_viewer/docs/`).

### Design

`clr` integrates with `claude_journal::JournalWriter` to record events at natural
execution boundaries. The journal writer is initialized once in `run_cli()` and
threaded through execution functions.

**Journal level:** Controlled by `--journal <level>` param (default: `full`).
- `full`: Record all event fields including complete stdout/stderr (truncated at 1 MB per field)
- `meta`: Record event metadata (timestamp, command, exit code, duration, cost, model) without stdout/stderr content
- `off`: Disable journaling entirely (no-op writer)

**Journal directory:** `--journal-dir <path>` overrides the default `~/.clr/journal/`.
Also configurable via `CLR_JOURNAL_DIR` env var. Resolution: CLI > env > default.

**Emission points:** Events are emitted at these locations in the execution flow:

| Event Type | Emission Point | Source Function |
|------------|----------------|-----------------|
| `execution` | After `run_print_mode()` subprocess completes | `execution.rs` |
| `credential` | After `dispatch_isolated()`/`dispatch_refresh()` completes | `cred_parse.rs` |
| `gate_wait` | When `wait_for_session_slot()` blocks | `gate.rs` |
| `retry` | On each retry attempt in `run_print_mode()` | `execution.rs` |
| `timeout` | On `poll_timeout()` firing (exit 4) | `execution.rs` |
| `runner_retry` | On each `apply_runner_retry()` attempt | `execution.rs` |
| `validation_retry` | On each expect-validation retry | `execution.rs` |
| `interactive` | On `run_interactive()` session start/end | `execution.rs` |

**Truncation:** At `full` level, stdout/stderr content exceeding 1 MB is truncated
to 1 MB with a `\n[truncated at 1MB]` suffix. This prevents journal files from
growing unboundedly with large subprocess outputs.

**Error handling:** Journal write failures are logged to stderr at verbosity >= 3
but never cause `clr` to exit non-zero. Journaling is best-effort — it must not
interfere with the primary execution path.

### Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC-001 | `clr "test" --journal full` produces an `execution` event in `~/.clr/journal/YYYY-MM-DD.jsonl` |
| AC-002 | `clr "test" --journal meta` produces an event without stdout/stderr fields |
| AC-003 | `clr "test" --journal off` produces no journal event |
| AC-004 | `clr "test"` (no --journal flag) journals at `full` level by default |
| AC-005 | `clr "test" --journal-dir /tmp/j` writes to `/tmp/j/YYYY-MM-DD.jsonl` |
| AC-006 | `CLR_JOURNAL_DIR=/tmp/j clr "test"` writes to `/tmp/j/` |
| AC-007 | Stdout exceeding 1 MB is truncated in the journal event at `full` level |
| AC-008 | Journal write failures do not change `clr` exit code |
| AC-009 | Gate wait events are journaled when `wait_for_session_slot()` blocks |
| AC-010 | Retry events include error_class, attempt number, and delay |
| AC-011 | Timeout events include timeout_secs and partial_stdout |
| AC-012 | Interactive session events include session_duration |

### Cross-References

- Library API: [claude_journal/docs/api/001_journal_writer.md](../../claude_journal/docs/api/001_journal_writer.md)
- Event schema: [claude_journal/docs/feature/002_event_schema.md](../../claude_journal/docs/feature/002_event_schema.md)
- Viewer: [claude_journal_viewer/docs/feature/001_cli_viewing.md](../../claude_journal_viewer/docs/feature/001_cli_viewing.md)
- Params: [cli/param/072_journal.md](../cli/param/072_journal.md), [cli/param/073_journal_dir.md](../cli/param/073_journal_dir.md)

### Since

Planned (unreleased)
