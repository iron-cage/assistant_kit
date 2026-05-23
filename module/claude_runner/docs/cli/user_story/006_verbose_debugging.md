# User Story :: 006. Verbose Debugging

### Persona

Developer troubleshooting unexpected runner behavior who wants to see diagnostic output at varying levels of detail.

### Goal

Control the level of runner diagnostic output to understand what `clr` is doing: from fully silent to step-by-step command preview to internal state.

### Acceptance Criteria

- `--verbosity 4` prints a command preview to stderr before execution
- `--verbosity 5` adds internal state, timing, and path information
- `--verbosity 0` suppresses all runner diagnostic output (silent automation)
- `--dry-run` output is always shown regardless of verbosity level
- Runner diagnostics go to stderr; Claude's captured output on stdout is unaffected

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| 1 | [`run`](../command.md#command--1-run) | `--verbosity` gates runner diagnostic output |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`--verbosity`](../param/12_verbosity.md) | Runner output gate level (0–5) |
| 2 | [`--dry-run`](../param/11_dry_run.md) | Always emits preview regardless of verbosity |
| 3 | [`--trace`](../param/13_trace.md) | Independent of verbosity: always prints env+cmd |

### Related User Stories

| # | User Story | Relationship |
|---|-----------|-------------|
| 1 | [004 Dry-run Preview](004_dry_run_preview.md) | `--dry-run` is related to diagnosis |
| 2 | [008 Trace Execution](008_trace_execution.md) | `--trace` is the complementary diagnostic flag |
