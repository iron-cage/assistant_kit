# tests/

### Scope

**Responsibilities:** Automated integration tests for the `claude_journal` crate — append-only daily JSONL event journal writing and reading.
**In Scope:** All crate functionality exercised via the public library API (`JournalWriter`, `JournalReader`, `JournalFilter`, `EventRecord`, `EventType`, `EventFields`).
**Out of Scope:** Manual testing, test planning documents (→ `docs/`).

### Domain Map

| Domain | File | Tests What |
|--------|------|------------|
| Journal integration (IT-1–IT-14) | `journal_integration_test.rs` | Daily JSONL file creation, ordered event query, `since` timestamp filtering, day-rotation filenames, corrupt/partial line skipping, concurrent-writer interleaving safety, `v` schema version invariant, `EventType` `as_str()`/`parse()` round-trip and unknown-value handling, `EventFields::default()` all-`None`, `Some`/`None` field serialization, existing variant string stability |
