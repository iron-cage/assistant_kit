# User Story :: 012. Code Block Extraction

### Persona

Developer or automation script consuming Claude's code output in a downstream tool that expects raw source code without markdown fence delimiters.

### Goal

Strip the outermost markdown code fence from captured stdout so the bare code is ready for piping to a compiler, linter, or file.

### Acceptance Criteria

- `--strip-fences` removes the first and last `` ``` `` lines (with optional language tag) from stdout
- All content between the fence lines is emitted unchanged, including any interior fence pairs
- If no fence pair is found, stdout passes through unmodified (no-op behavior)
- Stripping occurs after subprocess completion; has no effect in `--dry-run` mode
- Works in combination with `--file` for schema-to-code or diff-to-patch workflows

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| 1 | [`run`](../001_command.md#command--1-run) | `--strip-fences` post-processes `run` stdout |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`--strip-fences`](../param/026_strip_fences.md) | Remove outermost code fence from stdout |
| 2 | [`--print`](../param/002_print.md) | Print mode captures output for fence stripping |
| 3 | [`--file`](../param/025_file.md) | Commonly used with file-driven code generation |

### Related User Stories

| # | User Story | Relationship |
|---|-----------|-------------|
| 1 | [011 File Input](011_file_input.md) | Commonly combined: file → generate → strip |
| 2 | [013 Structured JSON Pipeline](013_structured_json_pipeline.md) | `--strip-fences` also used for JSON extraction |
