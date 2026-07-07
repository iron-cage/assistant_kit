# print_bg_wait_ceiling_ms

Bounds how long print/headless mode waits for outstanding background tasks before exiting.

## Type

Env

## Environment Variable

```
CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS
```

## Default

`0` (exit immediately — do not wait at all) — set unconditionally by
`ClaudeCommand::new()`.

Standard `claude` default: `600000` (10 minutes) — confirmed via the fallback
constant in the installed binary. See the binary-perspective entry:
[`contract/claude_code/docs/param/131_print_bg_wait_ceiling_ms.md`](../../../../contract/claude_code/docs/param/131_print_bg_wait_ceiling_ms.md).

## Description

In `-p`/`--print` mode — where the process is expected to exit once its main
turn completes rather than stay interactive — this value caps how long the
process keeps waiting for outstanding background shells, agents, or workflows
before finalizing exit and sweeping (killing) whatever is still running.

`claude_runner_core` sets this Tier 1 default to `0` — exit immediately,
never wait — because `clr` already owns background-task waiting at the
wrapper level via its own `gate_poll_secs`/`gate_max_attempts` polling (see
`module/claude_runner/docs/cli/003_env_param.md`). Leaving the binary's own
10-minute internal wait active would duplicate that logic: `claude` would
block inside its own print-mode wind-down while `clr` is separately polling
for the same outstanding work, delaying `clr`'s own exit for no benefit. A
sibling project in this workspace (`claude_storage/.claude/settings.local.json`)
independently arrived at the same `"0"` value for the same headless/CI reason.

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
- Unlike the other Tier 1 defaults, `0` here is not "generous" but
  "hands-off" — `clr` deliberately defers all background-task wait behavior
  to its own wrapper-level gate rather than the binary's internal ceiling.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [042_print.md](042_print.md) | `-p`/`--print` mode this ceiling applies to |
| doc | [../../../../contract/claude_code/docs/param/131_print_bg_wait_ceiling_ms.md](../../../../contract/claude_code/docs/param/131_print_bg_wait_ceiling_ms.md) | Binary-perspective reference with decompiled sweep logic |
