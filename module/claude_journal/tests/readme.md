# tests/

### Scope

**Responsibilities:** Automated integration tests for the `claude_journal` crate — append-only daily JSONL event journal writing and reading.
**In Scope:** All crate functionality exercised via the public library API (`JournalWriter`, `JournalReader`, `JournalFilter`, `EventRecord`, `EventType`, `EventFields`).
**Out of Scope:** Manual testing, test planning documents (→ `docs/`).

### Domain Map

| Domain | File | Tests What |
|--------|------|------------|
| Journal integration (IT-1–IT-18) | `journal_integration_test.rs` | Daily JSONL file creation, ordered event query, `since` timestamp filtering, day-rotation filenames, corrupt/partial line skipping, concurrent-writer interleaving safety, `v` schema version invariant, `EventType` `as_str()`/`parse()` round-trip and unknown-value handling, `EventFields::default()` all-`None`, `Some`/`None` field serialization, existing variant string stability, `tail()` full-batch delivery and torn-line deferral (audit-tail-data-loss), corrupt-`ts` exclusion under time bounds, `Duration::MAX` since-window safety |
| Rotation retention (RT-1–RT-12) | `rotation_test.rs` | `parse_date_filename` strictness, `list_journal_files` filtering/sorting, `prune_by_age` cutoff math (strict `<`, month boundaries, `keep_days = 0` never touching today), `dry_run` reporting, non-matching-filename immunity, missing-dir no-op |
