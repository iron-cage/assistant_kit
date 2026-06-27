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

### Since

pre-v1.0 (unverified)

### Description

Controls the encoding of Claude's response on stdout (print mode only). `text` is plain text. `json` emits a single JSON object on completion. `stream-json` emits a stream of newline-delimited JSON objects as chunks arrive. Use `stream-json` with `--include-partial-messages` for real-time streaming. Has no effect outside print mode.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [051_print.md](051_print.md) | Print mode (required for this flag) |
| doc | [033_include_partial_messages.md](033_include_partial_messages.md) | Streaming partial messages (used with stream-json) |
| doc | [045_output_style.md](045_output_style.md) | Visual rendering style (distinct from this) |
| doc | [../formats/007_json_response.md](../formats/007_json_response.md) | JSON response schema (produced when `--output-format json`) |