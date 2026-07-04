# Pitfall Doc Entity

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
| `001_quota_gate_pitfalls.md` | PP- spec for quota gate pitfall guards (Pitfalls 1–5) |
| `002_subprocess_integration_pitfalls.md` | PP- spec for subprocess integration pitfall guards (Pitfalls 1–4) |
| `003_credential_sync_pitfalls.md` | PP- spec for credential sync pitfall guards (Pitfalls 1–5) |
| `004_account_identity_pitfalls.md` | PP- spec for account identity pitfall guards (Pitfalls 1–2) |
| `005_ownership_gate_pitfalls.md` | PP- spec for ownership gate pitfall guards (Pitfalls 1–4) |
| `006_model_override_pitfalls.md` | PP- spec for model override pitfall guards (Pitfalls 1–4) |

### Coverage Summary

| Pitfall Doc | File | PP Cases | Status |
|-------------|------|----------|--------|
| [001_quota_gate_pitfalls.md](../../../docs/pitfall/001_quota_gate_pitfalls.md) | [001_quota_gate_pitfalls.md](001_quota_gate_pitfalls.md) | PP-1 … PP-5 | ✅ |
| [002_subprocess_integration_pitfalls.md](../../../docs/pitfall/002_subprocess_integration_pitfalls.md) | [002_subprocess_integration_pitfalls.md](002_subprocess_integration_pitfalls.md) | PP-1 … PP-4 | ✅ |
| [003_credential_sync_pitfalls.md](../../../docs/pitfall/003_credential_sync_pitfalls.md) | [003_credential_sync_pitfalls.md](003_credential_sync_pitfalls.md) | PP-1 … PP-5 | ✅ |
| [004_account_identity_pitfalls.md](../../../docs/pitfall/004_account_identity_pitfalls.md) | [004_account_identity_pitfalls.md](004_account_identity_pitfalls.md) | PP-1 … PP-2 | ✅ |
| [005_ownership_gate_pitfalls.md](../../../docs/pitfall/005_ownership_gate_pitfalls.md) | [005_ownership_gate_pitfalls.md](005_ownership_gate_pitfalls.md) | PP-1 … PP-4 | ✅ |
| [006_model_override_pitfalls.md](../../../docs/pitfall/006_model_override_pitfalls.md) | [006_model_override_pitfalls.md](006_model_override_pitfalls.md) | PP-1 … PP-4 | ✅ |
