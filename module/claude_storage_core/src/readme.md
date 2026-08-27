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
| `event.rs` | Parse every JSONL line kind, including context attachments |
| `context.rs` | Fold an event stream into current session context state |
| `json.rs` | Parse JSON with zero dependencies |
| `path.rs` | Encode and decode storage path representations |
| `filter.rs` | Filter projects and sessions by criteria |
| `search.rs` | Search conversation content full-text |
| `export.rs` | Export session data to multiple formats |
| `stats.rs` | Aggregate storage statistics |
| `rollup.rs` | Group/filter/sort/project flexible token-usage rollups |
| `cost.rs` | Per-model cost-relevant usage scanning and aggregation |
| `family.rs` | Discover root session plus its agent children |
| `continuation.rs` | Detect session continuation chains |
| `transcript_answer.rs` | Read one turn's assistant answer out of a live transcript |
| `canonical.rs` | Resolve paths to canonical physical absolute form |
| `topic_session.rs` | Deterministic topic-name to session-UUID rule (UUIDv5) |
| `session_id.rs` | Typed wrapper for session UUID stem |
| `scope.rs` | Compute all 6 CLAUDE_* path variables via scope_for() |
| `error.rs` | Define crate error types |
