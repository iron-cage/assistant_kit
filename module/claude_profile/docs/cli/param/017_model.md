# Parameter: 17. `model::`

Controls whether the active model line appears in output. Opt-in (default `0`). Source: `model` field in `settings.json` — read from live `~/.claude/settings.json` for `.credentials.status`. For `.accounts`, read from `{name}.json` per-account snapshot (captured by `save()` — BUG-222 fix); shows `N/A` when snapshot absent or `model` field missing.

- **Default:** `0` (hidden)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Purpose:** Shows the model currently selected in Claude Code settings. Shows `N/A` when the source file is absent or the `model` field is missing.

**Examples:**

```text
model::0   → line omitted  (default)
model::1   → Model:   sonnet
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
| 1 | [`.accounts`](../command/001_account.md#command--3-accounts) | Active model line per stored account |
| 2 | [`.credentials.status`](../command/002_credentials.md#command--10-credentialsstatus) | Active model line for live credentials |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Model setting context during account management |
| 2 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Selective field display in diagnostics |
