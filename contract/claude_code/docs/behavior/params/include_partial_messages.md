# include_partial_messages

Emits partial response chunks as they arrive rather than waiting for the complete response.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--include-partial-messages` |
| Env Var | — |
| Config Key | — |

### Type

bool

### Default

`off`

### Description

Emits partial response chunks as they arrive rather than waiting for the complete response. Requires `--output-format=stream-json` and print mode. Enables real-time streaming to downstream consumers. Without this flag, stream-json still streams but only emits complete message objects.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |