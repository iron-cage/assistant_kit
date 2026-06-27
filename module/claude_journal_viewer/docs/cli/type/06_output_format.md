# CLI Type: OutputFormat

Enumeration of output serialization formats.

- **Kind:** Enum
- **Fundamental:** String
- **Key Constraint:** One of 4 variants

### Variants

| Variant | Output | Use Case |
|---------|--------|----------|
| `table` | Aligned columns with headers | Human reading in terminal |
| `json` | JSON array `[{...}, ...]` | Programmatic consumption, one complete object |
| `jsonl` | One JSON object per line | Streaming, piping to `jq`, log aggregation |
| `csv` | Header row + comma-separated values | Spreadsheet import, data analysis |

### Validation

- Case-insensitive matching
- Invalid variant causes exit 1 listing valid options

### Referenced Parameters

| # | Parameter |
|---|-----------|
| 10 | [`format`](../param/10_format.md) |
