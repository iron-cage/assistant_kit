# log_level

Controls the minimum severity level of log messages emitted by Claude Code's internal logger.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_LOG_LEVEL` |
| Config Key | — |

### Type

enum — `Error` `Warn` `Info` `Debug` `Trace`

### Default

`Info`

### Since

pre-v1.0 (unverified)

### Description

Controls the minimum severity level of log messages emitted by Claude Code's internal logger. `Error` shows only errors; `Trace` shows everything including fine-grained internal events. `Info` is the standard operational level. Increase to `Debug` or `Trace` when diagnosing unexpected behaviour without full `--debug` mode overhead.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |