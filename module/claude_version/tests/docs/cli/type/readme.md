# Type Tests

### Scope

- **Purpose**: Type validation test cases for all 9 clv semantic types.
- **Responsibility**: Index of per-type test case files covering parsing, validation, and boundary behavior.
- **In Scope**: All 9 clv types: VerbosityLevel, OutputFormat, VersionSpec, SettingsKey, SettingsValue, ConfigScope, ConfigKey, ParamKind, PathKey.
- **Out of Scope**: Command integration (→ `../command/`), parameter edge cases (→ `../param/`).

### Overview Table

| Name | Purpose | Status |
|------|---------|--------|
| 01_verbosity_level.md | Type validation tests for `VerbosityLevel` (u8 0–2) | ✅ |
| 02_output_format.md | Type validation tests for `OutputFormat` (text\|json) | ✅ |
| 03_version_spec.md | Type validation tests for `VersionSpec` (aliases + semver) | ✅ |
| 04_settings_key.md | Type validation tests for `SettingsKey` (non-empty string) | ✅ |
| 05_settings_value.md | Type validation tests for `SettingsValue` (type-inferred string) | ✅ |
| 06_config_scope.md | Type validation tests for `ConfigScope` (user\|project enum) | ✅ |
| 07_config_key.md | Type validation tests for `ConfigKey` (non-empty string + catalog) | ✅ |
| 08_param_kind.md | Type validation tests for `ParamKind` (config\|env enum) | ✅ |
| 09_path_key.md | Type validation tests for `PathKey` (5-variant enum) | ✅ |
| procedure.md | Workflow for creating and updating type test specs | ✅ |

### Navigation

- [VerbosityLevel](01_verbosity_level.md) — u8 range 0–2
- [OutputFormat](02_output_format.md) — enum text|json
- [VersionSpec](03_version_spec.md) — named aliases + semver
- [SettingsKey](04_settings_key.md) — non-empty string key
- [SettingsValue](05_settings_value.md) — type-inferred string value
- [ConfigScope](06_config_scope.md) — user|project enum
- [ConfigKey](07_config_key.md) — non-empty string + catalog awareness
- [ParamKind](08_param_kind.md) — config|env enum
- [PathKey](09_path_key.md) — 5-variant path key enum

### See Also

- [Parameter Tests](../param/) — parameter edge cases that exercise these types
- [Source types](../../../../docs/cli/type/) — authoritative type definitions
