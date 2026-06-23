# State Machine: Account Lifecycle

### States

| State | Description | Marker file exists? | `{name}.credentials.json`? |
|-------|-------------|--------------------|-----------------------------|
| `absent` | Account not in credential store | — | No |
| `saved` | Account saved, not the active account on this machine | No own marker | Yes |
| `active` | Account is the active account on this machine | `_active_{host}_{user}` = `{name}` | Yes |

### Transitions

```
[absent] --account.save--> [saved]
[saved]  --account.save--> [saved]    (credential snapshot updated; {name}.json read-merged)
[saved]  --account.use---> [active]   (credentials written to live; marker written)
[active] --account.save--> [active]   (re-saved; no lifecycle change)
[active] --account.use other---> [saved]  (marker overwritten with new name; this account → saved)
[saved]  --account.delete--> [absent] (guard: cannot delete active account)
[absent] → [absent]  (account.delete on absent = no-op)
```

### Safety Guard

`.account.delete` refuses to delete the active account (the account whose name matches the current machine's `_active_{host}_{user}` marker). The account must first be switched away from (`account.use`) before deletion.

### Multi-Machine Note

"Active" is per-machine. Account `A` can be `active` on machine `w003` and `saved` on machine `w004` simultaneously. Each machine has its own `_active_{host}_{user}` marker. See [schema/005](../schema/005_active_marker.md).

### Cross-References

| File | Relationship |
|------|-------------|
| [feature/002_account_save.md](../feature/002_account_save.md) | `.account.save` transitions |
| [feature/004_account_use.md](../feature/004_account_use.md) | `.account.use` transitions |
| [feature/005_account_delete.md](../feature/005_account_delete.md) | `.account.delete` guard |
| [schema/005](../schema/005_active_marker.md) | Active marker format |
