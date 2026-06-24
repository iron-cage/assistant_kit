# Verb :: assign

Writes the per-machine active marker (`_active_{machine}_{user}`) for a named account — without rotating credentials or modifying the `owner` field. Unlike `use`, `assign` does not touch `~/.claude/.credentials.json`. It is the canonical command for marker assignment only; ownership is managed by `.account.save` (see Feature 036).

### Nouns

| # | Noun | Command | Idempotent | Requires Session |
|---|------|---------|-----------|-----------------|
| 1 | [account](../command_noun/001_account.md) | `.account.assign` | Yes | No |

### Behavioral Contract

**Pre-conditions:**
- Named account (or account resolved via `for::`) exists in credential store
- `$HOME` environment variable set

**Post-conditions:**
- Per-machine active marker file (`_active_{machine}_{user}`) written to credential store pointing to the named account
- `~/.claude/.credentials.json` unchanged
- `{name}.json` NOT modified — `owner` field and all other fields left untouched
- No credential rotation performed

**Side effects:**
- Writes marker file only; no other files modified

### Idempotency

**Yes.** Writing the same active marker repeatedly produces identical stored state. Repeated calls with the same `name::` and `for::` values are safe.

### Common Parameters

| Parameter | Semantics | Required |
|-----------|-----------|----------|
| `name::` | Account to mark as active; defaults to active account | No |
| `for::` | Target `USER@MACHINE` composite for the marker (default: current) | No |
| `dry::` | Validate without writing marker | No |

### State Transition Pattern

**Accumulates state.** Writes the per-machine active marker file only. No credential rotation; `~/.claude/.credentials.json` is not modified. `{name}.json` is not modified. The account's lifecycle state (saved/active) is unchanged.

```
[absent/saved/active] --account.assign--> [same state]  (marker written; {name}.json unchanged; credentials unchanged)
```

### Migration (Feature 037 + Feature 064)

> `.account.assign` has been removed as a standalone working command (Feature 037). Its behavior was absorbed into `.accounts`/`.usage` as `assign::1` + `for::`. Feature 064 replaced both with `active::USER@MACHINE name::X`.
> - `clp .account.assign name::X` → exits 1 with generic "unknown command" error (fully deregistered)
> - `clp .accounts assign::1 name::X` → exits 1 with REMOVED_TOGGLE migration message (Feature 064)
> - `clp .accounts active::$USER@$HOSTNAME name::X` → writes marker for current machine (current behavior)
> - `clp .accounts active::bob@laptop name::X` → writes marker for specific machine (current behavior)
> - `clp .accounts active::user1@w003` (no `name::`) → clears/unassigns the marker (Feature 064)
>
> See [feature/064_active_marker_and_owner_redesign.md](../../feature/064_active_marker_and_owner_redesign.md) AC-01 through AC-07.

### Cross-References

| File | Relationship |
|------|-------------|
| [feature/032_account_assign.md](../../feature/032_account_assign.md) | Marker write, `for::` resolution, sanitization rules |
| [feature/025_per_machine_active_marker.md](../../feature/025_per_machine_active_marker.md) | `_active_{machine}_{user}` marker semantics |
| [feature/036_account_ownership.md](../../feature/036_account_ownership.md) | Ownership model; enforcement gates G1–G8; ownership is stamped by `.account.save`, not `.account.assign` |
| [feature/037_accounts_usage_param_unification.md](../../feature/037_accounts_usage_param_unification.md) | `assign::` absorbed as mutation param; `.account.assign` standalone removed; `for::` and `assign::` further REMOVED in Feature 064 — use `active::USER@MACHINE` |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.assign`](../command/001_account.md#command--16-accountassign) | Write per-machine active marker without credential rotation (removed in Feature 037; `assign::1` further REMOVED in Feature 064 — use `.accounts active::USER@MACHINE name::X`) |
