# Type :: 7. `ConfigKey`

-- **Summary:** Name of a settings key — either a known catalog key or an arbitrary user-defined key.
-- **Base Type:** String
-- **Constraints:** non-empty UTF-8 string
-- **Default:** (required when used)
-- **Used By:** `key::` (in `.config` context)

A `ConfigKey` is any non-empty UTF-8 string. Known keys have registered defaults and env var mappings in the config catalog. Unknown keys are accepted without error — they have no catalog default and no env mapping.

- **Base type:** String
- **Constraints:** non-empty; any UTF-8 string; dot characters are literal, not path separators
- **Validation:** missing or empty → exit 1

**Known catalog keys:**

| Key | Env var | Default |
|-----|---------|---------|
| `model` | `CLAUDE_MODEL` | `claude-sonnet-4-6` |
| `preferredVersionSpec` | — | `stable` |
| `preferredVersionResolved` | — | (absent) |
| `autoUpdates` | — | `true` |
| `theme` | — | `system` |
| `hasCompletedOnboarding` | — | `false` |
| `env.DISABLE_AUTOUPDATER` | — | (absent) |

```sh
clv .config key::model              # catalog key — resolves env + default
clv .config key::theme              # catalog key — resolves user config + default
clv .config key::myCustomSetting    # arbitrary key — resolves user/project config only
```

### Referenced Parameters

| # | Parameter |
|---|-----------|
| 1 | [`key::`](../param/06_key.md) |

### See Also

| Type | File |
|------|------|
| `SettingsKey` | [04_settings_key.md](04_settings_key.md) — key type for deprecated `.settings.*` commands |
