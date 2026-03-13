# Feature: Content Search

### Scope

- **Purpose**: Enable streaming, memory-efficient search across conversation sessions without loading full session data.
- **Responsibility**: Documents the search design: streaming approach, filter types, result structure, and performance targets.
- **In Scope**: SearchFilter and SearchMatch design, streaming implementation strategy, role/content-type filtering.
- **Out of Scope**: Filter composition for sessions/projects (→ `data_structure/002_filter_types.md`), CLI search interface (→ `claude_storage` crate).

### Design

Content search scans JSONL files line-by-line without loading full session data into memory. Each search yields `SearchMatch` results as an iterator — memory usage is O(matches), not O(session size). This makes workspace-wide search across thousands of sessions feasible.

**Zero dependencies.** No regex crate. Substring matching only (case-insensitive by default). This is a deliberate tradeoff: stdlib-only search avoids adding dependencies, and case-insensitive substring matching covers the overwhelming majority of real use cases.

**Search parameters.** A `SearchFilter` specifies the query string, case-sensitivity flag, optional role filter (user/assistant/system), and optional content-type filter (text/code/thinking). Matches include the entry index, role, line number, and a 50-character context excerpt.

**Performance target.** Less than 5 seconds for 1000 sessions on a warm filesystem cache.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `../../src/search.rs` | SearchFilter, SearchMatch types and search implementation |
| source | `../../src/session.rs` | Session::search() method |
| source | `../../src/storage.rs` | Storage::search_all() method |
| test | `../../tests/search.rs` | Search correctness and streaming tests |
| doc | `../invariant/002_performance.md` | Performance targets including search |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Combined specification; content search section extracted here |
