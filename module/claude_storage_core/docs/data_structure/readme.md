# Data Structure Doc Entity

### Scope

- **Purpose**: Document in-memory data models used to represent Claude Code's storage hierarchy and query filters.
- **Responsibility**: Index of data structure doc instances covering model relationships and type semantics.
- **In Scope**: Logical structure, field semantics, and composition rules for all public data types.
- **Out of Scope**: On-disk encoding (→ `algorithm/001_path_encoding.md`), API stability (→ `api/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Storage Hierarchy](001_storage_hierarchy.md) | Storage → Project → Session → Entry model | ✅ |
| 002 | [Filter Types](002_filter_types.md) | SessionFilter, ProjectFilter, StringMatcher composition | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating data structure doc instances | ✅ |
