# Invariant: Performance

### Scope

- **Purpose**: Document the performance characteristics and constraints that must be respected when using storage operations.
- **Responsibility**: State the fast-path vs full-parse cost model, concrete measurements, and API usage rules.
- **In Scope**: .status verbosity mode costs, min_entries::N cost, count_entries() cost model, avoidance rules.
- **Out of Scope**: Testing strategy (→ `invariant/003_testing_strategy.md`), feature design (→ `feature/001_workspace_design.md`).

### Invariant Statement

Operations on Claude Code storage have two distinct cost modes. Callers must not unknowingly use expensive operations in hot paths.

**Fast path (filesystem only) — O(P+S):**
- `.status` at `v::0` and `v::1`: ~50ms
- Uses only directory listings and filename inspection; no JSONL content is read
- P = project count, S = total session file count

**Full path (JSONL parsing) — O(total JSONL bytes):**
- `.status` at `v::2` and above: several minutes with large storage
- `.list min_entries::N`: O(total JSONL bytes) — reads every session file
- `Session::count_entries()` per session: fast per-file, but O(total_JSONL_bytes) aggregate

### Enforcement Mechanism

There is no session entry count index. `min_entries::N` filtering and entry count collection require reading every JSONL file. This is by design — an index would require maintenance and storage overhead.

**Measured baseline (1903 projects / 2429 sessions / ~7 GB JSONL):**
- `.list min_entries::N` cold cache: ~12 minutes
- `.list min_entries::N` warm cache: ~25 seconds
- `.status v::0` / `v::1`: ~50ms regardless of cache

**`count_entries()` implementation note:** Uses byte-level string search on `"type":"user"` and `"type":"assistant"` patterns (not full JSON parsing). These patterns are unique to top-level type fields in well-formed JSONL — fast and accurate. But never call `count_entries()` in a loop over thousands of sessions without awareness of the aggregate O(total_JSONL_bytes) cost.

**Guidance:** Use `.count` (fast path) for project/session counts in scripts that run frequently. Use `min_entries::N` only in analysis workflows that explicitly require entry counts and can tolerate the runtime cost.

### Violation Consequences

- Calling `count_entries()` in a hot loop causes O(GB) reads for what appears to be a simple count operation
- Using `min_entries::N` in a frequent script causes multi-minute runtimes
- Confusing fast-path and full-path operation costs leads to unexpectedly slow pipelines

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| feature | [feature/001_workspace_design.md](../feature/001_workspace_design.md) | Workspace design that includes these storage operations |
| source | `../../module/claude_storage_core/src/` | count_entries() and storage parsing implementation |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Performance Characteristics section (.status verbosity, min_entries, count_entries) |
