# Doc Entities

## Master Doc Entities Table

| Entity | Purpose | Master File | Instances |
|--------|---------|-------------|----------:|
| `api/` | Public API contracts for JournalWriter, JournalReader, and EventType | [api/readme.md](api/readme.md) | 3 |
| `feature/` | Behavioral requirements for the journal subsystem | [feature/readme.md](feature/readme.md) | 3 |
| `invariant/` | Measurable constraints for the journal write/read protocol | [invariant/readme.md](invariant/readme.md) | 3 |
| `tests/docs/api/` | Per-API test case specifications | [../../tests/docs/api/readme.md](../tests/docs/api/readme.md) | 3 |
| `tests/docs/feature/` | Per-feature test case specifications | [../../tests/docs/feature/readme.md](../tests/docs/feature/readme.md) | 3 |
| `tests/docs/invariant/` | Per-invariant test case specifications | [../../tests/docs/invariant/readme.md](../tests/docs/invariant/readme.md) | 3 |

## Master Doc Instances Table

| Entity | ID | Name | File |
|--------|----|------|------|
| api | 001 | Journal Writer | [api/001_journal_writer.md](api/001_journal_writer.md) |
| api | 002 | Journal Reader | [api/002_journal_reader.md](api/002_journal_reader.md) |
| api | 003 | Event Type | [api/003_event_type.md](api/003_event_type.md) |
| feature | 001 | Event Journaling | [feature/001_event_journaling.md](feature/001_event_journaling.md) |
| feature | 002 | Event Schema | [feature/002_event_schema.md](feature/002_event_schema.md) |
| feature | 003 | Rotation | [feature/003_rotation.md](feature/003_rotation.md) |
| invariant | 001 | Append-Only | [invariant/001_append_only.md](invariant/001_append_only.md) |
| invariant | 002 | Crash Safety | [invariant/002_crash_safety.md](invariant/002_crash_safety.md) |
| invariant | 003 | Schema Version | [invariant/003_schema_version.md](invariant/003_schema_version.md) |
