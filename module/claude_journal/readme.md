# claude_journal

Append-only event journaling library for CLR automation sessions.

### Scope

Records structured events (execution, credential refresh, gate wait, retry, timeout) to daily JSONL files. Provides write-side (`JournalWriter`) and read-side (`JournalReader`) APIs. No CLI, no binary — pure library consumed by `claude_runner` (write path) and `claude_journal_viewer` (read path).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `Cargo.toml` | Crate manifest and dependency configuration |
| `src/` | JournalWriter, JournalReader, and event/rotation implementation |
| `verb/` | Shell scripts implementing do-protocol verbs for this crate. |
| `docs/` | Behavioral requirements, API contracts, invariant constraints |
| `tests/` | Unit and integration tests for write/read/rotation |
