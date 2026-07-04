# Invariant Doc Entity

### Scope

- **Purpose**: Document non-functional constraints that the `claude_journal` crate must always satisfy.
- **Responsibility**: Index of invariant doc instances covering append-only semantics, crash safety, and schema version compatibility.
- **In Scope**: Append-only file semantics, crash safety guarantees, schema version forward compatibility.
- **Out of Scope**: Runtime performance targets (not yet measured), storage capacity limits (managed by rotation).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Append-Only](001_append_only.md) | Journal files are never modified — only appended | ✅ |
| 002 | [Crash Safety](002_crash_safety.md) | Partial write corrupts at most one trailing line | ✅ |
| 003 | [Schema Version](003_schema_version.md) | `v` field enables forward-compatible event parsing | ✅ |
