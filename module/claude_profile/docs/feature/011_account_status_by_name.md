# Feature: Account Status by Name

### Scope

- **Purpose**: Let users inspect any stored account's token state and metadata without switching the active account.
- **Responsibility**: Documents the `name::` extension to `.account.status` (FR-16).
- **In Scope**: Named-account path, backward-compatible active-account path, N/A normalization, NotFound exit.
- **Out of Scope**: Active-account-only behavior (documented in base `.account.status` command), live credentials (→ 012_live_credentials_status.md).

### Design

`.account.status` must accept an optional `name::` parameter:

**With `name::`:**
- Read the named account's own credential snapshot at `{credential_store}/{name}.credentials.json`.
- Compute token state from that snapshot's own `expiresAt` (never `unknown` — the file either exists or doesn't).
- Show `Email: N/A` and `Org: N/A` because `~/.claude/.claude.json` OAuth profile is only valid for the currently active session, not for stored snapshots.
- Return `NotFound` error (exit 2) if the named account does not exist in the store.

**Without `name::` (backward-compatible):**
- Behaviour is identical to the original implementation: read `_active` marker, read live `~/.claude/.credentials.json`.
- At `v::1` and above: show email and org from `~/.claude/.claude.json` for the active account.

**N/A normalization:** Both `subscriptionType` and `rateLimitTier` fields normalize absent or empty-string values to `N/A` — this applies regardless of the `name::` path. Ensures output is never blank for these fields.

**At `v::1` (default):** Show `Sub:` (subscriptionType) and `Tier:` (rateLimitTier) for all accounts.

### Acceptance Criteria

- **AC-01**: `clp .account.status name::alice@acme.com` exits 0 and shows token state from `alice@acme.com.credentials.json`.
- **AC-02**: `clp .account.status name::ghost@example.com` exits 2 with a not-found error.
- **AC-03**: Named non-active account shows `Email: N/A` and `Org: N/A`.
- **AC-04**: `clp .account.status` without `name::` continues to show active account (backward compatible).
- **AC-05**: Empty or absent `subscriptionType`/`rateLimitTier` → shown as `N/A`, never blank.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/commands.rs` | `account_status_routine()` — named-account branch |
| test | `tests/cli/account_status_name_test.rs::astname01–14` | Full named-account path coverage |
| test | `tests/cli/account_list_status_test.rs::astat11–12` | N/A normalization in list context |
| doc | [012_live_credentials_status.md](012_live_credentials_status.md) | Related command — live credentials without account store |
| doc | [cli/commands.md](../cli/commands.md#command--4-accountstatus) | CLI command specification |
| doc | [cli/testing/command/04_account_status.md](../cli/testing/command/04_account_status.md) | Manual integration tests IT-24, IT-25, IT-26 |
