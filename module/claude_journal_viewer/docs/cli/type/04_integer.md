# CLI Type: Integer

Non-negative integer fundamental type.

- **Kind:** Fundamental
- **Fundamental:** Integer
- **Key Constraint:** Non-negative (>= 0)

### Validation

- Parsed as `u64` (unsigned 64-bit integer)
- Negative values cause exit 1
- Non-numeric input causes exit 1
- Specific parameters may constrain range further (e.g., `verbosity` accepts 0-2)

### Error Handling

Invalid integer input causes exit 1 with message:
`Error: invalid integer '<input>' for parameter '<name>'`

### Referenced Parameters

| # | Parameter | Range |
|---|-----------|-------|
| 05 | [`exit`](../param/05_exit.md) | 0-255 |
| 09 | [`limit`](../param/09_limit.md) | 0 = unlimited |
| 22 | [`verbosity`](../param/22_verbosity.md) | 0-2 |
| 27 | [`refresh`](../param/27_refresh.md) | 0 = disabled |
