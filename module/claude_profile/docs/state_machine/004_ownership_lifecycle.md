# State Machine: Ownership Lifecycle

### Scope

- **Purpose**: Define the lifecycle states and enforcement gates for account ownership.
- **Responsibility**: Documents `unclaimed`/`owned_here`/`owned_elsewhere` states, transition guards, and G1–G8 enforcement gates.
- **In Scope**: Ownership state transitions; `is_owned` semantics; save-is-ownership-neutral rule; gate enforcement table.
- **Out of Scope**: Account lifecycle (→ state_machine/001); ownership feature parameters (→ feature/036).

### States

| State | `owner` field | `is_owned` | Eligible for live operations? |
|-------|--------------|-----------|-------------------------------|
| `unclaimed` | `""` or absent | `true` | Yes — treated as owned by anyone |
| `owned_here` | matches `current_identity()` | `true` | Yes — this machine owns it |
| `owned_elsewhere` | non-empty, different machine | `false` | No — cache/approximate only |

`current_identity()` = `"{user}@{hostname}"`. `is_owned = true` when `owner` is empty OR matches `current_identity()`.

### Transitions

```
[unclaimed]      --.account.save (auto)--> [unclaimed]     (save is ownership-neutral: owner=None)
[unclaimed]      --.accounts owner::X --> [owned_here]     (X = current_identity())
[unclaimed]      --.accounts owner::X --> [owned_elsewhere] (X = different machine)
[owned_here]     --.accounts owner::0 name::X--> [unclaimed]
[owned_elsewhere]--.accounts owner::0 name::X--> [unclaimed]   (force::1 required if not the owner)
[owned_here]     --.accounts owner::Y --> [owned_elsewhere] (force::1 required)
[owned_elsewhere]--.accounts owner::X --> [owned_here]     (X = current_identity(); force::1 required)
```

### Ownership Gates (G1–G8)

Nine enforcement gates (G1, G1b, G2–G8) prevent non-owner credential operations. Key gates:

| Gate | Location | Effect |
|------|----------|--------|
| G1 | `fetch.rs` | Non-owned accounts use `read_cached_quota()` only |
| G1b | `fetch.rs` | Occupied-elsewhere accounts use `approximate_quota()` only |
| G2 | `refresh_predicate.rs` | `should_refresh()` returns false for non-owned or occupied |
| G4 | `touch.rs` | `apply_touch()` skips non-owned or occupied |
| G5 | `.usage rotate::` | Rotation requires ownership (unless `force::1`) |
| G8 | `.accounts owner::` | G8 guards ownership-write from non-owner (unless `force::1`) |

### Save is Ownership-Neutral

`.account.save` always passes `owner: None` to `save()` — it never overwrites the `owner` field. Ownership must be set explicitly via `.accounts owner::`. This means re-saving an account never accidentally unclaims or re-claims it.

### Behavioral Invariants

- `.account.save` never modifies the `owner` field — ownership is always set explicitly via `.accounts owner::`.
- An unclaimed account (`owner = ""`) is treated as owned by any machine (`is_owned = true`).
- Transitioning ownership away from the current machine or to a different machine requires `force::1`.

### Features

| File | Relationship |
|------|-------------|
| [feature/036_account_ownership.md](../feature/036_account_ownership.md) | Full ownership feature spec; G1–G8 gate details |
| [feature/063_explicit_ownership_claim.md](../feature/063_explicit_ownership_claim.md) | `owner::` parameter write path |

### Schema

| File | Relationship |
|------|-------------|
| [schema/002](../schema/002_account_json.md) | `owner` field in `{name}.json` |
