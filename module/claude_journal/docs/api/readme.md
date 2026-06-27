# api/

### Scope

**Responsibilities:** Public API contracts for the `claude_journal` crate.
**In Scope:** JournalWriter append interface, JournalReader query/tail interface, EventType enum and EventRecord struct.
**Out of Scope:** Internal serialization helpers, JSONL parsing internals.

### Responsibility Table

| # | File | Responsibility |
|---|------|----------------|
| 001 | `001_journal_writer.md` | JournalWriter: `new(dir)` -> `append(event)` contract |
| 002 | `002_journal_reader.md` | JournalReader: `open(dir)` -> `query(filter)` / `tail(filter)` contract |
| 003 | `003_event_type.md` | EventType enum, EventRecord struct, EventFields field bag |
