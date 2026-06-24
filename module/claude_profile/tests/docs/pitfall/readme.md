# Pitfall Guard Tests

Test cases verifying that each design pitfall documented in `docs/pitfall/` is guarded against.
Each spec covers one pitfall document instance and asserts that the described guard holds.

### Scope

- **Purpose**: Verify pitfall guards are in place; each case confirms the pitfall scenario cannot
  produce the described failure mode.
- **Responsibility**: Index of per-pitfall guard verification files (PP-N entries).
- **Case prefix**: `PP-` (Pitfall Protection)
- **Min cases**: 2 per spec
- **In Scope**: Pitfall instances from `docs/pitfall/` that have been guarded and verified.
- **Out of Scope**: Feature behavior (→ `../feature/`), CLI edge cases (→ `../cli/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `003_credential_sync_pitfalls.md` | PP- spec for credential sync pitfall guards (Pitfalls 1–5) |

### Coverage Summary

| Pitfall Doc | File | PP Cases | Status |
|-------------|------|----------|--------|
| [003_credential_sync_pitfalls.md](../../../docs/pitfall/003_credential_sync_pitfalls.md) | [003_credential_sync_pitfalls.md](003_credential_sync_pitfalls.md) | PP-1 … PP-5 | ✅ |
