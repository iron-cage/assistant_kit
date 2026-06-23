# Parameter :: 58. `force::`

Bypasses ownership enforcement gates on mutation commands. When `force::1`, the G5–G8 ownership check is skipped regardless of whether `current_identity()` matches the stored owner field.

- **Default:** `0` (enforce ownership gates normally)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Purpose:** Allows any machine/user identity to execute account mutations (use, delete, relogin, unclaim, owner set) on accounts owned by a different identity. Intended for administrative recovery and cross-machine management scenarios.

**Bypass scope:**

| Gate | Command | With `force::1` |
|------|---------|-----------------|
| G5 | `.account.use` | Ownership check skipped; proceeds to `switch_account()` |
| G6 | `.account.delete` | Ownership check skipped; proceeds to deletion |
| G7 | `.account.relogin` | Ownership check skipped; proceeds to 6-step relogin |
| G8 | `.accounts unclaim::1` | Ownership check skipped; clears owner field |
| G8 | `.accounts owner::VALUE` | Ownership check skipped; sets owner to VALUE (Feature 063) |

**No bypass for read-side gates:** `force::1` does not affect G1–G4 (fetch, refresh, touch suppression). Non-owned accounts continue to use cache-as-primary for quota reads regardless of `force::`.

**Examples:**

```text
force::1     → bypass ownership gate; proceed with mutation
force::0     → enforce normally (default)
force::true  → same as force::1
```

**Notes:**
- `force::1` without a mutation (no `unclaim::1`, no account switch in progress) is silently ignored.
- When combined with `dry::1`: ownership gate is bypassed, but the mutation is still previewed without writing — `[dry-run]` message is printed, exits 0.
- Ownership of the account is NOT changed by `force::` itself. Use `unclaim::1` to clear ownership.

### Referenced Type

- **Fundamental Type:** `bool`

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.use`](../command/001_account.md#command--5-accountuse) | Bypass G5 ownership guard |
| 2 | [`.account.delete`](../command/001_account.md#command--6-accountdelete) | Bypass G6 ownership guard |
| 3 | [`.account.relogin`](../command/001_account.md#command--12-accountrelogin) | Bypass G7 ownership guard |
| 4 | `.accounts` | Bypass G8 when used with `unclaim::1` (Feature 037) |

### See Also

- [feature/036_account_ownership.md](../../feature/036_account_ownership.md) — G1–G8 ownership enforcement gates; force:: bypass design
- [feature/037_accounts_usage_param_unification.md](../../feature/037_accounts_usage_param_unification.md) — force:: as unified param on `.accounts`/`.usage`
