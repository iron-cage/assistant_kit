# claude_journal

Append-only event journaling library for CLR automation sessions.

### Scope

Records structured events (execution, credential refresh, gate wait, retry, timeout) to daily JSONL files. Provides write-side (`JournalWriter`) and read-side (`JournalReader`) APIs. No CLI, no binary — pure library consumed by `claude_runner` (write path) and `claude_journal_viewer` (read path).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `src/lib.rs` | Public module re-exports and crate documentation |
| `src/event.rs` | EventType enum, EventRecord struct, EventFields bag |
| `src/writer.rs` | JournalWriter — append-only JSONL file writer |
| `src/reader.rs` | JournalReader — filtered iteration over JSONL files |
| `src/rotation.rs` | Daily file naming, listing, age/size pruning |
| `verb/` | Shell scripts implementing do-protocol verbs for this crate. |
| `docs/` | Behavioral requirements, API contracts, invariant constraints |
| `tests/` | Unit and integration tests for write/read/rotation |
