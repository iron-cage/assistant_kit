# Export Format Tests

Output format verification tests for all 3 export format specifications in `docs/cli/format/`.
Mirror of [format/](../../../../docs/cli/format/readme.md).

### Scope

- **Purpose**: Verify that `.export` output conforms to each format's structural and content specification.
- **Responsibility**: FM-N format verification test plans per export format.
- **In Scope**: All 3 export formats (markdown, json, text), output structure, content block handling.
- **Out of Scope**: Export parameter edge cases (→ `param/06_format.md` wait — actually `param/05_format.md`), command integration (→ `command/06_export.md`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `01_markdown.md` | FM tests for Markdown export — structure, headings, metadata |
| `02_json.md` | FM tests for JSON export — structure, field presence, parsability |
| `03_text.md` | FM tests for Text export — content filtering, plain-text constraints |

### Test ID Convention

| Prefix | Category | Used In |
|--------|----------|---------|
| `FM-N` | Format specification | Export format tests (`format/`) |

### Aggregate Counts

| File | Tests |
|------|-------|
| `01_markdown.md` | 5 |
| `02_json.md` | 5 |
| `03_text.md` | 5 |
| **Total** | **15** |

### Related Documentation

- [format/](../../../../docs/cli/format/readme.md) — Source format specifications
- [command/06_export.md](../command/06_export.md) — Export command integration tests
- [param/05_format.md](../param/05_format.md) — `format::` parameter edge cases
