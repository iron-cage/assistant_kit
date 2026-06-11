# Verb :: renewal

Sets or clears the billing renewal timestamp override (`_renewal_at`) for a named account. The override is stored in `{name}.json` via read-merge write and is used to compute the `~Renews` display field shown in `.accounts` output. Supports absolute timestamp (`at::`), relative duration (`from_now::`), or explicit clear (`clear::`).

### Nouns

| # | Noun | Command | Idempotent | Requires Session |
|---|------|---------|-----------|-----------------|
| 1 | [account](../command_noun/001_account.md) | `.account.renewal` | Yes | No |

### Behavioral Contract

**Pre-conditions:**
- Named account exists in credential store
- Exactly one of `at::`, `from_now::`, or `clear::` must be provided
- `$HOME` environment variable set

**Post-conditions:**
- `_renewal_at` field in `{name}.json` updated to specified timestamp, or removed if `clear::1`
- `~Renews` display for this account reflects the updated value

**Side effects:**
- Read-merge write to `{name}.json` — all existing fields preserved; only `_renewal_at` modified
- Monthly auto-advance: if `from_now::` is used, the stored date may be advanced automatically on each billing cycle

### Idempotency

**Yes.** Setting the same `_renewal_at` value produces identical stored state. Clearing an already-absent `_renewal_at` field is also a no-op. Repeated calls with the same parameters converge.

### Common Parameters

| Parameter | Semantics | Required |
|-----------|-----------|----------|
| `name::` | Account to update (email or unambiguous prefix) | Yes |
| `at::` | Set renewal to absolute ISO-8601 UTC timestamp | Conditional |
| `from_now::` | Set renewal to current time plus signed duration delta | Conditional |
| `clear::` | Remove `_renewal_at` override | Conditional |
| `dry::` | Validate without writing | No |
| `trace::` | Emit diagnostic trace output | No |

*Exactly one of `at::`, `from_now::`, or `clear::` is required.*

### State Transition Pattern

**Accumulates state.** Updates a single metadata field (`_renewal_at`) in `{name}.json` via read-merge. Account lifecycle state unchanged.

```
[saved/active] --account.renewal at::TS--> [saved/active]  (_renewal_at updated)
[saved/active] --account.renewal clear::1-> [saved/active]  (_renewal_at removed)
```

### Cross-References

| File | Relationship |
|------|-------------|
| [feature/030_account_renewal_override.md](../../feature/030_account_renewal_override.md) | `_renewal_at` storage, monthly auto-advance, and `~Renews` rendering |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.renewal`](../command/001_account.md#command--14-accountrenewal) | Set or clear billing renewal timestamp override |
