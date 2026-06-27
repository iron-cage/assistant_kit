# CLI Type: RetentionSpec

Retention specification for `.prune` — either an age-based duration
or a size-based limit. Determines which journal files to delete.

- **Kind:** Semantic
- **Fundamental:** String
- **Key Constraint:** Duration suffix (age) or byte suffix (size)

### Format

**Age-based** — same syntax as Duration type:

| Example | Meaning |
|---------|---------|
| `7d` | Delete files older than 7 days |
| `4w` | Delete files older than 4 weeks |
| `3M` | Delete files older than 3 months |
| `24h` | Delete files older than 24 hours |

**Size-based** — numeric value + byte suffix:

| Suffix | Unit | Example |
|--------|------|---------|
| `kb` | Kilobytes | `500kb` |
| `mb` | Megabytes | `100mb` |
| `gb` | Gigabytes | `1gb` |

### Validation

- Must match either Duration format or size format
- Size suffixes are case-insensitive (`MB` = `mb`)
- Invalid format causes exit 1 with:
  `Error: invalid retention spec '<input>' — expected duration (7d, 4w) or size (100mb, 1gb)`

### Behavior

- **Age-based**: Files with `YYYY-MM-DD` in filename older than threshold are candidates
- **Size-based**: Sum all file sizes; delete oldest files until total is under threshold

### Referenced Parameters

| # | Parameter |
|---|-----------|
| 18 | [`keep`](../param/18_keep.md) |
