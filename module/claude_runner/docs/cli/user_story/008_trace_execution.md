# CLI User Story: Trace Execution

### Scope

- **Purpose**: Document --trace mode for printing diagnostic details to stderr before executing.
- **Responsibility**: Define acceptance criteria for trace output across all executing subcommands.
- **In Scope**: --trace on run/isolated/refresh, stderr output, execution after trace, quiet independence.
- **Out of Scope**: Dry-run (no execution) variant (→ 004_dry_run_preview.md).

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
- Independent of `--quiet`: always emits to stderr even when diagnostics are suppressed

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`run`](../command/01_run.md) | `--trace` emits env+command before launch |
| 2 | [`isolated`](../command/02_isolated.md) | `--trace` emits creds path and temp HOME |
| 3 | [`refresh`](../command/03_refresh.md) | `--trace` emits creds path and fixed args |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | `--trace` is a runner control flag |
| 4 | [Credential Operations](../param_group/04_credential_operations.md) | `--trace` also applies to credential commands |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 11 | [`--dry-run`](../param/011_dry_run.md) | Related: preview only, no execution |
| 74 | [`--quiet`](../param/074_quiet.md) | Suppresses CLR diagnostics but NOT trace output |
| 13 | [`--trace`](../param/013_trace.md) | Print env+command to stderr then execute |

### Related User Stories

| # | User Story | Relationship |
|---|------------|--------------|
| 4 | [Dry-run Preview](004_dry_run_preview.md) | `--dry-run` is the non-executing variant |
| 6 | [Quiet Mode and Diagnostic Control](006_verbose_debugging.md) | Complementary diagnostic: `--quiet` suppresses non-fatal CLR output |
