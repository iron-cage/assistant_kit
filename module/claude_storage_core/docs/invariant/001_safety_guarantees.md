# Invariant: Safety Guarantees

### Scope

- **Purpose**: Prevent data corruption in Claude Code's append-only JSONL storage through safe write patterns and format validation.
- **Responsibility**: Documents the three safety guarantees: append-only semantics, atomic writes, and format validation.
- **In Scope**: Append-only write contract, temp-file-rename atomic pattern, JSONL structure validation on read.
- **Out of Scope**: Performance constraints (→ `invariant/002_performance.md`), path encoding safety (→ `algorithm/001_path_encoding.md`).

### Invariant Statement

For all write operations: the library NEVER modifies or deletes existing JSONL entries. New entries are appended only. Write failures leave existing data intact. Every read validates JSONL structure before returning data.

### Enforcement Mechanism

**Append-only writes:** `Session::append_entry()` opens the session file in append mode only. No existing bytes are overwritten.

**Atomic write pattern:** New session files (or rewrites) use a three-step sequence: (1) write to a temporary file in the same directory, (2) sync the temporary file to disk (`fsync`), (3) rename the temporary file over the target. POSIX rename is atomic — readers either see the old file or the new file, never a partially-written intermediate state.

**Format validation:** All JSON parsing validates structure and syntax. Malformed JSONL lines emit a warning and are skipped (graceful degradation). The library never panics on corrupted input.

**Path safety:** `encode_path()` and `decode_path()` prevent path traversal by normalizing separators. Raw paths are never interpolated into filesystem operations without encoding.

### Violation Consequences

An append-only violation (accidental truncation or overwrite) would silently destroy conversation history with no recovery path (there is no backup mechanism). An atomic write failure leaving a partially written file would corrupt the session, causing parse errors and data loss for that session. Format validation failure (panic on corrupted input) would crash the application.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `../../src/session.rs` | append_entry() implementation, atomic write pattern |
| source | `../../src/json.rs` | Format validation during JSONL parse |
| doc | `../data_structure/001_storage_hierarchy.md` | Session type that enforces append-only semantics |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Combined specification; safety guarantees section extracted here |
