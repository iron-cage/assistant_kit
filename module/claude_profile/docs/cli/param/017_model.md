# Parameter :: 17. `model::`

Controls whether the active model line appears in output. Opt-in (default `0`). Source: `model` field in `settings.json` — read from live `~/.claude/settings.json` for `.credentials.status`. For `.accounts`, read from `{name}.settings.json` per-account snapshot (captured by `save()` — BUG-222 fix); shows `N/A` when snapshot absent or `model` field missing.

- **Type:** `bool`
- **Default:** `0` (hidden)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.accounts`](../command/001_account.md#command--3-accounts), [`.credentials.status`](../command/002_credentials.md#command--10-credentialsstatus)
- **Purpose:** Shows the model currently selected in Claude Code settings. Shows `N/A` when the source file is absent or the `model` field is missing.
- **Group:** [Field Presence](../param_group/002_field_presence.md)

**Examples:**

```text
model::0   → line omitted  (default)
model::1   → Model:   sonnet
```
