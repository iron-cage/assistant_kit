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

### Since

pre-v1.0 (unverified)

### Description

Emits partial response chunks as they arrive rather than waiting for the complete response. Requires `--output-format=stream-json` and print mode. Enables real-time streaming to downstream consumers. Without this flag, stream-json still streams but only emits complete message objects.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [044_output_format.md](044_output_format.md) | Output format (stream-json required for this flag) |
| doc | [051_print.md](051_print.md) | Print mode (required for this flag) |