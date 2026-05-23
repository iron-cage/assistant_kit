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
cm .settings.get key::theme
cm .settings.get key::api.endpoint   # dot is literal
```

### Referenced Parameters

| # | Parameter |
|---|-----------|
| 1 | [`key::`](../param/06_key.md) |
