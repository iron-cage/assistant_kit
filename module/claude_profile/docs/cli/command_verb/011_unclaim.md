# Verb :: unclaim

Releases ownership of a saved account profile by writing `owner: ""` to `{name}.json` via `write_owner()` directly. Does not touch credentials (`{name}.credentials.json`), does not modify the active marker (`_active_{machine}_{user}`), does not call `save()`. This is a pure metadata-only operation.

### Nouns

| # | Noun | Command | Idempotent | Requires Session |
|---|------|---------|-----------|-----------------|
| 1 | [account](../command_noun/001_account.md) | `.account.unclaim` | Yes | No |

### Behavioral Contract

**Pre-conditions:**
- Named account exists in credential store (`{name}.json` present)
- `name::` is required — no name inference from active marker or `~/.claude.json`
- G8 ownership gate passes: `is_owned(&owner)` must be true (owner is empty, absent, or matches `current_identity()`)

**Post-conditions:**
- `{name}.json` contains `"owner": ""`
- `{name}.credentials.json` unchanged (mtime identical to before call)
- Active marker `_active_{machine}_{user}` unchanged
- All G1–G8 enforcement gates disabled for this account

**Side effects:**
- Writes `{name}.json` only (owner field cleared); no other files modified

### Ownership Gate (G8)

Before any write (including before the dry-run check), the `accounts_routine()` unclaim path reads the current `owner` field from `{name}.json` and evaluates `is_owned(&owner)`:

- `owner` empty or matches `current_identity()` → gate passes; proceed
- `owner` non-empty and does NOT match `current_identity()` → exit 1 with `"ownership violation: this account is owned by {owner}"`

This matches the pattern of G5/G6/G7 — gate evaluates before any mutation.

### Idempotency

**Yes.** Unclaiming an already-unowned account (`owner == ""`) passes the G8 gate and `write_owner()` writes `""` again — identical stored state.

### Common Parameters

| Parameter | Semantics | Required |
|-----------|-----------|----------|
| `name::` | Account name (full email address); no name inference | Yes |
| `dry::` | Validate and print intent without writing | No |
| `force::` | Bypass G8 ownership gate; allow any identity to unclaim | No |
| `trace::` | Emit `[trace]` diagnostic lines to stderr | No |

### State Transition Pattern

**Metadata-only mutation.** Writes `owner: ""` to `{name}.json`. No credential rotation; `{name}.credentials.json` is not read or written. Active marker is not changed. The account's lifecycle state (saved/active) is unchanged.

```
[saved/active, owned] --account.unclaim--> [same state, owner: ""]
[saved/active, unowned] --account.unclaim--> [same state, owner: ""]  (idempotent)
```

### Migration (Feature 037 + Feature 064)

> `.account.unclaim` has been removed as a standalone working command (Feature 037). Its behavior was absorbed as `unclaim::1`. Feature 064 removed `unclaim::1` and replaced it with `owner::0`.
> - `clp .account.unclaim name::X` → exits 1 with generic "unknown command" error (fully deregistered)
> - `clp .accounts unclaim::1 name::X` → exits 1 with REMOVED_TOGGLE migration message (Feature 064)
> - `clp .accounts owner::0 name::X` → clears owner field (current behavior)
> - `clp .accounts owner::0 name::X force::1` → bypasses G8; clears owner regardless of caller identity
> - Batch release: `clp .accounts owner::0` (no `name::`) → applies to all filtered owned accounts
>
> See [feature/064_active_marker_and_owner_redesign.md](../../feature/064_active_marker_and_owner_redesign.md) AC-08 through AC-12.

### Cross-References

| File | Relationship |
|------|-------------|
| [feature/036_account_ownership.md](../../feature/036_account_ownership.md) | Ownership model, G8 gate, `write_owner()` implementation, AC-02/AC-16–AC-21 (including `force::` bypass) |
| [feature/002_account_save.md](../../feature/002_account_save.md) | `.account.save` always stamps `current_identity()` — use `.accounts unclaim::1` to clear (post-Feature 037) |
| [feature/037_accounts_usage_param_unification.md](../../feature/037_accounts_usage_param_unification.md) | `unclaim::` absorbed as mutation param; `.account.unclaim` standalone removed; batch unclaim added |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.unclaim`](../command/001_account.md#command--17-accountunclaim) | Release ownership of saved account profile (removed in Feature 037 — use `.accounts unclaim::1`) |
