# CLI User Story: File Input

### Scope

- **Purpose**: Document piping a file's content as subprocess stdin using --file.
- **Responsibility**: Define acceptance criteria for --file behavior including error handling and path resolution.
- **In Scope**: --file path piping, error on non-readable path, path resolution relative to caller cwd.
- **Out of Scope**: JSON-schema structured output (→ 013_structured_json_pipeline.md), shell-pipeline equivalent.

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

| # | Command | Role |
|---|---------|------|
| 1 | [`run`](../command/01_run.md) | Default command; `--file` pipes content as stdin |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | `--file` is a runner control flag |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`--message`](../param/001_message.md) | Prompt sent alongside the file content |
| 2 | [`--print`](../param/002_print.md) | Print mode (typically used with file input) |
| 25 | [`--file`](../param/025_file.md) | Path to file piped as subprocess stdin |
| 26 | [`--strip-fences`](../param/026_strip_fences.md) | Strip output fences after file-driven generation |

### Related User Stories

| # | User Story | Relationship |
|---|------------|--------------|
| 12 | [Code Block Extraction](012_code_block_extraction.md) | Commonly combined with `--file` to extract code |
| 13 | [Structured JSON Pipeline](013_structured_json_pipeline.md) | File input drives JSON extraction pipelines |
