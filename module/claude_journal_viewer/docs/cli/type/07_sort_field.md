# CLI Type: SortField

Enumeration of fields available as sort keys for `.list` output.

- **Kind:** Enum
- **Fundamental:** String
- **Key Constraint:** One of 6 variants

### Variants

| Variant | Sorted By | Default Direction |
|---------|-----------|-------------------|
| `time` | Event timestamp | Ascending (oldest first) |
| `cost` | API cost in USD | Ascending (cheapest first) |
| `duration` | Execution duration in seconds | Ascending (fastest first) |
| `exit` | Subprocess exit code | Ascending (0 first) |
| `model` | Model name alphabetically | Ascending (a-z) |
| `command` | CLR command name alphabetically | Ascending (a-z) |

### Validation

- Case-insensitive matching
- Invalid variant causes exit 1 listing valid options

### Referenced Parameters

| # | Parameter |
|---|-----------|
| 11 | [`sort`](../param/11_sort.md) |
