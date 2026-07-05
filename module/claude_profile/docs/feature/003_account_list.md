# Feature: Accounts

### Scope

- **Purpose**: Give users a snapshot of all stored accounts and their metadata in one call, with optional scoping to a single named account.
- **Responsibility**: Documents the `account::list()` API and the `.accounts` CLI command (FR-8).
- **In Scope**: Account enumeration, metadata per entry, active-account marking, current-account detection, empty-store handling, named-account scoping, field-presence toggles, JSON output.
- **Out of Scope**: Switching accounts (→ 004_account_use.md), token classification logic (→ 006_token_status.md).

### Design

`claude_profile` must enumerate all `*.credentials.json` files in `{credential_store}` and return for each:

| Field | Source | Notes |
|-------|--------|-------|
| `name` | Filename stem | e.g. `alice@acme.com` from `alice@acme.com.credentials.json` |
| `is_active` | Name matches per-machine active marker content | `true` if name == contents of per-machine active marker file |
| `is_current` | `accessToken` matches `~/.claude/.credentials.json` | `true` for at most one account; `false` for all when credentials file unreadable |
| `subscriptionType` | Credential file JSON field | Empty or absent → shown as `N/A` |
| `rateLimitTier` | Credential file JSON field | Empty or absent → shown as `N/A` |
| `expiresAt` | Credential file `expiresAt` field | Unix epoch milliseconds |
| `email` | Saved `{name}.json` → `emailAddress` | Empty or absent → shown as `N/A` |
| `display_name` | Saved `{name}.json` → `oauthAccount.displayName` | Empty or absent → shown as `N/A` |
| `org_role` | Saved `{name}.json` → `organization_role` | Org admin/member role (e.g. `"admin"`, `"member"`); empty or absent → `N/A` |
| `billing` | Saved `{name}.json` → `oauthAccount.billingType` | Empty or absent → shown as `N/A` |
| `model` | Saved `{name}.json` → `model` field (BUG-222 fix: captured by `save()`) | Empty or absent → shown as `N/A` |
| `tagged_id` | Saved `{name}.json` → `oauthAccount.taggedId` | Empty or absent → shown as `N/A` |
| `uuid` | Saved `{name}.json` → `oauthAccount.uuid` | Empty or absent → shown as `N/A` |
| `capabilities` | Saved `{name}.json` → `oauthAccount.capabilities[]` | Empty array or absent → shown as `N/A` |
| `organization_uuid` | Saved `{name}.json` → `organization_uuid` | Empty or absent → shown as `N/A` |
| `organization_name` | Saved `{name}.json` → `organization_name` | Empty or absent → shown as `N/A` |
| `workspace_uuid` | Saved `{name}.json` → `workspace_uuid` | Enterprise accounts only; empty string for personal accounts; JSON output only — no text display line; see [022_org_identity_snapshot.md](022_org_identity_snapshot.md) |
| `workspace_name` | Saved `{name}.json` → `workspace_name` | Enterprise accounts only; empty string for personal accounts; JSON output only — no text display line; see [022_org_identity_snapshot.md](022_org_identity_snapshot.md) |
| `role` | Saved `{name}.json` → `role` | User-defined label (e.g. `"work"`, `"dev"`); see [029_account_host_metadata.md](029_account_host_metadata.md); empty or absent → `N/A` |
| `host` | Saved `{name}.json` → `host` | Machine host label; see [029_account_host_metadata.md](029_account_host_metadata.md); empty or absent → `N/A` |
| `owner` | Saved `{name}.json` → `owner` | Ownership identity string; empty = unowned (no enforcement); see [036_account_ownership.md](036_account_ownership.md) |
| `is_owned` | Derived: `owner.is_empty() \|\| owner == current_identity()` | `true` when unowned or owned by this machine; see [036_account_ownership.md](036_account_ownership.md) |
| `renewal_at` | Saved `{name}.json` → `_renewal_at` | Billing renewal timestamp override; absent when not set; see [030_account_renewal_override.md](030_account_renewal_override.md) |

**Without `name::`:** Lists all accounts as indented key-val blocks, sorted alphabetically. Each block is a header line (email) followed by indented `Key:  value` lines. A blank line separates consecutive blocks.

**With `name::EMAIL`:** Shows only the block for the named account. Exits 2 if the account is not found. Exits 1 if the name fails email validation.

**Empty account store:** Prints `(no accounts configured)` and exits 0.

**Column suppression via `cols::` (default-on columns, remove with `cols::-<col>`):**
- `cols::-active` — suppress the `Active:` line
- `cols::-current` — suppress the `Current:` line (live credential match; see [016_current_account_awareness.md](016_current_account_awareness.md))
- `cols::-sub` — suppress the `Sub:` line
- `cols::-tier` — suppress the `Tier:` line
- `cols::-expires` — suppress the `Expires:` line
- `cols::-email` — suppress the `Email:` line
- `cols::-owner` — suppress the `Owner:` line

**Column addition via `cols::` (opt-in columns, add with `cols::+<col>`):**
- `cols::+display_name` — show the `Display:` line
- `cols::+host` — show the `Host:` line
- `cols::+role` — show the `Role:` line
- `cols::+billing` — show the `Billing:` line
- `cols::+model` — show the `Model:` line
- `cols::+uuid` — show the `ID:` line (tagged user ID)
- `cols::+capabilities` — show the `Capabilities:` line
- `cols::+org_uuid` — show the `Org ID:` line
- `cols::+org_name` — show the `Org:` line

When all default-on columns are removed via `cols::`, only bare account name lines are printed (no indentation, no blank-line separators).

**`format::json`:** Returns a JSON array with all fields regardless of `cols::` column settings. Each object contains `name`, `is_active`, `is_current`, `subscription_type`, `rate_limit_tier`, `expires_at_ms`, `email`, `display_name`, `role`, `billing`, `model`, `tagged_id`, `capabilities`, `organization_uuid`, `organization_name`, `organization_role`, `workspace_uuid`, `workspace_name`, `host`, `owner`, `is_owned`, `renewal_at`.

### Acceptance Criteria

- **AC-01**: Empty credential store returns `(no accounts configured)`, exit 0.
- **AC-02**: Each entry reports `name`, `is_active`, `subscriptionType`, `rateLimitTier`, `expiresAt`, `email`; opt-in fields `display_name`, `role`, `billing`, `model` are available when enabled.
- **AC-03**: The account matching the per-machine active marker has `is_active: true`; all others `false`.
- **AC-04**: `format::json` output is a valid JSON array.
- **AC-05**: `name::EMAIL` scopes to single account; exit 2 if not found; exit 1 if invalid format.
- **AC-06**: `cols::-<col>` removes individual lines from text output; `cols::+<col>` adds opt-in lines. `format::json` always includes all fields regardless of `cols::` settings.
- **AC-07**: All default-on columns removed via `cols::` → bare name lines only; no blank-line separators.
- **AC-08**: Accounts listed alphabetically by name.
- **AC-09**: `cols::+display_name` shows `Display:` line per account from saved `{name}.json`.
- **AC-10**: `cols::+role` shows `Role:` line with the user-defined label from `{name}.json` `role` field; `cols::+billing`, `cols::+model` show corresponding lines per account from saved snapshots.
- **AC-11**: Accounts without saved metadata files show `N/A` for `email`, `display_name`, `host`, `role`, `billing`, `model`.
- **AC-12**: `format::json` includes `email`, `display_name`, `role`, `billing`, `model`, `tagged_id`, `capabilities`, `organization_uuid`, `organization_name`, `organization_role`, `workspace_uuid`, `workspace_name`, `host`, `owner`, `is_owned`, `renewal_at` keys per account object.
- **AC-13**: `Current:  yes` is shown for the account whose `accessToken` matches `~/.claude/.credentials.json`; `Current:  no` for all others. See [016_current_account_awareness.md](016_current_account_awareness.md).
- **AC-14**: `cols::-current` suppresses the `Current:` line; the line is also suppressed when `~/.claude/.credentials.json` is unreadable.
- **AC-15**: `format::json` includes `is_current` boolean field per account object.
- **AC-16**: `cols::+uuid` shows `ID:` line from `tagged_id` field; `N/A` when absent from snapshot.
- **AC-17**: `cols::+capabilities` shows `Capabilities:` line as comma-separated list; `N/A` when absent or empty.
- **AC-18**: `cols::+org_uuid` shows `Org ID:` line from `{name}.json`; `N/A` when absent.
- **AC-19**: `cols::+org_name` shows `Org:` line from `{name}.json`; `N/A` when absent.
- **AC-20**: `format::json` includes `owner` (string) and `is_owned` (boolean) fields per account object; `owner` is empty string when unowned; `is_owned` is `true` when owner is empty or matches current machine identity.
- **AC-21**: `format::json` includes `renewal_at` field per account object; absent (omitted or `null`) when `_renewal_at` is not set in `{name}.json`.

### Commands

| File | Relationship |
|------|--------------|
| [command/001_account.md](../cli/command/001_account.md#command-3-accounts) | CLI command specification |

### Features

| File | Relationship |
|------|--------------|
| [016_current_account_awareness.md](016_current_account_awareness.md) | Current-account detection algorithm and `Current:` field additions |
| [021_extended_snapshot_fields.md](021_extended_snapshot_fields.md) | `tagged_id`, `uuid`, `capabilities` fields and `uuid::`, `capabilities::` params |
| [022_org_identity_snapshot.md](022_org_identity_snapshot.md) | Org identity fields and `org_uuid::`, `org_name::` params |
| [029_account_host_metadata.md](029_account_host_metadata.md) | `host` and `role` user-defined label fields; `cols::+host`/`cols::+role` opt-in columns |
| [036_account_ownership.md](036_account_ownership.md) | `owner` and `is_owned` fields; ownership gate logic |
| [030_account_renewal_override.md](030_account_renewal_override.md) | `renewal_at` field and `_renewal_at` JSON key in `{name}.json` |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.accounts`](../cli/command/001_account.md#command-3-accounts) | CLI surface for this feature |

### Sources

| File | Relationship |
|------|--------------|
| `src/account.rs` | `list()` — enumerates credential store, reads per-machine active marker |
| `src/commands/accounts.rs` | `accounts_routine()` — CLI handler |

### Tests

| File | Relationship |
|------|--------------|
| [tests/docs/cli/command/03_accounts.md](../../tests/docs/cli/command/03_accounts.md) | Integration test plan |
