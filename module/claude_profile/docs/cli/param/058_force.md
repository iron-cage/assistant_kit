# Parameter: 58. `force::`

Bypasses ownership enforcement gates on mutation commands. When `force::1`, the G5–G8 ownership check is skipped regardless of whether `current_identity()` matches the stored owner field.

- **Default:** `0` (enforce ownership gates normally)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Purpose:** Allows any machine/user identity to execute account mutations (use, delete, relogin, owner release via `owner::0`, owner set via `owner::USER@MACHINE`) on accounts owned by a different identity. Intended for administrative recovery and cross-machine management scenarios.

**Bypass scope:**

| Gate | Command | With `force::1` |
|------|---------|-----------------|
| G5 | `.account.use` | Ownership check skipped; proceeds to `switch_account()` |
| G6 | `.account.delete` | Ownership check skipped; proceeds to deletion |
| G7 | `.account.relogin` | Ownership check skipped; proceeds to 6-step relogin |
| G8 | `.accounts owner::0 name::X` | Ownership check skipped; clears owner field (Feature 064) |
| G8 | `.accounts owner::VALUE` | Ownership check skipped; sets owner to VALUE (Feature 063) |

**No bypass for read-side gates:** `force::1` does not affect G1–G4 (fetch, refresh, touch suppression). Non-owned accounts continue to use cache-as-primary for quota reads regardless of `force::`.

**No effect on `assignee::` marker writes:** `force::1` is silently ignored when used with `assignee::USER@MACHINE` — marker writes have no ownership gate (Feature 065). `force::` applies only to the ownership gates G5–G8 listed above.

**Examples:**

```text
force::1     → bypass ownership gate; proceed with mutation
force::0     → enforce normally (default)
force::true  → same as force::1
```

**Notes:**
- `force::1` without a mutation (no `owner::0`/`owner::USER@MACHINE`, no account switch in progress) is silently ignored.
- When combined with `dry::1`: ownership gate is bypassed, but the mutation is still previewed without writing — `[dry-run]` message is printed, exits 0.
- Ownership of the account is NOT changed by `force::` itself. Use `owner::0 name::X` to clear ownership (Feature 064).

### Referenced Type

- **Fundamental Type:** `bool`

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.use`](../command/001_account.md#command--5-accountuse) | Bypass G5 ownership guard |
| 2 | [`.account.delete`](../command/001_account.md#command--6-accountdelete) | Bypass G6 ownership guard |
| 3 | [`.account.relogin`](../command/001_account.md#command--12-accountrelogin) | Bypass G7 ownership guard |
| 4 | `.accounts` | Bypass G8 when used with `owner::0` or `owner::USER@MACHINE` (Feature 064) |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Automatic Account Rotation](../user_story/001_account_rotation.md) | Bypass G5 ownership gate on rotation eligibility |

### See Also

- [feature/036_account_ownership.md](../../feature/036_account_ownership.md) — G1–G8 ownership enforcement gates; force:: bypass design
- [feature/037_accounts_usage_param_unification.md](../../feature/037_accounts_usage_param_unification.md) — force:: as unified param on `.accounts`/`.usage`
