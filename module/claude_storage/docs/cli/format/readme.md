# Output Formats

### Scope

- **Purpose**: Document the three named export rendering modes for the `.export` command.
- **Responsibility**: Index of format doc instances covering all export rendering modes.
- **In Scope**: Three named export formats (markdown, json, text) produced by `.export`.
- **Out of Scope**: Verbosity-driven output (→ `001_commands.md` per-command), parameter specs (→ `004_params.md`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `markdown.md` | Markdown export format rendering spec |
| `json.md` | JSON export format rendering spec |
| `text.md` | Plain text export format rendering spec |

### Taxonomy

The CLI produces two categories of output:

1. **Export formats** — controlled by the `format::` parameter on `.export`. Each format writes a complete session transcript to a file in a distinct structure. These are the formats documented in this catalog.
2. **Verbosity-driven output** — controlled by `verbosity::` across multiple commands. These are not named formats but progressive detail levels on the same structural output. Documented in [001_commands.md](../001_commands.md) per-command.

Export formats are mutually exclusive (one per invocation) and produce structurally different files.

### Catalog

| # | Format | Category | Trigger | File | Extension | Machine-Parseable |
|---|--------|----------|---------|------|-----------|-------------------|
| 1 | markdown | Export | `format::markdown` (default when omitted) | [002_markdown.md](002_markdown.md) | `.md` | No |
| 2 | json | Export | `format::json` | [001_json.md](001_json.md) | `.json` | Yes |
| 3 | text | Export | `format::text` | [003_text.md](003_text.md) | `.txt` | No |

**Parameter:** [`format::` in 004_params.md](../004_params.md#parameter--5-format) | **Type:** [`ExportFormat` in 005_types.md](../005_types.md#exportformat)

### Rendering Conventions

All three formats share these conventions:

- **Header section** — session ID, storage path, and entry count appear at the top of every export
- **Entry ordering** — entries appear in chronological order (same as JSONL file order)
- **Timestamps** — ISO 8601 strings as stored in the JSONL source (no reformatting)
- **File creation** — output file is created (or overwritten) atomically with `fsync` after write
- **Streaming** — markdown and text formats stream entries through a buffered writer; JSON streams raw JSONL lines. Memory usage scales with single-entry size, not session size
