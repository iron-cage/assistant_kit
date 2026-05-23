# Parameter :: 1. `version::`

-- **Summary:** Specify which Claude Code version to install or guard against.
-- **Type:** `VersionSpec`
-- **Default:** `stable`
-- **Commands:** `.version.install`, `.version.guard`
-- **Group:** none

Accepts named aliases (`stable`, `latest`, `month`) or semver strings (e.g., `1.2.3`).
On `.version.guard`, the value overrides the stored preference for a single invocation
without writing to `settings.json`.

- **Type:** [`VersionSpec`](../type/03_version_spec.md)
- **Default:** `stable`
- **Validation:** rejects 4-part semver (e.g., `1.2.3.4`), leading zeros (e.g., `01.02.03`), empty value

```sh
cm .version.install version::stable
cm .version.install version::1.2.3
cm .version.guard version::stable dry::1
cm .version.guard version::month
```

### Referenced Commands

| # | Command |
|---|---------|
| 1 | [`.version.install`](../command/version.md#command--4-versioninstall) |
| 2 | [`.version.guard`](../command/version.md#command--5-versionguard) |

### Referenced Types

| # | Type |
|---|------|
| 1 | [`VersionSpec`](../type/03_version_spec.md) |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [002 Version Upgrade](../user_story/002_version_upgrade.md) | Developer (version upgrade) |
| 2 | [005 Version Pinning](../user_story/005_version_pinning.md) | Team lead (version pinning) |
