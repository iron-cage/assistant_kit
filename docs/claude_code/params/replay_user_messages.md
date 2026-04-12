# replay_user_messages

Re-emits user messages received on stdin back on stdout for acknowledgment.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--replay-user-messages` |
| Env Var | — |
| Config Key | — |

### Type

bool

### Default

`off`

### Description

Re-emits user messages received on stdin back on stdout for acknowledgment. Requires both `--input-format=stream-json` and `--output-format=stream-json`. Useful for bidirectional streaming pipelines where the consumer needs to verify that Claude received each user message before processing the response.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |