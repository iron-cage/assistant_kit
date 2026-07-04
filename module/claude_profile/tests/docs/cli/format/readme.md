# Test: CLI Output Formats

FM-N format spec cases for clp CLI output formats. Each spec covers the structural
and behavioral contracts for one output format as defined in `docs/cli/format/`.

**FM- extension note:** FM- (Format Mode) is the project-local case prefix for CLI output
format specs. `docs/cli/format/` maps to `tests/docs/cli/format/` as a CLI-domain format
surface (see `tests/docs/cli/readme.md`). Min 4 FM- cases per spec.

### Responsibility Table

| File | Format | FM-N Cases |
|------|--------|-----------|
| `001_text.md` | `format::text` — default human-readable output | FM-1 through FM-4 |
| `002_json.md` | `format::json` — single-line machine-parseable JSON | FM-1 through FM-4 |
| `003_table.md` | `format::table` — compact aligned table (`.accounts` only) | FM-1 through FM-4 |

### Coverage Summary

| Format Files | Total FM- Cases |
|-------------|-----------------|
| 3 | 12 (4+4+4) |

### See Also

- [docs/cli/format/](../../../../../docs/cli/format/readme.md) — format source docs
- [tests/docs/cli/param/03_format.md](../param/03_format.md) — `format::` parameter edge cases
