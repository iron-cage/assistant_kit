# Feature: CLI Invocation Telemetry

### Scope

- **Purpose**: Append one redacted `Command` event to the `claude_journal` event log for every `clp`/`claude_profile` invocation, giving `claude_journal_viewer` a permanent, queryable record of CLI usage history without any change to the underlying command's own behavior or exit code.
- **Responsibility**: `run_cli()` in `src/lib.rs` — the single shared dispatch entry point behind both the `claude_profile` and `clp` binary targets — captures wall-clock duration around the existing dispatch pipeline, then calls `telemetry::record()` to append one `EventType::Command` event carrying `user`/`host`/`args` (redacted)/`exit_code`/`duration_ms`. The write is best-effort: any failure (unresolvable directory, I/O error) is silently swallowed and never alters the command's own exit code or output.
- **In Scope**: `src/telemetry.rs` (new module — `record()`, journal directory resolution, user/host detection, argument redaction); the `cli::run()` signature change (returns `i32` instead of calling `std::process::exit` internally) needed to observe the real exit code before the process terminates; `run_cli()` wiring in `src/lib.rs`; `claude_journal` and `json_redact` added as dependencies.
- **Out of Scope**: Chart generation/rendering (tasks 471, 472). Any change to `claude_journal`'s or `json_redact`'s own internals (owned by tasks 467, 468 respectively). Any change to `claude_profile_core` (account/token domain logic untouched). A CLI flag to disable telemetry (not requested — YAGNI). Reading back or displaying journal contents from within `clp` itself (that's `claude_journal_viewer`'s job).

### Design

Both binary targets (`src/main.rs`, `src/bin/clp.rs`) are 10-line wrappers calling only `claude_profile::run_cli()`, so instrumenting `run_cli()` once covers both binaries.

Before this feature, `cli::run()` called `std::process::exit()` directly at every error/help exit point, terminating the process immediately with no opportunity for a caller to observe the outcome. `cli::run()` now returns `i32` instead: every former `std::process::exit(N)` call site became `return N;`, and the final `Ok`/`Err` match arms in the execute phase return `0` / `exit_code_for(&e)` as expressions rather than calling `process::exit`. This is a pure control-flow change — the value returned at each exit point is identical to what was previously passed to `process::exit`.

`run_cli()` wraps the dispatch call:

```rust
let start       = std::time::Instant::now();
let exit_code   = cli::run( &binary, &argv );
let duration_ms = u64::try_from( start.elapsed().as_millis() ).unwrap_or( u64::MAX );

telemetry::record( &argv, exit_code, duration_ms );

if exit_code != 0
{
  std::process::exit( exit_code );
}
```

For the success path (`exit_code == 0`), this is behaviorally identical to before: `run_cli()` returns normally and the process exits 0 naturally. For error paths, telemetry now runs between dispatch completion and process termination, then `std::process::exit(exit_code)` is called explicitly — the exit code reaching the OS is unchanged, only its timing is delayed by a best-effort journal write.

`telemetry::record()` (all failures swallowed via `let _ = writer.append(&event);`):

- **Journal directory**: `CLR_JOURNAL_DIR` env var if set and non-empty, else `~/.clr/journal` — mirrors (without duplicating the `dir::` CLI-param tier of) `claude_journal_viewer::output::resolve_journal_dir`'s own env/default resolution order, so events land in the same default location `claude_journal_viewer` already reads from with zero extra configuration.
- **User/host detection**: `$USER` env var for user (`"unknown"` fallback); `$HOSTNAME` env var for host, falling back to reading `/proc/sys/kernel/hostname` (a plain file read). Neither spawns a subprocess (forbidden by this crate's own `tests/cli/invariant_test.rs::no_process_execution_in1_src_contains_zero_std_process` architectural boundary test) nor uses `unsafe` FFI (denied by workspace-wide lints) — the two approaches available for hostname detection on most systems.
- **Argument redaction**: `argv` is joined with spaces, passed through `json_redact::redact_str()` with `RedactionPolicy::default()` (its built-in deny-list: `token`, `password`, `secret`, `authorization`, `api_key`, `apikey`, `key`, `credential`), then split back on spaces — preserving individual-argument structure in the persisted `args` field while ensuring no raw sensitive value reaches disk.

### Acceptance Criteria

- **AC-01**: Every `clp`/`claude_profile` invocation appends exactly one `Command` event to the journal — never zero on success, never more than one.
- **AC-02**: The persisted `args` field has sensitive values (matching `json_redact`'s default deny-list) redacted before the event is written — no raw sensitive value reaches disk.
- **AC-03**: A journal write failure (unwritable directory, I/O error) never changes the underlying command's own exit code or aborts it — telemetry is observability, never load-bearing.
- **AC-04**: Events land in the same default directory `claude_journal_viewer` reads from (`~/.clr/journal`, or `CLR_JOURNAL_DIR` when set) — verified by asserting the written event is retrievable via `claude_journal::JournalReader::open()` on that exact resolved path.
- **AC-05**: `user`, `host`, `args`, `exit_code`, and `duration_ms` are all populated on every written `Command` event.

### Bugs

_(none — no defects filed against this feature)_

### Dependencies

| File | Relationship |
|------|-------------|
| [../../../claude_journal/docs/api/003_event_type.md](../../../claude_journal/docs/api/003_event_type.md) | `EventType::Command` and the `user`/`host`/`args` `EventFields` this feature's writes populate (schema landed by task 467) |
| `json_redact` crate (`module/json_redact/`) | `redact_str()` — the redaction primitive applied to `args` before persistence (task 468) |

### CLI Parameters

_(none — this feature has no CLI-facing parameter; it observes every invocation unconditionally)_

### Features

_(none — no other feature doc currently cross-references CLI invocation telemetry)_

### Sources

| File | Relationship |
|------|-------------|
| `src/telemetry.rs` | `record()`, journal directory resolution, user/host detection, argument redaction — the feature's entire implementation |
| `src/lib.rs` | `run_cli()` — captures start time, calls `cli::run()`, computes duration, calls `telemetry::record()`, exits explicitly on non-zero |
| `src/cli.rs` | `run()` — returns `i32` instead of calling `std::process::exit` internally, so `run_cli()` can observe the exit code before the process terminates |
| `Cargo.toml` | `claude_journal` and `json_redact` added as optional dependencies, activated by the `enabled` feature |

### Tests

| File | Relationship |
|------|-------------|
| `tests/cli/telemetry_test.rs` | T01–T06 + M01 — successful/failing invocations, sensitive-argument redaction, unwritable-directory failure isolation, `CLR_JOURNAL_DIR` set/unset directory routing, exactly-one-event-per-invocation measurement |
| `tests/cli/cross_cutting_test.rs` | Exit-code preservation across success/usage-error/unknown-command paths — regression coverage for the `cli::run()` return-value refactor |
| `tests/cli/invariant_test.rs` | `no_process_execution_in1_src_contains_zero_std_process` — confirms `telemetry.rs`'s user/host detection introduces no subprocess spawn |
