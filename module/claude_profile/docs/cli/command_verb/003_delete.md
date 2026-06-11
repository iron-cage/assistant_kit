# Verb :: delete

Removes an account profile from the credential store. Deletes `{name}.credentials.json`, `{name}.json`, and any legacy satellite files associated with the named account. If the target account is currently active, the active marker is also cleared.

### Nouns

| # | Noun | Command | Idempotent | Requires Session |
|---|------|---------|-----------|-----------------|
| 1 | [account](../command_noun/001_account.md) | `.account.delete` | Conditional | No |

### Behavioral Contract

**Pre-conditions:**
- Named account exists in credential store
- `$HOME` environment variable set

**Post-conditions:**
- `{name}.credentials.json` removed from credential store
- `{name}.json` removed from credential store
- Legacy satellite files (if any) removed
- If target was the active account, `~/.claude/.credentials.json` is no longer associated with a named profile

**Side effects:**
- File deletion is permanent — no backup or archive created
- If the deleted account was active, the active marker is cleared; no automatic failover to another account

### Idempotency

**Conditional.** Deleting an already-absent account exits with code 2 (not found). When `dry::1`, a non-existent account is reported without error. Re-deleting a successfully deleted account is therefore not idempotent in the default mode.

### Common Parameters

| Parameter | Semantics | Required |
|-----------|-----------|----------|
| `name::` | Account to delete (email or unambiguous prefix) | Yes |
| `dry::` | Validate target exists without deleting | No |
| `trace::` | Emit diagnostic trace output | No |

### State Transition Pattern

**Removes state.** Deletes all files associated with the named account from the per-machine credential store.

```
[saved]  --account.delete--> [absent]
[active] --account.delete--> [absent]  (active marker also cleared)
```

### Cross-References

| File | Relationship |
|------|-------------|
| [feature/005_account_delete.md](../../feature/005_account_delete.md) | File removal sequence and legacy satellite cleanup |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.delete`](../command/001_account.md#command--6-accountdelete) | Remove account profile from credential store |
