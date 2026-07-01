# Control CLR runner diagnostic output on stderr

**Persona:** Developer running `clr` in automation pipelines or scripts who needs clean stdout with no CLR runner chatter, or a developer troubleshooting runner behaviour who wants to see the exact command assembled.
**Goal:** Control whether CLR runner diagnostics (retry messages, gate-wait messages, warnings) appear on stderr. Use `--quiet` to silence them for pipeline use; use `--trace` to expose full command detail for debugging.
**Benefit:** Keeps automation output clean by default and provides detailed diagnostics on demand.
**Priority:** Medium

### Acceptance Criteria

- `--quiet` suppresses retry progress messages, gate-wait messages, and informational warnings from CLR on stderr
- `--quiet` does NOT suppress fatal spawn-failure errors (always emitted)
- `--quiet` does NOT suppress `--dry-run` preview output (core feature output, not a diagnostic)
- `--trace` prints the assembled env block and command to stderr before execution regardless of `--quiet`
- `CLR_QUIET=1` produces identical suppression to `--quiet` flag
- Without `--quiet`, CLR diagnostics appear on stderr (default; shows retry/gate/warning output)
- Runner diagnostics go to stderr; Claude's captured output on stdout is unaffected

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`run`](../command/01_run.md) | Default command; `--quiet` gates CLR diagnostic output |
| 5 | [`ask`](../command/05_ask.md) | Alias for `run`; same `--quiet` behavior |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | `--quiet` is a runner control flag |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 11 | [`--dry-run`](../param/011_dry_run.md) | Always emits preview regardless of `--quiet` |
| 13 | [`--trace`](../param/013_trace.md) | Emits env+command diagnostic; always fires regardless of `--quiet` |
| 74 | [`--quiet`](../param/074_quiet.md) | Suppress non-fatal CLR runner diagnostics |

### Workflow Steps

1. `clr --quiet "task"` — suppress CLR diagnostic output on stderr
2. `CLR_QUIET=1 clr "task"` — apply quiet mode via environment variable
3. `clr --quiet --dry-run "task"` — dry-run output still shows despite `--quiet`
4. `clr --quiet --trace "task"` — trace output still shows despite `--quiet`

### Related User Stories

| # | User Story | Relationship |
|---|------------|--------------|
| 4 | [Dry-run Preview](004_dry_run_preview.md) | `--dry-run` preview unaffected by `--quiet` |
| 8 | [Trace Execution](008_trace_execution.md) | `--trace` is the complementary diagnostic expansion flag |
