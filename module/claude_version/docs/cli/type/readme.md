# Types

### Scope

- **Purpose**: Semantic type definitions for clv parameter values.
- **Responsibility**: Type name, base type, format rules, and validation constraints.
- **In Scope**: semantic types used by clv parameters.
- **Out of Scope**: Parameter reference (→ `../param/`), type inference algorithm (→ `../../algorithm/`).

### Responsibility Table

| File | Responsibility |
|------|---------------|
| readme.md | Index and navigation for type files |
| procedure.md | Steps for adding, updating, or removing type instances |
| 01_verbosity_level.md | `VerbosityLevel` — u8 0–2, output detail control |
| 02_output_format.md | `OutputFormat` — enum text\|json, display format |
| 03_version_spec.md | `VersionSpec` — String, release target identifier |
| 04_settings_key.md | `SettingsKey` — String, settings entry name |
| 05_settings_value.md | `SettingsValue` — String, settings entry value with type inference |
| 06_config_scope.md | `ConfigScope` — enum user\|project, write target for `.config` |
| 07_config_key.md | `ConfigKey` — String with catalog context; known + arbitrary keys |
| 08_param_kind.md | `ParamKind` — enum config\|env, kind filter for `.params` show-all |

### All Types

| # | Type | Base | Used By | Purpose |
|---|------|------|---------|---------|
| 1 | [`VerbosityLevel`](01_verbosity_level.md) | u8 (0–2) | [`v::`](../param/04_v.md) | Output detail control |
| 2 | [`OutputFormat`](02_output_format.md) | enum | [`format::`](../param/05_format.md) | Display format selection |
| 3 | [`VersionSpec`](03_version_spec.md) | String | [`version::`](../param/01_version.md) | Release target identifier |
| 4 | [`SettingsKey`](04_settings_key.md) | String | [`key::`](../param/06_key.md) | Settings entry name (deprecated commands) |
| 5 | [`SettingsValue`](05_settings_value.md) | String | [`value::`](../param/07_value.md) | Settings entry value (type-inferred) |
| 6 | [`ConfigScope`](06_config_scope.md) | String enum | [`scope::`](../param/11_scope.md) | Write target: user or project |
| 7 | [`ConfigKey`](07_config_key.md) | String | [`key::`](../param/06_key.md) | Config key with catalog context |
| 8 | [`ParamKind`](08_param_kind.md) | enum | [`kind::`](../param/13_kind.md) | Param kind filter for `.params` show-all |

### See Also

- [Parameters](../param/readme.md) — parameter reference
- [Commands](../command/readme.md) — command reference
