# Feature: Accounts

### Scope

- **Purpose**: Give users a snapshot of all stored accounts and their metadata in one call, with optional scoping to a single named account.
- **Responsibility**: Documents the `account::list()` API and the `.accounts` CLI command (FR-8).
- **In Scope**: Account enumeration, metadata per entry, active-account marking, empty-store handling, named-account scoping, field-presence toggles, JSON output.
- **Out of Scope**: Switching accounts (→ 004_account_use.md), token classification logic (→ 006_token_status.md).

### Design

`claude_profile` must enumerate all `*.credentials.json` files in `{credential_store}` and return for each:

| Field | Source | Notes |
|-------|--------|-------|
| `name` | Filename stem | e.g. `alice@acme.com` from `alice@acme.com.credentials.json` |
| `is_active` | Name matches `_active` marker content | `true` if name == contents of `{credential_store}/_active` |
| `subscriptionType` | Credential file JSON field | Empty or absent → shown as `N/A` |
| `rateLimitTier` | Credential file JSON field | Empty or absent → shown as `N/A` |
| `expiresAt` | Credential file `expiresAt` field | Unix epoch milliseconds |
| `email` | Saved `{name}.claude.json` → `emailAddress` | Empty or absent → shown as `N/A` |
| `display_name` | Saved `{name}.claude.json` → `oauthAccount.displayName` | Empty or absent → shown as `N/A` |
| `role` | Saved `{name}.claude.json` → `oauthAccount.organizationRole` | Empty or absent → shown as `N/A` |
| `billing` | Saved `{name}.claude.json` → `oauthAccount.billingType` | Empty or absent → shown as `N/A` |
| `model` | Saved `{name}.settings.json` → `model` | Empty or absent → shown as `N/A` |

**Without `name::`:** Lists all accounts as indented key-val blocks, sorted alphabetically. Each block is a header line (email) followed by indented `Key:  value` lines. A blank line separates consecutive blocks.

**With `name::EMAIL`:** Shows only the block for the named account. Exits 2 if the account is not found. Exits 1 if the name fails email validation.

**Empty account store:** Prints `(no accounts configured)` and exits 0.

**Field-presence toggles (default-on):**
- `active::0` — suppress the `Active:` line
- `sub::0` — suppress the `Sub:` line
- `tier::0` — suppress the `Tier:` line
- `expires::0` — suppress the `Expires:` line
- `email::0` — suppress the `Email:` line

**Field-presence toggles (opt-in, default off):**
- `display_name::1` — show the `Display:` line
- `role::1` — show the `Role:` line
- `billing::1` — show the `Billing:` line
- `model::1` — show the `Model:` line

When all field toggles are disabled, only bare account name lines are printed (no indentation, no blank-line separators).

**`format::json`:** Returns a JSON array with all fields regardless of field-presence toggle values. Each object contains `name`, `is_active`, `subscription_type`, `rate_limit_tier`, `expires_at_ms`, `email`, `display_name`, `role`, `billing`, `model`.

### Acceptance Criteria

- **AC-01**: Empty credential store returns `(no accounts configured)`, exit 0.
- **AC-02**: Each entry reports `name`, `is_active`, `subscriptionType`, `rateLimitTier`, `expiresAt`, `email`; opt-in fields `display_name`, `role`, `billing`, `model` are available when enabled.
- **AC-03**: The account matching `_active` marker has `is_active: true`; all others `false`.
- **AC-04**: `format::json` output is a valid JSON array.
- **AC-05**: `name::EMAIL` scopes to single account; exit 2 if not found; exit 1 if invalid format.
- **AC-06**: Field-presence toggles suppress individual lines from text output only.
- **AC-07**: All fields disabled → bare name lines only; no blank-line separators.
- **AC-08**: Accounts listed alphabetically by name.
- **AC-09**: `display_name::1` shows `Display:` line per account from saved `{name}.claude.json`.
- **AC-10**: `role::1`, `billing::1`, `model::1` show corresponding lines per account from saved snapshots.
- **AC-11**: Accounts without saved metadata files show `N/A` for `email`, `display_name`, `role`, `billing`, `model`.
- **AC-12**: `format::json` includes `email`, `display_name`, `role`, `billing`, `model` keys per account object.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/account.rs` | `list()` — enumerates credential store, reads _active marker |
| source | `src/commands.rs` | `accounts_routine()` — CLI handler |
| doc | [cli/commands.md](../cli/commands.md#command--3-accounts) | CLI command specification |
| doc | [tests/docs/cli/command/03_accounts.md](../../tests/docs/cli/command/03_accounts.md) | Integration test plan |
