# Operation: Migration Guide

### Scope

- **Purpose**: Guide library consumers through migrating from `claude_storage` (monolithic) to `claude_storage_core` (extracted library).
- **Responsibility**: Documents the crate rename procedure for library users who depended directly on `claude_storage` for programmatic access.
- **In Scope**: Cargo.toml update, import path change, API compatibility note.
- **Out of Scope**: CLI usage changes (CLI users are unaffected), core library design (→ `claude_storage_core/docs/`).

### Procedure Steps

This migration applies to **library users only** — code that imported `claude_storage` types programmatically. CLI users (`cls` binary users) are not affected.

1. Update `Cargo.toml`: replace `claude_storage = { path = "../claude_storage" }` with `claude_storage_core = { path = "../claude_storage_core" }`.
2. Update all `use claude_storage::` imports to `use claude_storage_core::`.
3. Build and run tests. The public API is identical — only the crate name changed.

### Prerequisites

- `claude_storage_core` crate is available at a sibling path (`../claude_storage_core` relative to the dependent crate).
- No API changes are needed: `Storage`, `Project`, `Session`, `Entry`, and all other types are re-exported from `claude_storage_core` with identical signatures.

### Expected Outcome

All compilation errors of the form `could not find module claude_storage` are resolved. The binary continues to work as `claude_storage` CLI (`cls`) depends on `claude_storage_core` internally.

### Rollback Procedure

Revert `Cargo.toml` to point back to `claude_storage` and revert `use` statements. No data is modified by this migration.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | `../../module/claude_storage_core/docs/api/001_public_api.md` | API stability guarantees for migrated consumers |
| doc | `../../module/claude_storage_core/docs/feature/001_core_library.md` | Core library design rationale |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Combined specification; migration guide section extracted here |
