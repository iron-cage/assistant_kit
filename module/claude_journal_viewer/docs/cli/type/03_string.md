# CLI Type: String

UTF-8 text string fundamental type.

- **Kind:** Fundamental
- **Fundamental:** String
- **Key Constraint:** Any valid UTF-8 text

### Validation

- No length limit enforced by the type itself
- Specific parameters may impose additional constraints (e.g., regex validity for `pattern`, column name validity for `columns`)

### Referenced Parameters

| # | Parameter | Additional Constraint |
|---|-----------|----------------------|
| 04 | [`command`](../param/04_command.md) | Exact match: CLR command name |
| 06 | [`model`](../param/06_model.md) | Substring match |
| 08 | [`creds`](../param/08_creds.md) | Exact match: credential name |
| 14 | [`pattern`](../param/14_pattern.md) | Valid Rust regex |
| 16 | [`bind`](../param/16_bind.md) | Valid IPv4/IPv6 address |
| 26 | [`columns`](../param/26_columns.md) | Comma-separated column names |
