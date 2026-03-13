# Feature: Core Library

### Scope

- **Purpose**: Provide zero-dependency, safe, structured read/write access to Claude Code's conversation storage at `~/.claude/`.
- **Responsibility**: Documents the library's design principles, scope boundaries, and crate split rationale.
- **In Scope**: Design principles, backward-compatibility policy, crate split motivation, filesystem-native philosophy.
- **Out of Scope**: CLI interface (→ `claude_storage` crate), data model structure (→ `data_structure/`), API surface (→ `api/`).

### Design

The library is a pure extraction of storage primitives from the `claude_storage` CLI crate. It depends on nothing outside the standard library, making it suitable for use in environments where CLI dependencies (`unilang`, `phf`) are undesirable.

**Backward compatibility is a non-goal.** Public types, function signatures, and storage format reading logic can change freely between versions. This simplifies evolution and keeps the code clean — there are no compatibility shims, no version-negotiation logic, and no deprecated aliases.

**Filesystem-native design.** The library works directly with Claude Code's JSONL storage without additional database or abstraction layers. This provides human-readable data, easy debugging with standard tools, and atomic operations via POSIX filesystem semantics.

**Crate split rationale.** Separating the core library from the CLI binary means library consumers avoid pulling in CLI framework dependencies. The pattern mirrors ripgrep's `grep-regex` + `grep-cli` split.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `../../src/lib.rs` | Library entry point and public re-exports |
| source | `../../src/storage.rs` | Storage entry point type |
| doc | `../data_structure/001_storage_hierarchy.md` | Storage → Project → Session → Entry model |
| doc | `../api/001_public_api.md` | Public API stability guarantees |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Combined specification; design principles, crate split rationale, and filesystem-native design sections extracted here; storage model extracted to `data_structure/001_storage_hierarchy.md` |
