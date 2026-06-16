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
- **Validation errors:**
  - Non-integer: `"verbosity must be 0, 1, or 2, got: '{raw}'"`
  - Out of range: `"verbosity out of range: {n} (max 2)"`

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

### Referenced Parameters

| # | Parameter |
|---|-----------|
| 1 | [`v::`](../param/04_v.md) |
