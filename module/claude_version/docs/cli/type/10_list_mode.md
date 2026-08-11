# Type :: 10. `ListMode`

-- **Summary:** Select alias listing or release-history listing in `.version.list`.
-- **Base Type:** enum (2 variants)
-- **Valid Values:** `aliases`, `history`
-- **Default:** `aliases`
-- **Used By:** `mode::`

Case-sensitive matching. `Aliases`, `HISTORY`, `History` are all rejected.

- **Base type:** enum (2 variants)
- **Valid values:** `aliases`, `history`
- **Default:** `aliases` (local compile-time alias table; no network)
- **Parsing:** exact string match; case variants rejected
- **Validation errors:** `"unknown mode '{raw}': expected aliases or history"`

```sh
clv.version.list mode::aliases     # compile-time alias table (default)
clv.version.list mode::history     # GitHub release history (network, cached 1h)
clv.version.list mode::Aliases     # error: case-sensitive
```

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|--------------|
| 1 | [`.version.list`](../command/version.md#command-6-versionlist) | `mode::` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|---------|
| 1 | [`mode::`](../param/14_mode.md) | 1 |
