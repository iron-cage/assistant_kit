# Output Formats

### Scope

- **Purpose**: Document the three named export rendering modes for the `.export` command.
- **Responsibility**: Index of format doc instances covering all export rendering modes.
- **In Scope**: Three named export formats (markdown, json, text) produced by `.export`.
- **Out of Scope**: Per-command output toggles (→ `command/` per-command, `param_group/01_output_control.md`), parameter specs (→ `param/`).

### Taxonomy

The CLI produces two categories of output:

1. **Export formats** — controlled by the `format::` parameter on `.export`. Each format writes a complete session transcript to a file in a distinct structure. These are the formats documented in this catalog.
2. **Command output** — controlled by per-command boolean toggles (`show_tokens::`, `show_stat::`, `show_tree::`). Not named formats; documented in [command/](../command/) per-command and [param_group/01_output_control.md](../param_group/01_output_control.md).

Export formats are mutually exclusive (one per invocation) and produce structurally different files.

### Catalog

| # | Format | Category | Trigger | File | Extension | Machine-Parseable |
|---|--------|----------|---------|------|-----------|-------------------|
| 1 | markdown | Export | `format::markdown` (default when omitted) | [01_markdown.md](01_markdown.md) | `.md` | No |
| 2 | json | Export | `format::json` | [02_json.md](02_json.md) | `.json` | Yes |
| 3 | text | Export | `format::text` | [03_text.md](03_text.md) | `.txt` | No |

**Parameter:** [`format::`](../param/05_format.md) | **Type:** [`ExportFormat`](../type/03_export_format.md)

### Rendering Convention

All three formats share these conventions:

- **Header section** — session ID, storage path, and entry count appear at the top of every export
- **Entry ordering** — entries appear in chronological order (same as JSONL file order)
- **Timestamps** — ISO 8601 strings as stored in the JSONL source (no reformatting)
- **File creation** — output file is created (or overwritten) atomically with `fsync` after write
- **Streaming** — markdown and text formats stream entries through a buffered writer; JSON streams raw JSONL lines. Memory usage scales with single-entry size, not session size
