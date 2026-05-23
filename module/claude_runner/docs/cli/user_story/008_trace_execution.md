# User Story :: 008. Trace Execution

### Persona

Developer who wants to see exactly what is called under the hood — CLI commands, subprocess arguments, credential paths — while still letting execution proceed normally.

### Goal

Print diagnostic details to stderr — like shell `set -x` — then execute normally. Works across all commands that spawn a subprocess.

### Acceptance Criteria

- `--trace` on `run`: emits env vars and full `claude` command to stderr before launch
- `--trace` on `isolated`: emits creds path, temp HOME, timeout, forwarded args to stderr
- `--trace` on `refresh`: emits creds path, temp HOME, timeout, fixed args `["--print", "."]` to stderr
- Subprocess executes after the trace output (unlike `--dry-run` which does not execute)
- Trace output goes to stderr only; captured stdout in print mode is unaffected
- Independent of `--verbosity` level: always prints when set

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| 1 | [`run`](../001_command.md#command--1-run) | `--trace` shows env vars + assembled claude command |
| 2 | [`isolated`](../001_command.md#command--2-isolated) | `--trace` shows credential isolation details |
| 3 | [`refresh`](../001_command.md#command--3-refresh) | `--trace` shows refresh call details |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`--trace`](../param/013_trace.md) | Print env+command to stderr then execute |
| 2 | [`--dry-run`](../param/011_dry_run.md) | Related: preview only, no execution |
| 3 | [`--verbosity`](../param/012_verbosity.md) | Level 4 also shows preview but via different path |

### Related User Stories

| # | User Story | Relationship |
|---|-----------|-------------|
| 1 | [004 Dry-run Preview](004_dry_run_preview.md) | `--dry-run` is the non-executing variant |
| 2 | [006 Verbose Debugging](006_verbose_debugging.md) | Complementary diagnostic: `--verbosity` gates runner output |
