# CLI Type: String

UTF-8 text string fundamental type.

- **Kind:** Fundamental
- **Fundamental:** String
- **Key Constraint:** Any valid UTF-8 text

### Validation

- No length limit enforced by the type itself
- **No `String` parameter is rejected at parse time for its content.** Every value in the table below is accepted as written; the "Additional Constraint" column describes how the value is *used*, not a validation gate it must pass

### Referenced Parameters

| # | Parameter | Additional Constraint |
|---|-----------|----------------------|
| 04 | [`command`](../param/04_command.md) | Exact match: CLR command name |
| 06 | [`model`](../param/06_model.md) | Substring match |
| 08 | [`creds`](../param/08_creds.md) | Exact match: credential name |
| 14 | [`pattern`](../param/14_pattern.md) | Literal substring — no pattern syntax |
| 16 | [`bind`](../param/16_bind.md) | Address string; validated by the OS at bind, not at parse |

Two of these were previously written as validation rules that do not exist.
`pattern` was documented as requiring a valid Rust regex — the crate has no
`regex` dependency at all, so `pattern::"("` is not an error, it is a search for
an open parenthesis. `bind` was documented as requiring a valid IPv4/IPv6
address — it is handed to `tiny_http::Server::http()` untouched, so a malformed
address surfaces as an exit-1 *bind failure* at startup, not a parameter
rejection (see [param/16_bind.md](../param/16_bind.md)).

The distinction matters for callers: a validation error names the parameter and
happens before anything runs, while these surface later and describe something
else — or, in `pattern`'s case, never surface at all.
