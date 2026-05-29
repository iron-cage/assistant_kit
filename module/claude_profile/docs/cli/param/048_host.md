# Parameter :: 48. `host::`

Specifies the host/machine label to store in the account profile at `.account.save` time. Displayed via `cols::+host` in `.usage`.

- **Type:** `string`
- **Default:** `""` (auto-captured from `$USER@$HOSTNAME`)
- **Constraints:** Any non-empty string; empty string triggers auto-capture
- **Commands:** [`.account.save`](../command/001_account.md#command--4-accountsave)
- **Purpose:** Tag a saved account with the machine/user context where it was saved.
- **Group:** Account Targeting

**Behavior:** When `host::` is omitted or empty, the value is auto-captured as `$USER@$HOSTNAME` at save time. When provided, the explicit value overrides auto-capture. The value is written to `{name}.profile.json` and persists until the next `save()` call with a different `host::` value.

**Examples:**

```text
clp .account.save                         -> host auto-captured as "$USER@$HOSTNAME"
clp .account.save host::workstation       -> host stored as "workstation"
clp .account.save host::laptop role::dev  -> host "laptop", role "dev"
```

**See Also:** [feature/029_account_host_metadata.md](../../feature/029_account_host_metadata.md) for profile storage and display.
