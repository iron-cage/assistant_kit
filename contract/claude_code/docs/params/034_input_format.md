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

### Since

pre-v1.0 (unverified)

### Description

Specifies the encoding of input read from stdin (print mode only). `text` reads the prompt as plain text. `stream-json` accepts a stream of newline-delimited JSON message objects, enabling multi-turn structured input. Use with `--replay-user-messages` for full bidirectional JSON streaming. Has no effect outside print mode.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [044_output_format.md](044_output_format.md) | Output encoding (complement) |
| doc | [054_replay_user_messages.md](054_replay_user_messages.md) | Replay user messages (used with stream-json) |
| doc | [051_print.md](051_print.md) | Print mode (required for this flag) |