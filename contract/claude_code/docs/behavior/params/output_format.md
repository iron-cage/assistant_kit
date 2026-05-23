# output_format

Controls the encoding of Claude's response on stdout in print mode.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--output-format <fmt>` |
| Env Var | — |
| Config Key | — |

### Type

enum — `text` `json` `stream-json`

### Default

`text`

### Description

Controls the encoding of Claude's response on stdout (print mode only). `text` is plain text. `json` emits a single JSON object on completion. `stream-json` emits a stream of newline-delimited JSON objects as chunks arrive. Use `stream-json` with `--include-partial-messages` for real-time streaming. Has no effect outside print mode.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |