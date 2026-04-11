# CLI Output Formats

Output format catalog for `claude_storage` CLI. Documents each named rendering mode produced by the `format::` parameter on `.export`.

### Taxonomy

The CLI produces two categories of output:

1. **Export formats** — controlled by the `format::` parameter on `.export`. Each format writes a complete session transcript to a file in a distinct structure. These are the formats documented in this catalog.
2. **Verbosity-driven output** — controlled by `verbosity::` across multiple commands. These are not named formats but progressive detail levels on the same structural output. Documented in [commands.md](../commands.md) per-command.

Export formats are mutually exclusive (one per invocation) and produce structurally different files.

### Catalog

| Format | Extension | Content Type | Includes Thinking | Includes Tool Use | Machine-Parseable |
|--------|-----------|-------------|-------------------|-------------------|-------------------|
| [markdown](markdown.md) | `.md` | Human-readable conversation with metadata | Yes (collapsible) | Yes (code blocks) | No |
| [json](json.md) | `.json` | Pretty-printed raw JSONL entries in wrapper object | Yes (raw field) | Yes (raw field) | Yes |
| [text](text.md) | `.txt` | Plain transcript, text content only | No | No | No |

**Default:** `markdown` when `format::` is omitted.

**Parameter:** [`format::` in params.md](../params.md#parameter--5-format) | **Type:** [`ExportFormat` in types.md](../types.md#exportformat)

### Rendering Conventions

All three formats share these conventions:

- **Header section** — session ID, storage path, and entry count appear at the top of every export
- **Entry ordering** — entries appear in chronological order (same as JSONL file order)
- **Timestamps** — ISO 8601 strings as stored in the JSONL source (no reformatting)
- **File creation** — output file is created (or overwritten) atomically with `fsync` after write
- **Streaming** — markdown and text formats stream entries through a buffered writer; JSON streams raw JSONL lines. Memory usage scales with single-entry size, not session size
