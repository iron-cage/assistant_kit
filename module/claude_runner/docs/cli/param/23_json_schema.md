# Parameter :: 23. `--json-schema`

JSON Schema for structured output validation. Forwarded directly to the
`claude` subprocess as `--json-schema <schema>`. When present, the subprocess
validates its response against the provided schema and returns structured JSON.

- **Type:** [`JsonSchemaText`](../type.md#type--10-jsonschematext)
- **Default:** — (unset; no structured output constraint)
- **Command:** [`run`](../command.md#command--1-run)
- **Group:** [Claude-Native Flags](../param_group.md#group--1-claude-native-flags)

```sh
clr --json-schema '{"type":"object","properties":{"name":{"type":"string"}}}' "Get user"
clr --json-schema "$(cat schema.json)" "List failing tests"
```

**Note:** The value must be a valid JSON object string. The subprocess will
return a JSON-encoded response matching the schema shape, not free-form text.

**Note:** Combine with `--output-format json` (when implemented) for fully
machine-readable output pipelines.
