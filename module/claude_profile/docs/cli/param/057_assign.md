# Parameter :: 57. `assign::`

> **REMOVED (Feature 064):** The `assign::` parameter has been removed from the unified parameter set on `.accounts` and `.usage`. Marker assignment was moved to `active::USER@MACHINE name::X` (Feature 064), then `active::` was renamed to `assignee::` (Feature 065). Current CLI surface uses `assignee::`.
>
> **Migration:**
> - `assign::1 name::X` → `assignee::0 name::X` (current machine via `assignee::0` sentinel)
> - `assign::1 name::X for::bob@laptop` → `assignee::bob@laptop name::X`
>
> Using `assign::1` now exits 1 with a migration message.
>
> See [feature/065_assignee_param_redesign.md](../../feature/065_assignee_param_redesign.md) for full context.

[Historical specification retained below for reference.]

---

~~Mutation param on `.accounts` and `.usage` that writes the per-machine active-account marker for any host+user pair.~~ When `assign::1 name::X`, writes `{credential_store}/_active_{machine}_{user}` = X. Introduced in Feature 037 absorbing the former standalone `.account.assign` command.

- **Default:** `0` (no marker write)
- **Constraints:** Accepted values: `0`, `1`
- **Purpose:** Pre-seed which account a machine should use when accessing a shared or synced credential store — without performing a credential rotation. Works with `for::USER@MACHINE` to target a specific machine's marker.

**Behavior:**

```text
assign::1 name::X             → write _active_{current_machine}_{current_user} = X
assign::1 name::X for::bob@laptop  → write _active_laptop_bob = X
assign::1                     → emit live usage block (current machine + copy-paste examples)
```

When `name::` is absent and `assign::1`: emits a context-aware live usage block (current machine identity, active account, copy-paste ready examples) and exits 0 — identical to the former `.account.assign` no-args behavior.

**No credential side effects:** `~/.claude/.credentials.json`, `~/.claude.json`, and `{name}.json` are never touched. Only the marker file is written.

**Notes:**
- `force::1` has no effect when combined with `assign::1` — marker writes have no ownership gate.
- `dry::1 assign::1 name::X` previews without writing.
- Sanitization rules for `for::` components are identical to `active_marker_filename()`.

### Referenced Type

- **Fundamental Type:** `bool`

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | `.accounts` | Primary host — mutation param absorbed from `.account.assign` (Feature 037) |
| 2 | `.usage` | Shared unified param set (Feature 037) |

### See Also

- [cli/param/053_for.md](053_for.md) — `for::` — `USER@MACHINE` target identity for marker write
- [feature/032_account_assign.md](../../feature/032_account_assign.md) — full assign behavior being absorbed
- [feature/037_accounts_usage_param_unification.md](../../feature/037_accounts_usage_param_unification.md) — Feature 037 param unification
