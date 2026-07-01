# Generate schema-constrained JSON output for downstream processing

**Persona:** Developer building an automated pipeline that needs structured JSON output from Claude, ready for downstream tools like `jq` or direct deserialization.
**Goal:** Generate schema-constrained JSON output from Claude and deliver bare JSON to stdout — no fence delimiters — for immediate use in downstream processing.
**Benefit:** Enables schema-driven automation by delivering validated, structured JSON directly to downstream tools.
**Priority:** Medium

### Acceptance Criteria

- `--json-schema <schema>` passes a JSON Schema to constrain Claude's output format
- `--strip-fences` combined with `--json-schema` delivers bare JSON to stdout
- Output can be piped directly to `jq`, written to a file, or deserialized by a consumer
- Schema can be passed inline as a string or via shell substitution from a file (`$(cat schema.json)`)
- `--file` can supply input data for schema-based extraction

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`run`](../command/01_run.md) | Default command; `--json-schema` constrains output |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Claude-Native Flags](../param_group/01_claude_native_flags.md) | `--json-schema` is a Claude-native flag |
| 2 | [Runner Control](../param_group/02_runner_control.md) | `--strip-fences` post-processes output |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 2 | [`--print`](../param/002_print.md) | Print mode (required for stdout capture) |
| 23 | [`--json-schema`](../param/023_json_schema.md) | JSON Schema for structured output constraint |
| 25 | [`--file`](../param/025_file.md) | Optional: supply input data for extraction |
| 26 | [`--strip-fences`](../param/026_strip_fences.md) | Remove fence wrapping from JSON output |

### Workflow Steps

1. `clr -p "Extract metadata" --json-schema '{"type":"object","properties":{"name":{"type":"string"}}}' --strip-fences` — generate bare JSON constrained by schema
2. `clr -p "Extract fields" --json-schema "$(cat schema.json)" --strip-fences | jq .` — pipe schema from file and process output with jq
3. `clr -p "Extract from this" --file data.txt --json-schema "$(cat schema.json)" --strip-fences` — combine file input with schema constraint

### Related User Stories

| # | User Story | Relationship |
|---|------------|--------------|
| 11 | [File Input](011_file_input.md) | Input data often comes from a file |
| 12 | [Code Block Extraction](012_code_block_extraction.md) | Shares `--strip-fences` mechanics |
