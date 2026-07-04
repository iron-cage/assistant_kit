# Parameter: required_maximum_version

### Forms

| Form | Value |
|------|-------|
| Config Key | `requiredMaximumVersion` (managed `settings.json` only) |

### Type

string (semver)

### Default

Not set (no ceiling)

### Description

Managed-settings-only ceiling. Maximum Claude Code version allowed to start; if the running binary is newer, Claude Code exits at startup and instructs the user to install an approved version through the organization's approved method (`claude install <version>` may also work). Background auto-updates and `claude update` also respect this ceiling. `claude update`, `claude install`, and `claude doctor` keep working above the ceiling so users can recover, and versions that predate this setting ignore it. If set to an invalid value, fails open: the value is stripped rather than enforced, so a malformed policy push cannot itself block Claude Code from starting. Cannot be set from user or project `settings.json` — organization administrators only.

### Since

v2.1.163

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [123_required_minimum_version.md](123_required_minimum_version.md) | Companion startup floor |
| doc | [../pattern/001_version_pinning.md](../pattern/001_version_pinning.md) | Synthesis: full version-pinning landscape |
