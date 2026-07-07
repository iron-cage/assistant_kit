# Type :: 9. `PathKey`

-- **Summary:** Select which runtime path to report in `.paths` single-path mode.
-- **Base Type:** enum (5 variants)
-- **Valid Values:** `settings`, `project_settings`, `versions_dir`, `binary_symlink`, `version_history_cache`
-- **Default:** absent (no filter — show all paths)
-- **Used By:** `key::`

Case-sensitive matching. Mixed-case or unknown variants are rejected.

- **Base type:** enum (5 variants)
- **Valid values:** `settings`, `project_settings`, `versions_dir`, `binary_symlink`, `version_history_cache`
- **Default:** absent (no filter — all known paths shown)
- **Parsing:** exact string match; case variants rejected
- **Validation errors:** `"unknown key '{raw}': expected one of settings, project_settings, versions_dir, binary_symlink, version_history_cache"`

```sh
clv.paths key::settings              # single path: settings.json
clv.paths key::versions_dir          # single path: versions directory
clv.paths key::Settings              # error: case-sensitive
```

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|--------------|
| 1 | [`.paths`](../command/paths.md#command--16-paths) | `key::` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|---------|
| 1 | [`key::`](../param/06_key.md) | 5 |
