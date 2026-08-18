# CLI Type: RetentionSpec

Retention specification for `.prune` — an age-based duration, floored
to whole days. Determines which journal files to delete. (A size-based
mode was considered and dropped — no consumer needs it; use `.status`
to monitor journal size.)

- **Kind:** Semantic
- **Fundamental:** String
- **Key Constraint:** Duration syntax, interpreted as whole days

### Format

Same syntax as the [Duration](01_duration.md) type (`<number><s|m|h|d|w>`),
then floored to whole days — journal files rotate daily, so day granularity
is the natural unit:

| Example | Meaning |
|---------|---------|
| `7d` | Delete files dated more than 7 days ago |
| `4w` | Delete files dated more than 28 days ago |
| `30d` | The default when `keep::` is omitted |
| `12h` | Floors to 0 days — keep only today's file |

### Validation

- Must match Duration format (`s`, `m`, `h`, `d`, `w` suffixes)
- Invalid format causes exit 1 with:
  `Error: invalid duration '<input>' (expected e.g. 30s, 5m, 1h, 7d, 2w)`

### Behavior

- Candidates are exactly the files named `YYYY-MM-DD.jsonl` whose filename
  date is strictly before `today - keep_days` (UTC) — filesystem mtime is
  never consulted, and non-matching filenames are ignored entirely
- Today's file is structurally never deleted, even at a 0-day window

### Referenced Parameters

| # | Parameter |
|---|-----------|
| 18 | [`keep`](../param/18_keep.md) |
