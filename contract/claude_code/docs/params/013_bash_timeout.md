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

### Since

pre-v1.0 (unverified)

### Description

Sets the default timeout for each bash command Claude executes. If a bash command runs longer than this value, Claude Code terminates it and returns a timeout error. The binary default is 2 minutes, suited for interactive use. Automation workflows running `cargo test`, `docker build`, or similar long commands should increase this accordingly.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [012_bash_max_timeout.md](012_bash_max_timeout.md) | `CLAUDE_CODE_BASH_MAX_TIMEOUT` — ceiling that caps this value |
| doc | [096_bash_default_timeout_ms.md](096_bash_default_timeout_ms.md) | `BASH_DEFAULT_TIMEOUT_MS` — binary-level default timeout equivalent |
| doc | [../tool/004_bash.md](../tool/004_bash.md) | Bash tool this configures |