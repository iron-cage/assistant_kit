# Parameter: 52. `role::` (metadata label)

Specifies a user-defined role label to store in the account profile at `.account.save` time. Displayed via `cols::+role` in `.usage`.

- **Default:** `""` (empty; field stored as `""` in `{name}.json`)
- **Constraints:** Any string; no format validation
- **Purpose:** Tag a saved account with an organizational or contextual label (e.g., `work`, `personal`, `dev`, `staging`).

**Note:** This is distinct from [param 015 `role::`](015_role.md), which is a boolean display toggle for the org role field from `.accounts` / `.credentials.status`. This parameter is a free-text metadata label written to `{name}.json`.

**Behavior:** The value is written to the `role` field in `{name}.json` alongside `host::`. It persists until `.account.save` is re-run with a different `role::` value. An empty string stores `""` (not omitted). Displayed in `.usage` when `cols::+role` is active.

**Examples:**

```text
clp .account.save                                   -> role stored as "" (empty)
clp .account.save role::work                        -> role stored as "work"
clp .account.save host::workstation role::personal  -> host "workstation", role "personal"
```

**See Also:** [feature/029_account_host_metadata.md](../../feature/029_account_host_metadata.md) for profile storage and display. [048_host.md](048_host.md) for the companion host label parameter.

### Referenced Type

- **Fundamental Type:** `string`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Account Targeting](../param_group/006_account_targeting.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.save`](../command/001_account.md#command-4-accountsave) | Write role metadata label to account profile |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Contextual role label during account profile creation |
