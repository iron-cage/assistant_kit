# Capture output to a file while printing to stdout

**Persona:** Developer or CI system that runs `clr` in print mode and needs the output both on stdout (for immediate display or piping) and persisted to a file (for logging, review, or downstream consumption) without shell redirection.
**Goal:** Capture Claude's output to a file and print it to stdout in a single `clr` invocation, so that automated pipelines can log results without losing real-time visibility.
**Benefit:** Provides simultaneous real-time visibility and durable log capture without shell redirection gymnastics.
**Priority:** Medium

### Acceptance Criteria

- `clr -p --output-file /path/to/out.txt "task"` writes captured stdout to `/path/to/out.txt`
  AND prints to stdout; both destinations contain identical content
- If the file path is not writable (permission denied, missing directory), `clr` exits 1 and
  emits the OS error to stderr; the subprocess is not affected
- In dry-run mode (`--dry-run`), the file is NOT created; the path is shown in the describe output
- `CLR_OUTPUT_FILE=/path/to/out.txt clr -p "task"` applies the env var when `--output-file` is
  absent from CLI
- When combined with `--strip-fences`, the fence-stripped text is written to the file and printed
  to stdout (both destinations receive identical stripped content)
- `--output-file` is orthogonal to `--file`; both can be used simultaneously

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`run`](../command/01_run.md) | Primary command; tee behavior applies in print mode |
| 5 | [`ask`](../command/05_ask.md) | Also supported; same behavior |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | `--output-file` is a Runner Control parameter |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 29 | [`--output-file`](../param/029_output_file.md) | File destination for captured stdout |
| 2 | [`--print`](../param/002_print.md) | Activates print mode (capture); required for tee behavior |
| 11 | [`--dry-run`](../param/011_dry_run.md) | Skips file creation in dry-run mode |
| 26 | [`--strip-fences`](../param/026_strip_fences.md) | Content is stripped before reaching both destinations |
| 25 | [`--file`](../param/025_file.md) | Orthogonal — stdin input direction, unrelated to output capture |

### Workflow Steps

1. `clr -p --output-file /path/to/out.txt "task"` — write output to file and print to stdout simultaneously
2. `clr -p --output-file out.txt --strip-fences "task"` — strip fences then tee to file and stdout
3. `CLR_OUTPUT_FILE=/path/to/out.txt clr -p "task"` — set output file via environment variable
4. `clr -p --output-file out.txt --dry-run "task"` — verify path shown in preview without creating the file

### Related User Stories

| # | User Story | Relationship |
|---|------------|--------------|
| 2 | [Print Mode Capture](002_print_mode_capture.md) | `--output-file` builds on print mode capture |
| 12 | [Code Block Extraction](012_code_block_extraction.md) | `--strip-fences` can be combined with `--output-file` |
| 18 | [Env-var Configuration](018_env_var_configuration.md) | `CLR_OUTPUT_FILE` is an instance of the CLR_* env var system |
