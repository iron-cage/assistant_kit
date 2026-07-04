# Type :: 2. `OutputFormat`

-- **Summary:** Select between human-readable text and machine-readable JSON output.
-- **Base Type:** enum (2 variants)
-- **Valid Values:** `text`, `json`
-- **Default:** `text`
-- **Used By:** `format::`

Case-sensitive matching. `Text`, `JSON`, `Json` are all rejected.

- **Base type:** enum (2 variants)
- **Valid values:** `text`, `json`
- **Default:** `text`
- **Parsing:** exact string match; `Text`, `JSON`, `Json` all rejected
- **Validation errors:** `"unknown format '{raw}': expected text or json"`

```sh
clv .status format::text       # human-readable
clv .status format::json       # machine-readable
clv .status format::JSON       # error: case-sensitive
```

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|--------------|
| 1 | [`.status`](../command/root.md#command--2-status) | `format::` |
| 2 | [`.version.show`](../command/version.md#command--3-versionshow) | `format::` |
| 3 | [`.version.install`](../command/version.md#command--4-versioninstall) | `format::` |
| 4 | [`.version.guard`](../command/version.md#command--5-versionguard) | `format::` |
| 5 | [`.version.list`](../command/version.md#command--6-versionlist) | `format::` |
| 6 | [`.processes`](../command/processes.md#command--7-processes) | `format::` |
| 7 | [`.processes.kill`](../command/processes.md#command--8-processeskill) | `format::` |
| 8 | [`.settings.show`](../command/settings.md#command--9-settingsshow) | `format::` |
| 9 | [`.settings.get`](../command/settings.md#command--10-settingsget) | `format::` |
| 10 | [`.version.history`](../command/version.md#command--12-versionhistory) | `format::` |
| 11 | [`.config`](../command/config.md#command--13-config) | `format::` |
| 12 | [`.params`](../command/params.md#command--14-params) | `format::` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|---------|
| 1 | [`format::`](../param/05_format.md) | 12 |
