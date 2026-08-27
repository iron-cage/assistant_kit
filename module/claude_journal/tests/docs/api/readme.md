# API Doc Entity

### Scope

- **Purpose**: Document test case planning for API doc instances in `docs/api/`.
- **Responsibility**: Index of per-api-doc test case spec files.
- **In Scope**: API doc instances 001–003. Instance 004 (`004_rotation.md`) is covered, but its test spec lives in the feature mirror — see the Responsibility Table note below.
- **Out of Scope**: Feature tests (→ `../feature/`), invariant tests (→ `../invariant/`).

Per-api-doc test case indices for `claude_journal`. See [api/readme.md](../../../docs/api/readme.md) for the API doc instances.

### Responsibility Table

| Name | Purpose | Status |
|------|---------|--------|
| [001_journal_writer.md](001_journal_writer.md) | Test cases for the JournalWriter API | ✅ |
| [002_journal_reader.md](002_journal_reader.md) | Test cases for the JournalReader + JournalFilter API | ✅ |
| [003_event_type.md](003_event_type.md) | Test cases for EventType, EventRecord, EventFields | ✅ |
| — (no api-side spec) | [`docs/api/004_rotation.md`](../../../docs/api/004_rotation.md) is covered by [`../feature/003_rotation.md`](../feature/003_rotation.md), which specs the same surface (`date_filename()`, `list_journal_files()`, `prune_by_age()`, `prune_by_size()`) against `tests/rotation_test.rs` | ✅ |
