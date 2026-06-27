# CLI Type: Duration

Human-friendly duration string parsed into a time offset from now.

- **Kind:** Semantic
- **Fundamental:** String
- **Key Constraint:** Numeric value + time suffix

### Format

`<number><suffix>` where suffix is one of:

| Suffix | Unit | Example | Seconds |
|--------|------|---------|---------|
| `s` | Seconds | `30s` | 30 |
| `m` | Minutes | `15m` | 900 |
| `h` | Hours | `2h` | 7200 |
| `d` | Days | `7d` | 604800 |
| `w` | Weeks | `4w` | 2419200 |
| `M` | Months (30 days) | `3M` | 7776000 |

### Validation

- Must match regex `^[0-9]+[smhdwM]$`
- Numeric part must be > 0
- No spaces between number and suffix
- Case-sensitive: `M` = months, `m` = minutes

### Error Handling

Invalid duration strings cause exit 1 with message:
`Error: invalid duration '<input>' — expected format: <number><s|m|h|d|w|M>`

### Referenced Parameters

| # | Parameter |
|---|-----------|
| 01 | [`since`](../param/01_since.md) |
| 02 | [`until`](../param/02_until.md) |
