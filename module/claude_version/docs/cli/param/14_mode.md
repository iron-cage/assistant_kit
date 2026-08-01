# Parameter :: 14. `mode::`

-- **Summary:** Select alias listing or release-history listing for `.version.list`.
-- **Type:** `ListMode`
-- **Default:** aliases
-- **Commands:** `.version.list`
-- **Group:** none

When absent or `aliases`, `.version.list` shows the compile-time alias table (no network). When `history`, it fetches recent release history from the GitHub Releases API instead. `count::` only affects output when `mode::history` is set — it is accepted but has no effect under `mode::aliases`.

- **Type:** [`ListMode`](../type/10_list_mode.md)
- **Default:** `aliases`
- **Validation:** must be `aliases` or `history`; any other value → exit 1

```sh
clv.version.list                    # aliases = default
clv.version.list mode::aliases      # same as default, explicit
clv.version.list mode::history      # release history, count::10 default
clv.version.list mode::history count::3
```

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`.version.list`](../command/version.md#command-6-versionlist) | aliases | `count::` is meaningful only under `mode::history` |

### Referenced Type

| # | Type |
|---|------|
| 1 | [`ListMode`](../type/10_list_mode.md) |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [002 Version Upgrade](../user_story/002_version_upgrade.md) | Developer (version upgrade) |
| 2 | [005 Version Pinning](../user_story/005_version_pinning.md) | Team lead (version pinning) |
