# Feature: List Accounts

### Scope

- **Purpose**: Give users a snapshot of all stored accounts and their metadata in one call.
- **Responsibility**: Documents the `account::list()` API and the `.account.list` CLI command (FR-8).
- **In Scope**: Account enumeration, metadata per entry, active-account marking, empty-store handling.
- **Out of Scope**: Switching accounts (→ 004_account_switch.md), token classification logic (→ 006_token_status.md).

### Design

`claude_profile` must enumerate all `*.credentials.json` files in `~/.claude/accounts/` and return for each:

| Field | Source | Notes |
|-------|--------|-------|
| `name` | Filename stem | e.g. `work` from `work.credentials.json` |
| `subscriptionType` | Credential file JSON field | Empty or absent → shown as `N/A` |
| `rateLimitTier` | Credential file JSON field | Empty or absent → shown as `N/A` |
| `expiresAt` | Credential file `expiresAt` field | Unix epoch milliseconds |
| `is_active` | Name matches `_active` marker content | `true` if name == contents of `~/.claude/accounts/_active` |

**Empty account store:** Returns an empty `Vec`, not an error. Exit 0.

**CLI output verbosity:**
- `v::0`: account names only (one per line), active account marked with `<-` suffix
- `v::1` (default): name + sub + tier + expiry + active marker
- `v::2`: same as v::1 (no additional fields at this level)
- `format::json`: JSON array with all fields

### Acceptance Criteria

- **AC-01**: Empty `~/.claude/accounts/` returns empty list, exit 0.
- **AC-02**: Each entry reports name, subscriptionType, rateLimitTier, expiresAt, is_active.
- **AC-03**: The account matching `_active` marker has `is_active: true`; all others `false`.
- **AC-04**: `format::json` output is a valid JSON array.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/account.rs` | `list()` — enumerates accounts/, reads _active marker |
| source | `src/commands.rs` | `account_list_routine()` — CLI handler |
| test | `tests/account_tests.rs::list_marks_active_account_via_active_marker` | Verifies is_active field |
| doc | [cli/commands.md](../cli/commands.md#command--3-accountlist) | CLI command specification |
