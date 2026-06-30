# Type :: 4. `SettingsKey`

-- **Summary:** Name of a settings entry in ~/.claude/settings.json.
-- **Base Type:** String
-- **Constraints:** non-empty; any UTF-8 string
-- **Default:** (required)
-- **Used By:** `key::`

Stored as a literal JSON object key. Dot characters are literal, not path
separators.

- **Base type:** String
- **Constraints:** non-empty; any UTF-8 string
- **Validation:** `"key:: is required"` if missing; `"key:: value cannot be empty"` if empty

```sh
clv .settings.get key::theme
clv .settings.get key::api.endpoint   # dot is literal
```

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|--------------|
| 1 | [`.settings.get`](../command/settings.md#command--10-settingsget) | `key::` |
| 2 | [`.settings.set`](../command/settings.md#command--11-settingsset) | `key::` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|---------|
| 1 | [`key::`](../param/06_key.md) | 2 |
