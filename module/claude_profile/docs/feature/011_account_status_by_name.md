# Feature: Named Account Scoping

### Scope

- **Purpose**: Let users inspect any stored account's metadata without switching the active account.
- **Responsibility**: Documents the `name::` extension to `.accounts` (FR-16).
- **In Scope**: Named-account path, backward-compatible all-accounts path, N/A normalization, NotFound exit.
- **Out of Scope**: Active-account-only behavior (documented in base `.accounts` command), live credentials (→ 012_live_credentials_status.md).

### Design

`.accounts` must accept an optional `name::` parameter:

**With `name::`:**
- Read the named account's credential snapshot at `{credential_store}/{name}.credentials.json`.
- Display a single indented key-val block for that account only.
- Return `NotFound` error (exit 2) if the named account does not exist in the store.
- Return usage error (exit 1) if the name fails email validation.

**Without `name::` (default):**
- List all accounts sorted alphabetically as indented key-val blocks.
- Empty store → `(no accounts configured)`, exit 0.

**N/A normalization:** Both `subscriptionType` and `rateLimitTier` fields normalize absent or empty-string values to `N/A` — regardless of the `name::` path.

**Field-presence toggles apply equally** whether `name::` is used or not — the same `active::`, `sub::`, `tier::`, `expires::`, `email::` toggles suppress lines.

### Acceptance Criteria

- **AC-01**: `clp .accounts name::alice@acme.com` exits 0 and shows one indented block for `alice@acme.com`.
- **AC-02**: `clp .accounts name::ghost@example.com` exits 2 with a not-found error.
- **AC-03**: `clp .accounts name::notanemail` exits 1 with a validation error.
- **AC-04**: `clp .accounts` without `name::` lists all accounts (backward compatible).
- **AC-05**: Empty or absent `subscriptionType`/`rateLimitTier` → shown as `N/A`, never blank.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/commands.rs` | `accounts_routine()` — named-account branch |
| test | `tests/cli/accounts_test.rs` | Named-account path coverage (acc04, acc05, acc06) |
| doc | [012_live_credentials_status.md](012_live_credentials_status.md) | Related command — live credentials without account store |
| doc | [cli/commands.md](../cli/commands.md#command--3-accounts) | CLI command specification |
| doc | [tests/docs/cli/command/03_accounts.md](../../tests/docs/cli/command/03_accounts.md) | Integration tests IT-4, IT-5, IT-6 |
