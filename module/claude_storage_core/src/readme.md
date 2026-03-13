# src/

Zero-dependency core library for reading Claude Code filesystem storage.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lib.rs` | Define crate root and re-export public API |
| `storage.rs` | Provide main storage interface and entry point |
| `project.rs` | Represent and query project directories |
| `session.rs` | Represent and query conversation sessions |
| `entry.rs` | Parse conversation entry types from JSONL |
| `json.rs` | Parse JSON with zero dependencies |
| `path.rs` | Encode and decode storage path representations |
| `filter.rs` | Filter projects and sessions by criteria |
| `search.rs` | Search conversation content full-text |
| `export.rs` | Export session data to multiple formats |
| `stats.rs` | Aggregate storage statistics |
| `continuation.rs` | Detect session continuation chains |
| `error.rs` | Define crate error types |
