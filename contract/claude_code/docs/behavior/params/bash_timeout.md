# bash_timeout

Sets the default timeout for each bash command Claude executes.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_BASH_TIMEOUT` |
| Config Key | — |

### Type

integer (milliseconds)

### Default

`120 000` (2 minutes)

### Description

Sets the default timeout for each bash command Claude executes. If a bash command runs longer than this value, Claude Code terminates it and returns a timeout error. The binary default is 2 minutes, suited for interactive use. Automation workflows running `cargo test`, `docker build`, or similar long commands should increase this accordingly.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |