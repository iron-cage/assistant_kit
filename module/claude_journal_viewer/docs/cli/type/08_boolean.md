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
| 24 | [`no_color`](../param/24_no_color.md) |

This table is walked by `ec36_boolean_params_accept_only_0_and_1`, which applies
every value in § Validation to each entry — so a parameter added here without a
matching implementation fails rather than going unnoticed.

`include_stdout` (28) was a fifth entry until it was superseded. `.search` reads
`stdout`/`stderr` unconditionally, so the flag had no narrower default left to
widen and is no longer accepted; its
[page](../param/28_include_stdout.md) is kept only as a tombstone for the
documents that still link to it.
