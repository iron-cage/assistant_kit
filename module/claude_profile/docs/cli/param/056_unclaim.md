# Parameter: 56. `unclaim::`

> **REMOVED (Feature 064):** The `unclaim::` parameter has been removed from the unified parameter set on `.accounts` and `.usage`. Ownership release is now performed via `owner::0`.
>
> **Migration:**
> - `unclaim::1 name::X` → `owner::0 name::X`
> - `unclaim::1` (batch, no `name::`) → `owner::0` (batch-clears all owned accounts in filtered set)
> - `unclaim::1 name::X force::1` → `owner::0 name::X force::1`
>
> Using `unclaim::1` now exits 1 with a migration message.
>
> See [feature/064_active_marker_and_owner_redesign.md](../../feature/064_active_marker_and_owner_redesign.md) for full context.

[Historical specification retained below for reference.]

---

~~Mutation param on `.accounts` and `.usage` that clears the `owner` field in `{name}.json` for one or all accounts. Re-activated in Feature 037 as part of the unified parameter set; the original `unclaim::` on `.account.save` was removed and the standalone `.account.unclaim` command was deregistered. **Removed again in Feature 064** — replaced by `owner::0` sentinel.~~

### See Also

- [cli/param/062_owner.md](062_owner.md) — `owner::` — `owner::0` replaces `unclaim::1`; `owner::USER@MACHINE` for ownership assignment
- [cli/param/058_force.md](058_force.md) — `force::` — bypass G8 ownership gate
- [feature/036_account_ownership.md](../../feature/036_account_ownership.md) — ownership model; G1–G8 enforcement gates
- [feature/064_active_marker_and_owner_redesign.md](../../feature/064_active_marker_and_owner_redesign.md) — removal context
