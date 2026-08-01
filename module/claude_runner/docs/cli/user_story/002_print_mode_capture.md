# Send a prompt and capture the response on stdout

**Persona:** Developer or automation script that needs Claude's response as capturable stdout — to assign to a variable, pipe to another tool, or redirect to a file.
**Goal:** Send a prompt to Claude and capture the response on stdout with no interactive TTY behavior.
**Benefit:** Enables clean capture of Claude's response for pipelines, variables, and downstream tools.
**Priority:** High

### Acceptance Criteria

- Providing `[MESSAGE]` defaults to print mode (one of several auto-print triggers, alongside non-TTY stdin and `--file`/stdin content — see D11); `-p`/`--print` is not required
- `-p`/`--print` is an explicit alias that works identically (backward compatibility)
- Captured stdout is clean and pipeable: `result=$(clr "task")` and `clr "task" | grep X` work
- `--strip-fences` removes the outermost code fence from stdout when bare code is needed
- Requested print mode (`-p`/`--print`, `CLR_PRINT`, or JSON config) with no message, `--file`, or piped stdin content exits with error code 1 and a clear error message

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`run`](../command/01_run.md) | Default command; message is one of several print-mode triggers |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Claude-Native Flags](../param_group/01_claude_native_flags.md) | `--print` is a Claude-native flag |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`--message`](../param/001_message.md) | Prompt text; presence is one of several print-mode triggers |
| 2 | [`--print`](../param/002_print.md) | Explicit print mode selector (alias) |
| 3 | [`--model`](../param/003_model.md) | Select model for the response |
| 26 | [`--strip-fences`](../param/026_strip_fences.md) | Remove outermost code fence from captured output |

### Workflow Steps

1. `clr "your prompt"` — send a prompt; print mode activates automatically
2. `result=$(clr "task")` — capture the response in a shell variable
3. `clr "task" | grep pattern` — pipe the response to another tool
4. `clr "task" --strip-fences` — strip outermost code fence from captured output
