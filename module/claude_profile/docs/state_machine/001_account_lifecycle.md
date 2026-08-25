# State Machine: Account Lifecycle

### Scope

- **Purpose**: Define the lifecycle states and transitions for accounts in the credential store.
- **Responsibility**: Documents `absent`/`saved`/`active` states, transition triggers, and cross-machine active-marker cleanup on delete.
- **In Scope**: Account state transitions; `_active_{host}_{user}` marker semantics; multi-machine concurrency.
- **Out of Scope**: OAuth token state (→ state_machine/002); credential file format (→ schema/001).

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
[saved]  --account.delete--> [absent] (no active marker to clear)
[active] --account.delete--> [absent] (clears every _active_* marker across all machines naming this account — Fix(BUG-347))
[absent] → [absent]  (account.delete on absent = no-op)
```

### Delete Behavior

`.account.delete` deletes unconditionally, regardless of whether the account is active on any machine — there is no refusal guard. When the deleted account is active (on the calling machine or any other), every `_active_{host}_{user}` marker file naming it is also removed, not only the calling machine's own marker (Fix BUG-347: the pre-fix implementation resolved and checked only the calling machine's own marker path, leaving foreign-machine markers naming the same account orphaned). This can leave one or more machines with no active account; a subsequent `.account.use` or `.account.save` on each affected machine restores one.

### Multi-Machine Note

"Active" is per-machine. Account `A` can be `active` on machine `devbox` and `saved` on machine `buildbox` simultaneously. Each machine has its own `_active_{host}_{user}` marker. See [schema/005](../schema/005_active_marker.md).

### Behavioral Invariants

- An account can be deleted regardless of active state on any machine — `.account.delete` has no refusal guard.
- Deleting an account clears every `_active_*` marker across all machines that names it, not only the calling machine's own marker (Fix BUG-347).
- A `saved` account's `{name}.json` data is preserved (read-merged) on re-save — no data loss on snapshot update.
- "Active" is per-machine — multiple machines may each hold a different account as `active` simultaneously.

### Features

| File | Relationship |
|------|-------------|
| [feature/002_account_save.md](../feature/002_account_save.md) | `.account.save` transitions |
| [feature/004_account_use.md](../feature/004_account_use.md) | `.account.use` transitions |
| [feature/005_account_delete.md](../feature/005_account_delete.md) | `.account.delete` guard |

### Schema

| File | Relationship |
|------|-------------|
| [schema/005](../schema/005_active_marker.md) | Active marker format |
