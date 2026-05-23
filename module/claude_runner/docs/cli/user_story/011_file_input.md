# User Story :: 011. File Input

### Persona

Developer who wants to feed a file's content to Claude as stdin alongside a prompt, without constructing a shell pipeline.

### Goal

Pipe a file's content as stdin to the claude subprocess using a single `clr` invocation.

### Acceptance Criteria

- `--file <path>` opens the file and pipes its bytes as subprocess stdin
- Functionally equivalent to `cat file | clr -p "message"` without requiring a shell pipeline
- Non-readable path causes `clr` to exit with an error message including the path and OS error
- Path is resolved relative to the caller's working directory (after any `--dir` change)
- `--file` is distinct from `--json-schema`: `--file` feeds raw bytes to stdin; `--json-schema` sets structured-output constraints

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| 1 | [`run`](../001_command.md#command--1-run) | `--file` applies to the `run` subcommand |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`--file`](../param/025_file.md) | Path to file piped as subprocess stdin |
| 2 | [`[MESSAGE]`](../param/001_message.md) | Prompt sent alongside the file content |
| 3 | [`--print`](../param/002_print.md) | Print mode (typically used with file input) |
| 4 | [`--strip-fences`](../param/026_strip_fences.md) | Strip output fences after file-driven generation |

### Related User Stories

| # | User Story | Relationship |
|---|-----------|-------------|
| 1 | [012 Code Block Extraction](012_code_block_extraction.md) | Commonly combined with `--file` to extract code |
| 2 | [013 Structured JSON Pipeline](013_structured_json_pipeline.md) | File input drives JSON extraction pipelines |
