# param/ — clvParameter Reference

### Scope

- **Purpose**: Per-parameter reference for all clvparameters.
- **Responsibility**: Parameter type, default, validation, and cross-references.
- **In Scope**: All clvparameters.
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
| 11_scope.md | `scope::` — write target: user or project |
| 12_unset.md | `unset::` — delete key from target scope |

### All Parameters

| # | Parameter | Type | Default | Groups | Used In |
|---|-----------|------|---------|--------|---------|
| 1 | [`version::`](01_version.md) | `VersionSpec` | stable | — | `.version.install`, `.version.guard` |
| 2 | [`dry::`](02_dry.md) | bool | false | Execution Control | write/mutation commands |
| 3 | [`force::`](03_force.md) | bool | false | Execution Control | `.version.install`, `.version.guard`, `.processes.kill` |
| 4 | [`v::`](04_v.md) | `VerbosityLevel` | 1 | Output Control | all output-supporting commands |
| 5 | [`format::`](05_format.md) | `OutputFormat` | text | Output Control | all output-supporting commands |
| 6 | [`key::`](06_key.md) | `SettingsKey` | — (opt. in .config) | Settings Identity, Config Identity | `.settings.get`, `.settings.set`, `.config` |
| 7 | [`value::`](07_value.md) | `SettingsValue` | — (opt. in .config) | Settings Identity, Config Identity | `.settings.set`, `.config` |
| 8 | [`interval::`](08_interval.md) | u64 | 0 | — | `.version.guard` |
| 9 | [`count::`](09_count.md) | u64 | 10 | Output Control | `.version.history` |
| 10 | [`.help`](10_help.md) | bool | false | — | all commands |
| 11 | [`scope::`](11_scope.md) | `ConfigScope` | user | Config Identity | `.config` |
| 12 | [`unset::`](12_unset.md) | bool | false | Config Identity | `.config` |

### See Also

- [Commands](../command/readme.md) — command reference
- [Types](../type/readme.md) — type definitions
- [Parameter Groups](../param_group/readme.md) — group membership
