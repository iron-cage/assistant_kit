# feature/

### Scope

**Responsibilities:** Behavioral requirements for the `claude_journal` crate covering event journaling, event schema design, and log rotation.
**In Scope:** Event recording lifecycle, JSONL schema structure, daily file rotation and retention pruning.
**Out of Scope:** CLI viewing (-> `claude_journal_viewer`), integration call sites (-> `claude_runner/docs/feature/002_journaling_integration.md`).

### Responsibility Table

| # | File | Responsibility |
|---|------|----------------|
| 001 | `001_event_journaling.md` | Core append-only event logging to daily JSONL files |
| 002 | `002_event_schema.md` | Extensible type-discriminated event structure with version field |
| 003 | `003_rotation.md` | Daily file rotation, age-based and size-based retention pruning |
