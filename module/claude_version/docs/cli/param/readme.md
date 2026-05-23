# param/ — cm Parameter Reference

### Scope

- **Purpose**: Per-parameter reference for all 10 cm parameters.
- **Responsibility**: Parameter type, default, validation, and cross-references.
- **In Scope**: All 10 cm parameters.
- **Out of Scope**: Command reference (→ `../command/`), type definitions (→ `../type/`), parameter interactions (→ `../004_parameter_interactions.md`).

### Responsibility Table

| File | Responsibility |
|------|---------------|
| readme.md | Index and navigation for parameter files |
| procedure.md | Steps for adding, updating, or removing parameter instances |
| 01_version.md | `version::` — version target for install and guard |
| 02_dry.md | `dry::` — preview without executing |
| 03_force.md | `force::` — bypass safety guards |
| 04_v.md | `v::` — output verbosity level |
| 05_format.md | `format::` — output format selection |
| 06_key.md | `key::` — settings entry name (required) |
| 07_value.md | `value::` — settings entry value (required) |
| 08_interval.md | `interval::` — guard check interval in seconds |
| 09_count.md | `count::` — limit number of history entries |
| 10_help.md | `.help` — universal help override |

### All Parameters (10 total)

| # | Parameter | Type | Default | Groups | Used In |
|---|-----------|------|---------|--------|---------|
| 1 | [`version::`](01_version.md) | `VersionSpec` | stable | — | 2 cmds |
| 2 | [`dry::`](02_dry.md) | bool | false | Execution Control | 4 cmds |
| 3 | [`force::`](03_force.md) | bool | false | Execution Control | 3 cmds |
| 4 | [`v::`](04_v.md) | `VerbosityLevel` | 1 | Output Control | 10 cmds |
| 5 | [`format::`](05_format.md) | `OutputFormat` | text | Output Control | 10 cmds |
| 6 | [`key::`](06_key.md) | `SettingsKey` | **(required)** | Settings Identity | 2 cmds |
| 7 | [`value::`](07_value.md) | `SettingsValue` | **(required)** | Settings Identity | 1 cmd |
| 8 | [`interval::`](08_interval.md) | u64 | 0 | — | 1 cmd |
| 9 | [`count::`](09_count.md) | u64 | 10 | Output Control | 1 cmd |
| 10 | [`.help`](10_help.md) | bool | false | — | 12 cmds |

### See Also

- [Commands](../command/readme.md) — command reference
- [Types](../type/readme.md) — type definitions
- [Parameter Groups](../param_group/readme.md) — group membership
