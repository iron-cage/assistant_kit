# Parameter: 14. `display_name::`

Controls whether the display name line appears in output. Opt-in (default `0`). Source: `displayName` field in `oauthAccount` — read from live `~/.claude.json` (`.credentials.status`). No longer usable on `.accounts` — removed in [Feature 037](../../feature/037_accounts_usage_param_unification.md); `.accounts` now rejects `display_name::` with `parameter 'display_name' removed — use 'cols::+display_name' instead` (see `REMOVED_TOGGLES` in `src/commands/accounts.rs`).

- **Default:** `0` (hidden)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Purpose:** Exposes the human-readable display name set by the OAuth account. Shows `N/A` when the source file is absent or the field is missing.

**Examples:**

```text
display_name::0   → line omitted  (default)
display_name::1   → Display: alice
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
| 1 | [`.credentials.status`](../command/002_credentials.md#command-10-credentialsstatus) | Display name line for live credentials |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Human-readable name during account management |
| 2 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Selective field display in diagnostics |
