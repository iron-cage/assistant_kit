# User Story :: 004. Dry-run Preview

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

| # | Command | Notes |
|---|---------|-------|
| 1 | [`run`](../001_command.md#command--1-run) | `--dry-run` previews the `run` invocation |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`--dry-run`](../param/011_dry_run.md) | Gate: preview without execution |
| 2 | [`--trace`](../param/013_trace.md) | Related: print to stderr then execute (not dry) |
| 3 | [`--verbosity`](../param/012_verbosity.md) | Level 4+ also shows preview before execution |

### Related User Stories

| # | User Story | Relationship |
|---|-----------|-------------|
| 1 | [008 Trace Execution](008_trace_execution.md) | `--trace` is the execute-as-well variant |
