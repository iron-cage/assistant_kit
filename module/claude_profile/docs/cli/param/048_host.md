# Parameter :: 48. `host::`

Specifies the host/machine label to store in the account profile at `.account.save` time. Displayed via `cols::+host` in `.usage`.

- **Default:** `""` (auto-captured from `$USER@<hostname>` via `resolve_hostname()` fallback chain)
- **Constraints:** Any non-empty string; empty string triggers auto-capture
- **Purpose:** Tag a saved account with the machine/user context where it was saved.

**Behavior:** When `host::` is omitted or empty, the value is auto-captured as `$USER@<hostname>` at save time, where hostname is resolved via `resolve_hostname()` (`$HOSTNAME` env → `/etc/hostname` → `"local"`). When provided, the explicit value overrides auto-capture. The value is written to `{name}.json` and persists until the next `save()` call with a different `host::` value.

**Examples:**

```text
clp .account.save                         -> host auto-captured as "$USER@<hostname>" (hostname via fallback chain)
clp .account.save host::workstation       -> host stored as "workstation"
clp .account.save host::laptop role::dev  -> host "laptop", role "dev"
```

**Note:** `host::` is a display label only — it does not control ownership or access enforcement. The `owner` field in `{name}.json` (always auto-captured from `current_identity()`, never user-specified) is what governs which identity may operate on the account's credentials. See [feature/036_account_ownership.md](../../feature/036_account_ownership.md).

**See Also:** [feature/029_account_host_metadata.md](../../feature/029_account_host_metadata.md) for profile storage and display.

### Referenced Type

- **Fundamental Type:** `string`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Account Targeting](../param_group/006_account_targeting.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.save`](../command/001_account.md#command--4-accountsave) | Write host metadata to account profile |
| 2 | [`.accounts`](../command/001_account.md#command--3-accounts) | Display host column when `cols::+host` |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Machine context tag during account profile creation |
