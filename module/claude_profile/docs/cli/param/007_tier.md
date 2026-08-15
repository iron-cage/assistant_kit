# Parameter: 7. `tier::`

Controls whether the rate-limit tier line appears in output. Used by `.credentials.status` (from live credentials). No longer usable on `.accounts` — removed in [Feature 037](../../feature/037_accounts_usage_param_unification.md); `.accounts` now rejects `tier::` with `parameter 'tier' removed — use 'cols::-tier' instead` (see `REMOVED_TOGGLES` in `src/commands/accounts.rs`).

- **Default:** `1` (shown)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Purpose:** Allows suppression of the tier when only core token state is needed.

**Examples:**

```text
tier::1   → Tier:    default_claude_max_20x  (default)
tier::0   → line omitted
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
| 1 | [`.credentials.status`](../command/002_credentials.md#command-10-credentialsstatus) | Rate-limit tier line for live credentials |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Tier context during account management |
| 2 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Selective field display in diagnostics |
