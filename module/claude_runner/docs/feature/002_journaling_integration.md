# Feature: Journaling Integration

### Scope

- **Purpose**: Document the integration of `claude_journal` into `clr` for automatic event journaling.
- **Responsibility**: Define how `clr` calls `JournalWriter` at execution boundaries to record events.
- **In Scope**: Journal writer initialization, event emission points, journal level control, truncation behavior, once-daily retention auto-prune.
- **Out of Scope**: Journal file format (-> `claude_journal/docs/feature/001_event_journaling.md`), viewer CLI (-> `claude_journal_viewer/docs/`), pruning mechanics — cutoff math and filename filtering (-> `claude_journal/docs/feature/003_rotation.md`).

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

**Attribution stamping (task 542):** Every event, regardless of type, is stamped
with attribution fields at the append boundary (`stamp_attribution()` in
`execution.rs` — called by the shared `emit()` helper and by `gate.rs`'s and
`cred_parse.rs`'s direct appends). Stamping order matters: `dir` first falls back
to the process cwd when no explicit `--dir`/`--to` populated it (explicit values
always win), then `agent_id` is composed from that effective dir via
`claude_journal::compose_agent_id( user, host, dir )` — so both fields always
describe the same location. `user`/`host` come from `claude_profile_core`'s env
fallback chains (`$USER` → `$USERNAME` → `"user"`; `$HOSTNAME` → `/etc/hostname`
→ `"local"`). `account` resolves through a first-hit-wins hierarchy: non-empty
`CLR_ACCOUNT` env override → this machine's active-account marker in the default
credential store (`claude_profile_core::account::{default_credential_store,
active_account}` — the marker holds only the account name, email or redirect
profile name, never token material) → absent. The triple `(user, host, account)`
is resolved once per process and cached, keeping the store read off the
per-event path.

**Truncation:** At `full` level, stdout/stderr content exceeding 1 MB is truncated
to 1 MB with a `\n[truncated at 1MB]` suffix. This prevents journal files from
growing unboundedly with large subprocess outputs.

**Error handling:** Journal write failures are logged to stderr unless `--quiet`
but never cause `clr` to exit non-zero. Journaling is best-effort — it must not
interfere with the primary execution path.

**Retention auto-prune:** When journal resolution runs (any journaling-enabled
invocation), `clr` prunes journal files older than the keep window — once per UTC
day at most, gated by a `-last_prune` stamp file in the journal dir holding the
last attempt's date. The window defaults to 30 days; `CLR_JOURNAL_KEEP` overrides
it (`"45"` or `"45d"` = days; `"0"` or `"off"` disables pruning — then no stamp is
written, so re-enabling takes effect on the next invocation; an unparsable value
warns on stderr and the default applies). Deletion is filename-date-based via
`claude_journal::rotation::prune_by_age` — only `YYYY-MM-DD.jsonl` files qualify
and today's file is structurally never deleted. Best-effort like all journaling:
prune failures never abort the runner.

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
| AC-012 | Interactive session events include the session duration as `duration_ms`, at both completion sites (blocking and timeout-polling) — BUG-539 |
| AC-013 | Validation-retry events are emitted when `--expect-strategy retry` fires a retry |
| AC-014 | A journaling-enabled run deletes journal files older than the keep window (default 30 days) and writes the `-last_prune` stamp |
| AC-015 | A second run on the same UTC day does not prune again (stamp gate) |
| AC-016 | `CLR_JOURNAL_KEEP=off` (or `0`) disables pruning and writes no stamp; `Nd`/`N` sets the window; invalid values warn on stderr and fall back to 30 days |
| AC-017 | Auto-prune never deletes today's file or files not matching `YYYY-MM-DD.jsonl` |
| AC-018 | No newly-emitted event lacks `dir`: absent an explicit `--dir`/`--to`, the process cwd is stamped in (explicit values always win; task 542) |
| AC-019 | No newly-emitted event lacks `user`, `host`, or `agent_id` (`{user}@{host}{abs_dir}/` via `claude_journal::compose_agent_id`; `agent_id` may be absent only when the cwd itself is unresolvable) |
| AC-020 | `account` is present whenever an identity resolves (`CLR_ACCOUNT` override first, else the active-account marker in the credential store) and absent otherwise; its value is never token material |

### Features

| File | Relationship |
|------|--------------|
| [feature/001_runner_tool.md](001_runner_tool.md) | Parent feature — runner tool design (journaling section) |
| [claude_journal/docs/feature/002_event_schema.md](../../../claude_journal/docs/feature/002_event_schema.md) | Event schema for journal entries |
| [claude_journal_viewer/docs/feature/001_cli_viewing.md](../../../claude_journal_viewer/docs/feature/001_cli_viewing.md) | Viewer CLI for journal files |

### APIs

| File | Relationship |
|------|--------------|
| [claude_journal/docs/api/001_journal_writer.md](../../../claude_journal/docs/api/001_journal_writer.md) | `JournalWriter` API — write-side contract |
| [claude_journal/docs/api/004_rotation.md](../../../claude_journal/docs/api/004_rotation.md) | `rotation` API — `prune_by_age` consumed by the auto-prune |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/072_journal.md](../cli/param/072_journal.md) | `--journal` level control (full / meta / off) |
| [cli/param/073_journal_dir.md](../cli/param/073_journal_dir.md) | `--journal-dir` directory override |

### Since

1.2.0
