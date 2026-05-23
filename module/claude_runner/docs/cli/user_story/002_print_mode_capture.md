# User Story :: 002. Print Mode Capture

### Persona

Developer or automation script that needs Claude's response as capturable stdout — to assign to a variable, pipe to another tool, or redirect to a file.

### Goal

Send a prompt to Claude and capture the response on stdout with no interactive TTY behavior.

### Acceptance Criteria

- Providing `[MESSAGE]` defaults to print mode; `-p`/`--print` is not required
- `-p`/`--print` is an explicit alias that works identically (backward compatibility)
- Captured stdout is clean and pipeble: `result=$(clr "task")` and `clr "task" | grep X` work
- `--strip-fences` removes the outermost code fence from stdout when bare code is needed
- Print mode without a message exits with error code 1 and a clear error message

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| 1 | [`run`](../command.md#command--1-run) | Default command; message triggers print mode |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`[MESSAGE]`](../param/01_message.md) | Prompt text; presence triggers print mode |
| 2 | [`--print`](../param/02_print.md) | Explicit print mode selector (alias) |
| 3 | [`--strip-fences`](../param/26_strip_fences.md) | Remove outermost code fence from captured output |
| 4 | [`--model`](../param/03_model.md) | Select model for the response |
