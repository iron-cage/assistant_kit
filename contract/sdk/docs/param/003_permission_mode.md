# permissionMode

Session-wide permission posture governing whether tool calls are auto-approved, prompted, or denied.

### Forms

| | Value |
|-|-------|
| TS Field | `permissionMode?: PermissionMode` |
| Python Field | `permission_mode` (confirmed via official hooks example: `permission_mode="acceptEdits"`) |
| CLI Equivalent | `--permission-mode` — [`../../../claude_code/docs/param/046_permission_mode.md`](../../../claude_code/docs/param/046_permission_mode.md) (narrower enum; see [S6](../behavior/006_s6_permission_modes_richer_than_cli.md)) |

### Type

enum — `"default"` `"acceptEdits"` `"bypassPermissions"` `"plan"` `"dontAsk"` `"auto"`

### Default

`'default'`

### Since

SDK GA

### Description

Sets the session's default tool-approval behavior. Also settable live, mid-session, via `Query.setPermissionMode()` (see [`../api/004_query_control_object.md`](../api/004_query_control_object.md)) — a capability with no CLI equivalent since the CLI flag is argv-time-fixed. Can be overridden on a per-tool-call basis by `canUseTool` (see [`013_can_use_tool.md`](013_can_use_tool.md)), which fires before this session-wide default is applied. See [S6](../behavior/006_s6_permission_modes_richer_than_cli.md) for the full 6-value semantics and the comparison against the CLI's narrower surface.

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| doc | [readme.md](readme.md) | Master curated parameter table |
| doc | [013_can_use_tool.md](013_can_use_tool.md) | Per-call override that takes precedence over this session-wide setting |
| doc | [../api/004_query_control_object.md](../api/004_query_control_object.md) | `setPermissionMode()` live control method |
| behavior | [../behavior/006_s6_permission_modes_richer_than_cli.md](../behavior/006_s6_permission_modes_richer_than_cli.md) | Full enum semantics and CLI comparison |
| doc | [../../../claude_code/docs/param/046_permission_mode.md](../../../claude_code/docs/param/046_permission_mode.md) | CLI-level equivalent flag |
