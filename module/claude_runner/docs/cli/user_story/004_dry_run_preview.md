# CLI User Story: Dry-run Preview

### Scope

- **Purpose**: Document the --dry-run preview mode for inspecting the assembled subprocess command.
- **Responsibility**: Define acceptance criteria for dry-run showing the full command without executing.
- **In Scope**: --dry-run output format, default injection visibility, env var reflection, exit 0.
- **Out of Scope**: Trace mode with execution (→ 008_trace_execution.md).

### Persona

Developer debugging a flag combination or verifying the exact subprocess command before committing to execution.

### Goal

Inspect the fully assembled `claude` subprocess command — including all default injections — without spawning the subprocess.

### Acceptance Criteria

- `--dry-run` prints the assembled command to stdout and exits 0 without executing
- Output includes all default-injected flags: `-c`, `--dangerously-skip-permissions`, `--chrome`, `--effort max`, and the `ultrathink` message suffix
- Output reflects the effective values after all CLI flags and env var overrides are applied
- Any combination of other flags can be previewed via `--dry-run` before running for real

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`run`](../command/01_run.md) | Default command; `--dry-run` prevents execution |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | `--dry-run` is a runner control flag |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 11 | [`--dry-run`](../param/011_dry_run.md) | Gate: preview without execution |
| 12 | [`--verbosity`](../param/012_verbosity.md) | Level 4+ also shows preview before execution |
| 13 | [`--trace`](../param/013_trace.md) | Related: print to stderr then execute (not dry) |

### Related User Stories

| # | User Story | Relationship |
|---|------------|--------------|
| 8 | [Trace Execution](008_trace_execution.md) | `--trace` is the execute-as-well variant |
