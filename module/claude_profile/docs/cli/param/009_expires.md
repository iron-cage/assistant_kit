# Parameter: 9. `expires::`

Controls whether the token expiry duration line appears in output. Used by `.credentials.status` (from live credentials). No longer usable on `.accounts` — removed in [Feature 037](../../feature/037_accounts_usage_param_unification.md); `.accounts` now rejects `expires::` with `parameter 'expires' removed — use 'cols::-expires' instead` (see `REMOVED_TOGGLES` in `src/commands/accounts.rs`).

- **Default:** `1` (shown)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Purpose:** Allows suppression of the expiry countdown when exact timing is not needed.

**Examples:**

```text
expires::1   → Expires: in 7h 24m  (default)
expires::0   → line omitted
```

### Referenced Type

- **Fundamental Type:** `bool`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Field Presence](../param_group/002_field_presence.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.credentials.status`](../command/002_credentials.md#command-10-credentialsstatus) | Expiry duration line for live credentials |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Expiry timing during account management |
| 2 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Token expiry in diagnostic output |
