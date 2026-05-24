# CLI User Story: Verbose Debugging

### Scope

- **Purpose**: Document verbosity control for runner diagnostic output at different levels.
- **Responsibility**: Define acceptance criteria for --verbosity gate behavior from 0 to 5.
- **In Scope**: Verbosity levels 0–5, stderr diagnostic output, --dry-run independence, --trace independence.
- **Out of Scope**: Subprocess output (unaffected by --verbosity), trace mode (→ 008_trace_execution.md).

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

| # | Command | Role |
|---|---------|------|
| 1 | [`run`](../command/01_run.md) | Default command; `--verbosity` gates diagnostic output |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | `--verbosity` is a runner control flag |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 11 | [`--dry-run`](../param/011_dry_run.md) | Always emits preview regardless of verbosity |
| 12 | [`--verbosity`](../param/012_verbosity.md) | Runner output gate level (0–5) |
| 13 | [`--trace`](../param/013_trace.md) | Independent of verbosity: always prints env+cmd |

### Related User Stories

| # | User Story | Relationship |
|---|------------|--------------|
| 4 | [Dry-run Preview](004_dry_run_preview.md) | `--dry-run` is related to diagnosis |
| 8 | [Trace Execution](008_trace_execution.md) | `--trace` is the complementary diagnostic flag |
