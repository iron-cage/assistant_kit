# CLI Type: Boolean

Binary flag using integer representation (unilang convention).

- **Kind:** Fundamental
- **Fundamental:** Integer
- **Key Constraint:** 0 or 1

### Values

| Value | Meaning |
|-------|---------|
| `0` | False / disabled / off |
| `1` | True / enabled / on |

### Validation

- Only `0` and `1` are accepted
- Any other value causes exit 1 with message:
  `Error: invalid boolean '<input>' for parameter '<name>' — expected 0 or 1`

### Referenced Parameters

| # | Parameter |
|---|-----------|
| 12 | [`reverse`](../param/12_reverse.md) |
| 17 | [`open`](../param/17_open.md) |
| 19 | [`dry_run`](../param/19_dry_run.md) |
| 20 | [`confirm`](../param/20_confirm.md) |
| 24 | [`no_color`](../param/24_no_color.md) |
| 25 | [`wide`](../param/25_wide.md) |
| 28 | [`include_stdout`](../param/28_include_stdout.md) |
