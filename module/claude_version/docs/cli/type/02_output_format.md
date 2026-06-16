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

### Referenced Parameters

| # | Parameter |
|---|-----------|
| 1 | [`format::`](../param/05_format.md) |
