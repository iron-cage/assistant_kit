# CLI User Story: Code Block Extraction

### Scope

- **Purpose**: Document stripping outermost markdown fences from captured stdout using --strip-fences.
- **Responsibility**: Define acceptance criteria for fence stripping behavior including no-op case.
- **In Scope**: Fence removal, content preservation, no-op on missing fence pair, interaction with --file.
- **Out of Scope**: JSON structured output (→ 013_structured_json_pipeline.md).

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

| # | Command | Role |
|---|---------|------|
| 1 | [`run`](../command/01_run.md) | Default command; `--strip-fences` removes code fences |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Claude-Native Flags](../param_group/01_claude_native_flags.md) | `--print` captures stdout for fence stripping |
| 2 | [Runner Control](../param_group/02_runner_control.md) | `--strip-fences` is a runner control flag |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 2 | [`--print`](../param/002_print.md) | Print mode captures output for fence stripping |
| 25 | [`--file`](../param/025_file.md) | Commonly used with file-driven code generation |
| 26 | [`--strip-fences`](../param/026_strip_fences.md) | Remove outermost code fence from stdout |

### Related User Stories

| # | User Story | Relationship |
|---|------------|--------------|
| 11 | [File Input](011_file_input.md) | Commonly combined: file → generate → strip |
| 13 | [Structured JSON Pipeline](013_structured_json_pipeline.md) | `--strip-fences` also used for JSON extraction |
