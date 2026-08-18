# src/

Source code for the `claude_profile_core` crate.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lib.rs` | Crate root; module declarations and public re-exports |
| `token.rs` | `TokenStatus`: read `expiresAt`, classify Valid / ExpiringSoon / Expired |
| `account/` | Account domain engine: store CRUD, switching, refresh, ownership, quota cache |
