# Verb: rotate *(DEPRECATED — Feature 038)*

> **DEPRECATED** — The `rotate` verb and `.account.rotate` command are retained only as a hidden redirector stub that always exits 1 with a migration notice; the rotation logic described below has been removed from the live code path. Use `clp .usage rotate::1` (with optional `sort::` strategy) instead. See [feature/038_usage_strategy_rotate.md](../../feature/038_usage_strategy_rotate.md).

Selects the inactive saved account with the highest `expiresAt` timestamp and activates it. Designed for automated rotation workflows where maintaining a valid active session is critical. Skips the currently active account and any accounts with exhausted or expired tokens.

### Nouns

| # | Noun | Command | Idempotent | Requires Session |
|---|------|---------|-----------|-----------------|
| 1 | [account](../command_noun/001_account.md) | `.account.rotate` | No | No |

### Behavioral Contract

**Pre-conditions:**
- At least one inactive saved account exists in the credential store with a non-expired token
- `$HOME` environment variable set

**Post-conditions:**
- Account with the highest `expiresAt` among inactive accounts is now active
- `~/.claude/.credentials.json` updated atomically
- Per-machine active marker updated

**Side effects:**
- Overwrites `~/.claude/.credentials.json` with selected account's credentials (atomic)
- Prior active account transitions to saved state

### Idempotency

**No.** The selected account depends on current `expiresAt` values across all inactive accounts. As tokens expire, repeated calls may select different accounts. The outcome changes with time.

### Common Parameters

| Parameter | Semantics | Required |
|-----------|-----------|----------|
| `dry::` | Report which account would be selected without switching | No |
| `trace::` | Emit diagnostic trace output | No |

### State Transition Pattern

**Transitions state.** Selects the best inactive account by token expiry and activates it via the same atomic switch mechanism used by `use`.

```
[saved]  --account.rotate (selected)--> [active]
[active] --account.rotate (prior)-----> [saved]
```

### Migration (Feature 038)

| Old | New | Notes |
|-----|-----|-------|
| `clp .account.rotate` | `clp .usage rotate::1` | Default `sort::renew` (soonest renewal). Former default was `max_by_key(expires_at_ms)`. |
| `clp .account.rotate dry::1` | `clp .usage rotate::1 dry::1` | Same semantics. |
| `clp .account.rotate trace::1` | `clp .usage rotate::1 trace::1` | Same semantics. |
| *(no equivalent)* | `clp .usage rotate::1 sort::renews` | New: rotate to account with soonest billing renewal. |

### See Also

| File | Relationship |
|------|-------------|
| [feature/008_auto_rotate.md](../../feature/008_auto_rotate.md) | Auto-rotation selection algorithm and trigger conditions |
| [feature/038_usage_strategy_rotate.md](../../feature/038_usage_strategy_rotate.md) | Replacement feature — `.usage rotate::1`; see Migration table above |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.rotate`](../command/001_account.md#command-13-accountrotate-deprecated-feature-038) | Auto-rotate to best inactive account by token expiry |
