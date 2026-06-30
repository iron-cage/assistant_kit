# Type :: 5. `SettingsValue`

-- **Summary:** Value to write to a settings entry; automatically type-inferred for JSON storage.
-- **Base Type:** String (auto-typed for JSON serialization)
-- **Constraints:** non-empty; any UTF-8 string
-- **Default:** (required)
-- **Used By:** `value::`

- **Base type:** String (auto-typed for JSON serialization)
- **Constraints:** non-empty; any UTF-8 string
- **Validation:** `"value:: is required"` if missing; `"value:: value cannot be empty"` if empty

**Type Inference Rules:**

| Input | Inferred Type | JSON Output |
|-------|---------------|-------------|
| `"true"` / `"false"` | Bool | `true` / `false` |
| Integer string (e.g., `"42"`) | Number (i64) | `42` |
| Finite float string (e.g., `"3.14"`) | Number (f64) | `3.14` |
| `"NaN"`, `"inf"`, `"infinity"` | String | `"NaN"`, `"inf"` |
| Everything else | String | `"value"` |

**Note:** Non-finite floats (`NaN`, `inf`, `infinity` and variants) are
classified as strings because they are not valid JSON number literals.
Special characters (`"`, `\`) in string values are properly escaped.

```sh
clv .settings.set key::autoUpdate value::true    # -> true (bool)
clv .settings.set key::timeout value::30         # -> 30 (number)
clv .settings.set key::theme value::dark         # -> "dark" (string)
clv .settings.set key::rate value::3.14          # -> 3.14 (number)
clv .settings.set key::special value::NaN        # -> "NaN" (string)
```

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|--------------|
| 1 | [`.settings.set`](../command/settings.md#command--11-settingsset) | `value::` |
| 2 | [`.config`](../command/config.md#command--13-config) | `value::` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|---------|
| 1 | [`value::`](../param/07_value.md) | 2 |
