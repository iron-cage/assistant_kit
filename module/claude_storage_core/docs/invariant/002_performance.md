# Invariant: Performance

### Scope

- **Purpose**: Ensure the library performs within acceptable bounds for interactive and scripted use against real Claude Code storage (1900+ projects, 2400+ sessions, ~7 GB JSONL).
- **Responsibility**: Documents measurable performance targets and the cost model for each operation category.
- **In Scope**: Fast-path vs full-parse cost model, streaming guarantees, JSON parse throughput, search latency targets.
- **Out of Scope**: CLI-level performance (→ `claude_storage` crate), export format speed (documented in `feature/003_export_formats.md`).

### Invariant Statement

Fast-path operations (project enumeration, session listing, entry counting) MUST complete in under 100ms for 1900 projects / 2400 sessions on a warm filesystem cache. Full-parse operations (entry-level statistics, content search) operate at O(total JSONL bytes) and are expected to take minutes on cold cache; this is acceptable and documented behaviour, not a violation.

### Enforcement Mechanism

**Lazy loading.** Sessions and entries are not loaded until explicitly requested. `Storage::list_projects()` reads only directory listings — no JSONL content.

**Fast entry count.** `Session::count_entries()` uses byte-level string search for `"type":"user"` and `"type":"assistant"` patterns rather than full JSON parsing. This provides a 100x speedup (~5ms for 1000 entries vs ~500ms for full parse).

**Streaming search.** Content search reads JSONL line-by-line and yields matches immediately. Memory usage is O(matches), not O(session size).

**Statistics without full parse.** `Session::stats()` uses selective field extraction — only type, timestamp, and token usage fields are parsed; full entry content is skipped.

### Violation Consequences

A regression in fast-path performance (e.g. accidentally triggering JSONL parse during project listing) would make the `.status` command unusably slow for users with large storage (minutes instead of milliseconds). This would be immediately visible as a user-facing regression.

Measured targets (warm cache, real storage with ~7 GB JSONL):
- `list_projects()`: < 50ms
- `sessions_filtered()` (no `min_entries`): < 50ms
- `count_entries()` per session: < 5ms
- Content search per session: 100–500ms
- Workspace-wide search (1000 sessions): < 5s
- JSON parse: ~80ns/operation

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `../../src/session.rs` | count_entries() byte-level fast path |
| source | `../../src/stats.rs` | Selective-parse statistics aggregation |
| source | `../../src/search.rs` | Streaming search implementation |
| test | `../../tests/count_entries_bug.rs` | count_entries() correctness regression test |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Combined specification; performance characteristics section extracted here |
