# Parameter :: 52. `role::` (metadata label)

Specifies a user-defined role label to store in the account profile at `.account.save` time. Displayed via `cols::+role` in `.usage`.

- **Type:** `string`
- **Default:** `""` (empty; field stored as `""` in `{name}.profile.json`)
- **Constraints:** Any string; no format validation
- **Commands:** [`.account.save`](../command/001_account.md#command--4-accountsave)
- **Purpose:** Tag a saved account with an organizational or contextual label (e.g., `work`, `personal`, `dev`, `staging`).
- **Group:** Account Targeting

**Note:** This is distinct from [param 015 `role::`](015_role.md), which is a boolean display toggle for the org role field from `.accounts` / `.credentials.status`. This parameter is a free-text metadata label written to `{name}.profile.json`.

**Behavior:** The value is written to the `role` field in `{name}.profile.json` alongside `host::`. It persists until `.account.save` is re-run with a different `role::` value. An empty string stores `""` (not omitted). Displayed in `.usage` when `cols::+role` is active.

**Examples:**

```text
clp .account.save                                   -> role stored as "" (empty)
clp .account.save role::work                        -> role stored as "work"
clp .account.save host::workstation role::personal  -> host "workstation", role "personal"
```

**See Also:** [feature/029_account_host_metadata.md](../../feature/029_account_host_metadata.md) for profile storage and display. [048_host.md](048_host.md) for the companion host label parameter.
