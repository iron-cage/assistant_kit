# Data Structure: Filter Types

### Scope

- **Purpose**: Enable zero-dependency, composable filtering of projects and sessions using AND-logic conditions.
- **Responsibility**: Documents the filter type structures, composition semantics, and StringMatcher design.
- **In Scope**: SessionFilter, ProjectFilter, StringMatcher field semantics, AND-composition rules, performance characteristics.
- **Out of Scope**: Content search filters (→ `feature/002_content_search.md`), filter application to the hierarchy (→ `data_structure/001_storage_hierarchy.md`).

### Abstract

The filter types encode query predicates over the storage hierarchy without any external regex or query-parsing library. All matching is case-insensitive substring matching via `StringMatcher`. Filters compose with AND logic — every specified condition must match.

### Structure

**SessionFilter** — predicates over a single session:
- `agent_only`: optional boolean; when `Some(true)` includes only agent sessions; `Some(false)` only main sessions.
- `min_entries`: optional minimum entry count (inclusive); requires counting entries per session.
- `session_id_substring`: optional case-insensitive substring match against the session ID string.

**ProjectFilter** — predicates over a single project:
- `path_substring`: optional case-insensitive substring match against the decoded project path.
- `min_entries`: optional minimum total entry count across all sessions in the project.
- `min_sessions`: optional minimum session count.

**StringMatcher** — zero-allocation case-insensitive matcher:
- Constructed from a pattern string (normalized to lowercase on construction).
- `matches(text)` returns true if the lowercased text contains the pattern.
- Empty pattern matches any text (pass-through semantics).

### Operations

- **Construct filter**: set desired fields; leave others as `None` for no constraint.
- **Apply**: `project.sessions_filtered(&filter)`, `storage.list_projects_filtered(&filter)`.
- **Match text**: `StringMatcher::new(pattern).matches(text)`.
- **Short-circuit**: filters are applied cheapest-first (agent flag → entry count → substring).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `../../src/filter.rs` | SessionFilter, ProjectFilter, StringMatcher implementations |
| test | `../../tests/filtering.rs` | Filter composition and edge-case tests |
| test | `../../tests/string_matcher.rs` | StringMatcher case-insensitive matching tests |
| doc | `../data_structure/001_storage_hierarchy.md` | Storage hierarchy that filters operate over |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Combined specification; filtering section extracted here |
