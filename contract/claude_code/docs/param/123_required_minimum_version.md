# Parameter: required_minimum_version

### Forms

| Form | Value |
|------|-------|
| Config Key | `requiredMinimumVersion` (managed `settings.json` only) |

### Type

string (semver)

### Default

Not set (no floor)

### Description

Managed-settings-only floor. Minimum Claude Code version required to start; if the running binary is older, Claude Code exits at startup and instructs the user to update through the organization's approved method. Differs from `minimumVersion`, which prevents downgrades but never blocks startup. `claude update`, `claude install`, and `claude doctor` keep working below the floor so users can recover, and versions that predate this setting ignore it. If set to an invalid value, fails open: the value is stripped rather than enforced, so a malformed policy push cannot itself block Claude Code from starting. Cannot be set from user or project `settings.json` — organization administrators only.

### Since

v2.1.163

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [122_minimum_version.md](122_minimum_version.md) | Soft update-only floor (vs. this hard startup floor) |
| doc | [124_required_maximum_version.md](124_required_maximum_version.md) | Companion startup ceiling |
| doc | [../pattern/001_version_pinning.md](../pattern/001_version_pinning.md) | Synthesis: full version-pinning landscape |
