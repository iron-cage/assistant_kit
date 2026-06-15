# Parameter :: 56. `unclaim::` — REMOVED

**Status:** REMOVED — superseded by the dedicated `.account.unclaim` command.

The `unclaim::` parameter was removed from `.account.save`. To release account ownership, use `clp .account.unclaim name::EMAIL` — a pure metadata operation that calls `write_owner(name, store, "")` directly without touching credentials or the active marker.

### See Also

- [cli/command/001_account.md](../command/001_account.md) — Command 17 `.account.unclaim`
- [feature/036_account_ownership.md](../../feature/036_account_ownership.md) — full ownership model; G1–G8 enforcement gates
