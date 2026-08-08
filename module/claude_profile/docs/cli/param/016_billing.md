# Parameter: 16. `billing::`

Controls whether the billing type line appears in output. Opt-in (default `0`). Source: `billingType` field in `oauthAccount` — read from live `~/.claude.json` (`.credentials.status`). No longer usable on `.accounts` — removed in [Feature 037](../../feature/037_accounts_usage_param_unification.md); `.accounts` now rejects `billing::` with `parameter 'billing' removed — use 'cols::+billing' instead` (see `REMOVED_TOGGLES` in `src/commands/accounts.rs`).

- **Default:** `0` (hidden)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Purpose:** Shows the raw billing type string (e.g., `stripe_subscription`). Shows `N/A` when the source file is absent or the field is missing.

**Examples:**

```text
billing::0   → line omitted  (default)
billing::1   → Billing: stripe_subscription
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
| 1 | [`.credentials.status`](../command/002_credentials.md#command-10-credentialsstatus) | Billing type line for live credentials |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Billing context during account management |
| 2 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Selective field display in diagnostics |
