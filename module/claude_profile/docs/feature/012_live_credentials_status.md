# Feature: Live Credentials Status

### Scope

- **Purpose**: Show credential metadata on any authenticated machine without requiring account store setup.
- **Responsibility**: Documents the `.credentials.status` command and `credentials_status_routine()` (FR-17).
- **In Scope**: Direct credentials file read, no account store dependency, per-field presence params, N/A handling.
- **Out of Scope**: Account-store-aware status (→ 011_account_status_by_name.md), OAuth refresh (forbidden — NFR-5).

### Design

`.credentials.status` reads `~/.claude/.credentials.json` directly and succeeds on any machine where only that file is present — no hard dependency on:
- The credential store directory
- Any account store setup

The `_active` marker is read opportunistically for the `Account:` line if it exists; the command still succeeds and shows `N/A` when it is absent (e.g. a machine where no account has ever been saved).

**Field Presence Parameters:**

Each output line is independently controlled by a boolean param. All default to `1` (shown) except `file::` and `saved::` which are opt-in (`0` by default).

| Param | Default | Output Line |
|-------|---------|-------------|
| `account::` | `1` | `Account: {active_account_or_N/A}` (from `_active` marker if present) |
| `sub::` | `1` | `Sub:     {subscriptionType}` |
| `tier::` | `1` | `Tier:    {rateLimitTier}` |
| `token::` | `1` | `Token:   valid / expiring in Xm / expired` |
| `expires::` | `1` | `Expires: in Xh Ym` |
| `email::` | `1` | `Email:   {emailAddress_or_N/A}` |
| `file::` | `0` | `File:    {credentials_file_path}` (opt-in) |
| `saved::` | `0` | `Saved:   N account(s)` (opt-in, counts `*.credentials.json`) |

**`format::json`:** Returns all fields regardless of field-presence params:
`{"subscription":"…","tier":"…","token":"…","expires_in_secs":N,"email":"…","account":"…","file":"…","saved":N}`.

**`Account:` line:** Reads `_active` marker if it exists. Shows `N/A` when no `_active` marker is present (fresh install or uninitialised account store). Because `.account.save` writes `_active` on every successful save, the account name is always present after any save operation.

**Missing fields:** Email and org show `N/A` when `~/.claude.json` is absent or the fields are empty.

**Absent credentials file:** Exit non-zero (exit 2) with an actionable error naming the full path to `~/.claude/.credentials.json`.

**Must NOT call:** `account::list()` or scan the credential store (reading `_active` is permitted for the `account::` line only).

### Acceptance Criteria

- **AC-01**: `.credentials.status` exits 0 on a machine with only `~/.claude/.credentials.json` (no credential store).
- **AC-02**: Default output (no params) shows all 6 default-on fields: account, sub, tier, token, expires, email.
- **AC-03**: `format::json` returns valid JSON with subscription, tier, token, expires_in_secs, email, account, file, saved.
- **AC-04**: Absent `~/.claude/.credentials.json` exits 2 with error naming the file path.
- **AC-05**: Missing or empty email and absent `_active` marker → shown as `N/A`.
- **AC-06**: `sub::0 tier::0 expires::0 email::0 account::0` → only Token line shown.
- **AC-07**: `file::1 saved::1` → File and Saved lines appended after default-on fields.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/commands.rs` | `credentials_status_routine()` — reads credentials directly, no account store calls |
| test | `tests/cli/credentials_test.rs::cred01–cred07` | Account-store independence, field presence, JSON, N/A cases |
| doc | [011_account_status_by_name.md](011_account_status_by_name.md) | Related: account-store-aware status command |
| doc | [cli/commands.md](../cli/commands.md#command--11-credentialsstatus) | CLI command specification |
| doc | [tests/docs/cli/command/10_credentials_status.md](../../tests/docs/cli/command/10_credentials_status.md) | Test case planning |
