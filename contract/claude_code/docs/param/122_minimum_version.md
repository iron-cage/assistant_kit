# Parameter: minimum_version

### Forms

| Form | Value |
|------|-------|
| Config Key | `minimumVersion` (user, project, local, and managed `settings.json`) |

### Type

string (semver, e.g. `"2.1.100"`)

### Default

Not set (no floor)

### Description

Floor that prevents background auto-updates and `claude update` from installing a version below this value. Set automatically when a user switches from the `"latest"` to `"stable"` channel via `/config` and chooses to stay on the current version rather than allow the downgrade; switching back to `"latest"` clears it. Can also be set directly in `settings.json`. In managed settings, enforces an organization-wide minimum that user and project settings cannot override. Constrains updates only — it does not block startup. To make Claude Code refuse to start below a version, use `requiredMinimumVersion` instead.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [121_auto_updates_channel.md](121_auto_updates_channel.md) | Release channel selector (triggers this on downgrade) |
| doc | [123_required_minimum_version.md](123_required_minimum_version.md) | Managed-only hard startup floor (vs. this soft update floor) |
| doc | [../pattern/001_version_pinning.md](../pattern/001_version_pinning.md) | Synthesis: full version-pinning landscape |
