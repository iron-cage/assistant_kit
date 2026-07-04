# Invariant Doc Entity

### Scope

**Responsibilities:** Non-negotiable structural constraints for the `claude_journal` crate.
**In Scope:** Append-only file semantics, crash safety guarantees, schema version forward compatibility.
**Out of Scope:** Runtime performance targets (not yet measured), storage capacity limits (managed by rotation).

### Responsibility Table

| # | File | Responsibility |
|---|------|----------------|
| 001 | `001_append_only.md` | Journal files are never modified — only appended |
| 002 | `002_crash_safety.md` | Partial write corrupts at most one trailing line |
| 003 | `003_schema_version.md` | `v` field enables forward-compatible event parsing |
