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

### Validation

- Must match regex `^[0-9]+[smhdw]$` (no month suffix — use `d`/`w` multiples)
- No spaces between number and suffix
- Suffix is case-sensitive and lowercase-only

### Error Handling

Invalid duration strings cause exit 1 with message:
`Error: invalid duration '<input>' (expected e.g. 30s, 5m, 1h, 7d, 2w)`

### Referenced Parameters

| # | Parameter |
|---|-----------|
| 01 | [`since`](../param/01_since.md) |
| 02 | [`until`](../param/02_until.md) |
