# User Story :: 013. Structured JSON Pipeline

### Persona

Developer building an automated pipeline that needs structured JSON output from Claude, ready for downstream tools like `jq` or direct deserialization.

### Goal

Generate schema-constrained JSON output from Claude and deliver bare JSON to stdout — no fence delimiters — for immediate use in downstream processing.

### Acceptance Criteria

- `--json-schema <schema>` passes a JSON Schema to constrain Claude's output format
- `--strip-fences` combined with `--json-schema` delivers bare JSON to stdout
- Output can be piped directly to `jq`, written to a file, or deserialized by a consumer
- Schema can be passed inline as a string or via shell substitution from a file (`$(cat schema.json)`)
- `--file` can supply input data for schema-based extraction

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| 1 | [`run`](../command.md#command--1-run) | Both `--json-schema` and `--strip-fences` apply to `run` |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`--json-schema`](../param/23_json_schema.md) | JSON Schema for structured output constraint |
| 2 | [`--strip-fences`](../param/26_strip_fences.md) | Remove fence wrapping from JSON output |
| 3 | [`--file`](../param/25_file.md) | Optional: supply input data for extraction |
| 4 | [`--print`](../param/02_print.md) | Print mode (required for stdout capture) |

### Related User Stories

| # | User Story | Relationship |
|---|-----------|-------------|
| 1 | [012 Code Block Extraction](012_code_block_extraction.md) | Shares `--strip-fences` mechanics |
| 2 | [011 File Input](011_file_input.md) | Input data often comes from a file |
