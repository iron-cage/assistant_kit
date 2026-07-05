# Feature: Export Formats

### Scope

- **Purpose**: Allow conversations to be exported in human-readable or machine-readable formats for archival, review, or programmatic processing.
- **Responsibility**: Documents the three supported export formats and the streaming export design.
- **In Scope**: Markdown, JSON, and plain-text export; format selection API; streaming writer contract; graceful metadata entry skipping.
- **Out of Scope**: CLI export command (→ `claude_storage` crate), session loading (→ `data_structure/001_storage_hierarchy.md`).

### Design

Export converts a `Session` into a writer-based stream in one of three formats:

- **Markdown**: Human-readable conversation with message structure, timestamps, token counts, and collapsible thinking blocks. Suitable for archival and documentation.
- **JSON**: Pretty-printed version of Claude Code's internal JSONL format. Suitable for programmatic processing — the structure is identical to the on-disk format.
- **Plain text**: Minimal transcript (role: content lines only). No metadata. Suitable for quick review.

**Streaming.** Export writes to any `Write` implementor. Memory usage is O(1) — entries are serialized incrementally without loading all of them simultaneously. The format is selected via `ExportFormat` enum.

**Graceful degradation.** Non-conversation metadata entries (e.g. queue-operation, summary) are automatically skipped during export — see [003_entry_type_format.md](../../../claude_storage/docs/invariant/003_entry_type_format.md) for the full type contract and evidence tiers. Corrupted entries emit a warning and are skipped without aborting the export.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `../../src/export.rs` | ExportFormat enum, export_session(), export_session_to_file() |
| source | `../../src/session.rs` | Session entry iteration used by export |
| test | `../../tests/export.rs` | Export format correctness tests |
| doc | `../invariant/002_performance.md` | Export throughput targets |
| doc | `../../../claude_storage/docs/invariant/003_entry_type_format.md` | Non-conversation `type` value contract and evidence tiers |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Combined specification; export functionality section extracted here |
