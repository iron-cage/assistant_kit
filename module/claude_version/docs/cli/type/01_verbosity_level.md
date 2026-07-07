# Type :: 1. `VerbosityLevel`

-- **Summary:** Controls the detail level of command output.
-- **Base Type:** u8
-- **Constraints:** 0 to 2
-- **Default:** 1 (normal)
-- **Used By:** `v::`

Range 0–2 with different semantics from claude_runner's 0–5 range.

- **Base type:** u8
- **Constraints:** 0 to 2
- **Default:** 1 (normal)
- **Validation errors** (`{key}` is whichever alias was used — `v` or `verbosity` — always followed by `::`, per `normalise_verbosity()`):
  - Non-integer: `"{key}:: must be 0, 1, or 2, got: '{raw}'"`
  - Out of range: `"{key}:: out of range: {n} (max 2)"`

**Level Semantics:**

| Level | Name | Output |
|-------|------|--------|
| 0 | minimal | Raw values only (no labels) |
| 1 | normal | Labeled key-value pairs (default) |
| 2 | verbose | Diagnostic details, extra context |

```sh
clv .status v::0       # minimal
clv .status v::2       # verbose
clv .status v::3       # error: out of range
```

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|--------------|
| 1 | [`.status`](../command/root.md#command-2-status) | `v::` |
| 2 | [`.version.show`](../command/version.md#command-3-versionshow) | `v::` |
| 3 | [`.version.install`](../command/version.md#command-4-versioninstall) | `v::` |
| 4 | [`.version.guard`](../command/version.md#command-5-versionguard) | `v::` |
| 5 | [`.version.list`](../command/version.md#command-6-versionlist) | `v::` |
| 6 | [`.processes`](../command/processes.md#command-7-processes) | `v::` |
| 7 | [`.processes.kill`](../command/processes.md#command-8-processeskill) | `v::` |
| 8 | [`.settings.show`](../command/settings.md#command-9-settingsshow) | `v::` |
| 9 | [`.settings.get`](../command/settings.md#command-10-settingsget) | `v::` |
| 10 | [`.version.history`](../command/version.md#command-12-versionhistory) | `v::` |
| 11 | [`.config`](../command/config.md#command-13-config) | `v::` |
| 12 | [`.params`](../command/params.md#command-14-params) | `v::` |
| 13 | [`.paths`](../command/paths.md#command--16-paths) | `v::` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|---------|
| 1 | [`v::`](../param/04_v.md) | 13 |
