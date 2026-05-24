# CLI Type: JsonSchemaText

JSON Schema document passed as a string to `--json-schema`. Must be a valid
JSON object conforming to JSON Schema specification (draft-07 or later).

- **Purpose:** JSON Schema object string for structured output
- **Fundamental Type:** String
- **Constants:** —
- **Constraints:** must parse as valid JSON; must be a JSON object (`{…}`)
- **Parsing:** consumed as the next token after `--json-schema`
- **Methods:** —

```sh
clr --json-schema '{"type":"object","properties":{"n":{"type":"string"}}}' "task"
clr --json-schema "$(cat schema.json)" "task"
```

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|---------------|
| 1 | [`run`](../command/01_run.md) | `--json-schema` |
| 5 | [`ask`](../command/05_ask.md) | `--json-schema` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 23 | [`--json-schema`](../param/023_json_schema.md) | 2 |
