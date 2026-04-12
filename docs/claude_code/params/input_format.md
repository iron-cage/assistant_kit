# input_format

Specifies the encoding of input read from stdin in print mode.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--input-format <fmt>` |
| Env Var | — |
| Config Key | — |

### Type

enum — `text` `stream-json`

### Default

`text`

### Description

Specifies the encoding of input read from stdin (print mode only). `text` reads the prompt as plain text. `stream-json` accepts a stream of newline-delimited JSON message objects, enabling multi-turn structured input. Use with `--replay-user-messages` for full bidirectional JSON streaming. Has no effect outside print mode.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |