# Doc Entities

### Master Doc Entities Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|----------:|
| `feature/` | Event journaling design: append-only logging, schema, rotation | [feature/readme.md](feature/readme.md) | 3 |
| `invariant/` | Structural constraints: append-only, crash safety, schema versioning | [invariant/readme.md](invariant/readme.md) | 3 |
| `api/` | Public library API contracts: JournalWriter, JournalReader, EventType | [api/readme.md](api/readme.md) | 3 |

### Master Doc Instances Table

| Entity | ID | Name | File |
|--------|----|------|------|
| feature | 001 | Event Journaling | [feature/001_event_journaling.md](feature/001_event_journaling.md) |
| feature | 002 | Event Schema | [feature/002_event_schema.md](feature/002_event_schema.md) |
| feature | 003 | Rotation | [feature/003_rotation.md](feature/003_rotation.md) |
| invariant | 001 | Append-Only | [invariant/001_append_only.md](invariant/001_append_only.md) |
| invariant | 002 | Crash Safety | [invariant/002_crash_safety.md](invariant/002_crash_safety.md) |
| invariant | 003 | Schema Version | [invariant/003_schema_version.md](invariant/003_schema_version.md) |
| api | 001 | JournalWriter | [api/001_journal_writer.md](api/001_journal_writer.md) |
| api | 002 | JournalReader | [api/002_journal_reader.md](api/002_journal_reader.md) |
| api | 003 | EventType | [api/003_event_type.md](api/003_event_type.md) |
