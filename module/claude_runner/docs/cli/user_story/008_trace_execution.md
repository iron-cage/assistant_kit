# User Story :: 008. Trace Execution

### Persona

Developer who wants to see exactly what environment variables and command are sent to the claude subprocess, while still letting the subprocess run normally.

### Goal

Print the assembled environment and subprocess command to stderr — like shell `set -x` — then execute normally.

### Acceptance Criteria

- `--trace` emits env vars and full command to stderr before launching the subprocess
- Subprocess executes after the trace output (unlike `--dry-run` which does not execute)
- Trace output goes to stderr only; captured stdout in print mode is unaffected
- Independent of `--verbosity` level: always prints when set

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| 1 | [`run`](../command.md#command--1-run) | `--trace` applies to `run` subprocess |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`--trace`](../param/13_trace.md) | Print env+command to stderr then execute |
| 2 | [`--dry-run`](../param/11_dry_run.md) | Related: preview only, no execution |
| 3 | [`--verbosity`](../param/12_verbosity.md) | Level 4 also shows preview but via different path |

### Related User Stories

| # | User Story | Relationship |
|---|-----------|-------------|
| 1 | [004 Dry-run Preview](004_dry_run_preview.md) | `--dry-run` is the non-executing variant |
| 2 | [006 Verbose Debugging](006_verbose_debugging.md) | Complementary diagnostic: `--verbosity` gates runner output |
