# Feature: Live Credentials Status

### Scope

- **Purpose**: Show credential metadata on any authenticated machine without requiring account store setup.
- **Responsibility**: Documents the `.credentials.status` command and `credentials_routine()` (FR-17).
- **In Scope**: Direct credentials file read, no account store dependency, verbosity levels, N/A handling.
- **Out of Scope**: Account-store-aware status (→ 011_account_status_by_name.md), OAuth refresh (forbidden — NFR-5).

### Design

`.credentials.status` must read `~/.claude/.credentials.json` directly with no dependency on:
- `~/.claude/accounts/_active` marker
- `~/.claude/accounts/` directory
- Any account store setup

This command succeeds on a fresh Claude Code installation where only `~/.claude/.credentials.json` exists.

**Verbosity levels:**
- `v::0`: subscription type + token state only
- `v::1` (default): + rate limit tier, email (from `~/.claude/.claude.json`), org
- `v::2`: + token expiry time (`Expires: in Xh Ym`)

**`format::json`:** Returns `{"subscription":"…","tier":"…","token":"…","expires_in_secs":N}`.

**Missing fields:** Email and org show `N/A` when `~/.claude/.claude.json` is absent or the fields are empty.

**Absent credentials file:** Exit non-zero (exit 2) with an actionable error naming the full path to `~/.claude/.credentials.json`.

**Must NOT call:** `account::list()`, read `_active`, or scan `accounts/` directory.

### Acceptance Criteria

- **AC-01**: `.credentials.status` exits 0 on a machine with only `~/.claude/.credentials.json` (no `accounts/` dir).
- **AC-02**: `v::2` shows subscription, tier, token state, expiry time, email, org.
- **AC-03**: `format::json` returns valid JSON with all four fields.
- **AC-04**: Absent `~/.claude/.credentials.json` exits 2 with error naming the file path.
- **AC-05**: Missing or empty email/org fields → shown as `N/A`.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/commands.rs` | `credentials_routine()` — reads credentials directly, no account store calls |
| test | `tests/credentials_test.rs::cred01–cred05` | No-account-store, all verbosity levels, JSON, N/A cases |
| doc | [011_account_status_by_name.md](011_account_status_by_name.md) | Related: account-store-aware status command |
| doc | [cli/commands.md](../cli/commands.md#command--11-credentialsstatus) | CLI command specification |
| doc | [cli/testing/command/credentials_status.md](../cli/testing/command/credentials_status.md) | Manual integration tests |
