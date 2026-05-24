# Type Tests

### Scope

- **Purpose**: Type validation test cases for all 5 cm semantic types.
- **Responsibility**: Index of per-type test case files covering parsing, validation, and boundary behavior.
- **In Scope**: All 5 cm types: VerbosityLevel, OutputFormat, VersionSpec, SettingsKey, SettingsValue.
- **Out of Scope**: Command integration (→ `../command/`), parameter edge cases (→ `../param/`).

### Overview Table

| Name | Purpose | Status |
|------|---------|--------|
| 001_verbosity_level.md | Type validation tests for `VerbosityLevel` (u8 0–2) | ⏳ |
| 002_output_format.md | Type validation tests for `OutputFormat` (text\|json) | ⏳ |
| 003_version_spec.md | Type validation tests for `VersionSpec` (aliases + semver) | ⏳ |
| 004_settings_key.md | Type validation tests for `SettingsKey` (non-empty string) | ⏳ |
| 005_settings_value.md | Type validation tests for `SettingsValue` (type-inferred string) | ⏳ |
| procedure.md | Workflow for creating and updating type test specs | ✅ |

### Navigation

- [VerbosityLevel](001_verbosity_level.md) — u8 range 0–2
- [OutputFormat](002_output_format.md) — enum text|json
- [VersionSpec](003_version_spec.md) — named aliases + semver
- [SettingsKey](004_settings_key.md) — non-empty string key
- [SettingsValue](005_settings_value.md) — type-inferred string value

### See Also

- [Parameter Tests](../param/) — parameter edge cases that exercise these types
- [Source types](../../../../docs/cli/type/) — authoritative type definitions
