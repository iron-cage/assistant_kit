# print_bg_wait_ceiling_ms

Bounds how long print/headless mode waits for outstanding background tasks before exiting.

## Type

Env

## Environment Variable

```
CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS
```

## Default

`0` (wait indefinitely — disables the ceiling-based forced-sweep path
entirely; does **not** mean "exit immediately") — set unconditionally by
`ClaudeCommand::new()`.

Standard `claude` default: `600000` (10 minutes) — confirmed via the fallback
constant in the installed binary. See the binary-perspective entry:
[`contract/claude_code/docs/param/131_print_bg_wait_ceiling_ms.md`](../../../../contract/claude_code/docs/param/131_print_bg_wait_ceiling_ms.md),
which documents and retracts an earlier "exit immediately" misreading of
`0` — the `ra > 0` guard that gates the forced-sweep path is permanently
false at `0`, so the path never fires. Independently confirmed by causal
testing against a live process:
[`docs/claude_code_background_task_env_vars.md`](../../../../docs/claude_code_background_task_env_vars.md).

## Description

In `-p`/`--print` mode — where the process is expected to exit once its main
turn completes rather than stay interactive — a positive value here caps how
long the process keeps waiting for outstanding background shells, agents, or
workflows before finalizing exit and sweeping (killing) whatever is still
running. At `0`, that ceiling-based sweep path is disabled outright — the
process waits indefinitely for backgrounded Agent/Workflow dispatches. A
separate, ceiling-independent ~5s grace period still applies to plain
background shells regardless of this variable's value.

`claude_runner_core` sets this Tier 1 default to `0` so that backgrounded
Agent/Workflow dispatches are never force-killed by `claude`'s own
print-mode wind-down just because the main turn ended — the most generous
value on this axis, consistent with the other Tier 1 defaults
(`bash_timeout`, `bash_max_timeout`) that also grant the subprocess more
room than the binary's own standard. The actual backstop bounding total
wall-clock time in this configuration is `clr`'s own outer `--timeout`
watchdog (`DEFAULT_PRINT_TIMEOUT_SECS` in
`claude_runner/src/cli/execution.rs`) — **not** the `--max-sessions`
gate/slot mechanism (`gate_poll_secs`/`gate_max_attempts`), which governs
concurrency admission for new `clr` invocations and has no relationship to
waiting for a running session's own background work. A sibling project in
this workspace (`claude_storage/.claude/settings.local.json`) independently
arrived at the same `"0"` value.

## Builder API

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_print_bg_wait_ceiling_ms( 600_000 );  // opt back into the binary's own 10-minute wait
```

Builder method: `with_print_bg_wait_ceiling_ms()`

## Examples

```bash
# Inspect the default in a dry-run
clr --dry-run --message "hi"
# ... CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS=0 ...
```

## Notes

- Only takes effect in `-p`/`--print` mode (see
  [042_print.md](042_print.md)); irrelevant to interactive sessions.
- No `clr`-level CLI flag or `CLR_*` env var currently overrides this — same
  pattern as `bash_timeout`/`bash_max_timeout`/`auto_continue`/`telemetry`
  (Tier 1 defaults set purely at the `claude_runner_core` level).
- Sibling to [128_bg_tasks_report_running](../../../../contract/claude_code/docs/param/128_bg_tasks_report_running.md) —
  the "is a background task still outstanding" signal this ceiling's sweep
  logic consumes.
- Like the other Tier 1 defaults, `0` here is the most generous value on
  this axis (indefinite wait vs. the binary's own bounded 10-minute
  default) — not a "hands-off" delegation to a separate wrapper-level
  mechanism; see § Description above for the actual backstop.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [042_print.md](042_print.md) | `-p`/`--print` mode this ceiling applies to |
| doc | [../../../../contract/claude_code/docs/param/131_print_bg_wait_ceiling_ms.md](../../../../contract/claude_code/docs/param/131_print_bg_wait_ceiling_ms.md) | Binary-perspective reference with decompiled sweep logic |
| doc | [../../../../docs/claude_code_background_task_env_vars.md](../../../../docs/claude_code_background_task_env_vars.md) | Independent, empirically-tested (live-process) confirmation of the `0` = wait-indefinitely reading |
