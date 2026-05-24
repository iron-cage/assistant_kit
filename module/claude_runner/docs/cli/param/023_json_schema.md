# CLI Parameter: --json-schema

JSON Schema for structured output validation. Forwarded directly to the
`claude` subprocess as `--json-schema <schema>`. When present, the subprocess
validates its response against the provided schema and returns structured JSON.

- **Type:** [`JsonSchemaText`](../type/10_json_schema_text.md)
- **Default:** — (unset; no structured output constraint)
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Claude-Native Flags](../param_group/01_claude_native_flags.md)

```sh
clr --json-schema '{"type":"object","properties":{"name":{"type":"string"}}}' "Get user"
clr --json-schema "$(cat schema.json)" "List failing tests"
```

**Note:** The value must be a valid JSON object string. The subprocess will
return a JSON-encoded response matching the schema shape, not free-form text.

**Note:** Combine with `--output-format json` (when implemented) for fully
machine-readable output pipelines.

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`JsonSchemaText`](../type/10_json_schema_text.md) | Semantic | String | valid JSON object |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 1 | [Claude-Native Flags](../param_group/01_claude_native_flags.md) | Full | `--print`, `--model`, `--verbose`, `--effort`, `--mcp-config` |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | — | — |
| 5 | [`ask`](../command/05_ask.md) | — | — |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 13 | [013_structured_json_pipeline.md](../user_story/013_structured_json_pipeline.md) | Developer |
