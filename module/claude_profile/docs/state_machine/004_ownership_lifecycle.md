# State Machine: Ownership Lifecycle

### Scope

- **Purpose**: Define the lifecycle states and enforcement gates for account ownership, plus the explicit-command gate for the independent `claim_lock` flag.
- **Responsibility**: Documents `unclaimed`/`owned_here`/`owned_elsewhere` states, transition guards, G1–G8 ownership enforcement gates, and the G9 `claim_lock` explicit-command gate.
- **In Scope**: Ownership state transitions; `is_owned` semantics; save-is-ownership-neutral rule; gate enforcement table; G9 (`claim_lock`, `force::1`-bypassable, on `.account.use` and `.accounts assignee::` target-side).
- **Out of Scope**: Account lifecycle (→ state_machine/001); ownership feature parameters (→ feature/036); Gate 9 (the unconditional eligibility-layer `claim_lock` exclusion, a *different* gate on the same field → algorithm/004); `reserve` field (→ feature/070).

### States

| State | `owner` field | `is_owned` | Eligible for live operations? |
|-------|--------------|-----------|-------------------------------|
| `unclaimed` | `""` or absent | `true` | Yes — treated as owned by anyone |
| `owned_here` | matches `current_identity()` | `true` | Yes — this machine owns it |
| `owned_elsewhere` | non-empty, different machine | `false` | No — cache/approximate only |

`current_identity()` = `"{user}@{hostname}"`. `is_owned = true` when `owner` is empty OR matches `current_identity()`.

**`claim_lock` is orthogonal to these states:** it is a separate boolean field, not a fourth ownership state — any of `unclaimed`/`owned_here`/`owned_elsewhere` can independently carry `claim_lock = true` or `false`. Toggled via `.accounts lock::`, which is ungated (no ownership check on the write itself). See [feature/070](../feature/070_account_claim_and_reservation_control.md).

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
| G5 | `.account.use` (`account_use_routine()` in `account_ops.rs`) | Switch requires ownership (unless `force::1`). `.usage rotate::1` mirrors this account via the shared `find_next_for_strategy()` selection ([feature/038](../feature/038_usage_strategy_rotate.md)) but is gated separately at the eligibility layer (Gate 8, → algorithm/004), not by G5 itself |
| G8 | `.accounts owner::` | G8 guards ownership-write from non-owner (unless `force::1`) |

### Claim-Lock Gate (G9)

A *different* field from `owner` — `claim_lock` (→ [feature/070](../feature/070_account_claim_and_reservation_control.md)) has its own explicit-command gate, numbered G9 to continue this table's sequence since it shares a call site with G5 and follows the identical `force::1`-bypass mechanic. G9 is **not** an ownership check — an account can be `is_owned = true` and still be blocked by G9 if `claim_lock = true`.

| Gate | Location | Condition | Effect | `force::1` bypass |
|------|----------|-----------|--------|-------------------|
| G9 | `.account.use` (`account_use_routine()` in `account_ops.rs`); `.accounts assignee::` target-side (`accounts_routine()` in `commands/accounts.rs`) | `claim_lock = true` | Exit 1 with a claim-lock violation message; `switch_account()` / marker write not performed | Yes — `force::1` skips the gate; proceeds normally |

G9 is independent of, and stacks with, ownership: `.account.use name::X` on an account that is both non-owned (fails G5) and claim-locked (fails G9) requires `force::1` to pass both gates — `force::1` bypasses each independently, not jointly. G9 has **no** effect on the *automatic* selection path (footer recommendation, `rotate::1`, auto-switch) — that path is gated separately and unconditionally by Gate 9 in [algorithm/004](../algorithm/004_eligibility_gates.md) (no `force::1` bypass there at all). Same field, two gates, two bypass semantics — see [feature/070](../feature/070_account_claim_and_reservation_control.md) for the full picture.

### Save is Ownership-Neutral

`.account.save` always passes `owner: None` to `save()` — it never overwrites the `owner` field. Ownership must be set explicitly via `.accounts owner::`. This means re-saving an account never accidentally unclaims or re-claims it.

### Behavioral Invariants

- `.account.save` never modifies the `owner` field — ownership is always set explicitly via `.accounts owner::`.
- An unclaimed account (`owner = ""`) is treated as owned by any machine (`is_owned = true`).
- Transitioning ownership away from the current machine or to a different machine requires `force::1`.
- `claim_lock` transitions (`.accounts lock::0`/`lock::1`) are independent of ownership state and ownership transitions — locking or unlocking never reads or writes the `owner` field, and vice versa.

### Features

| File | Relationship |
|------|-------------|
| [feature/036_account_ownership.md](../feature/036_account_ownership.md) | Full ownership feature spec; G1–G8 gate details |
| [feature/063_explicit_ownership_claim.md](../feature/063_explicit_ownership_claim.md) | `owner::` parameter write path |
| [feature/070_account_claim_and_reservation_control.md](../feature/070_account_claim_and_reservation_control.md) | `claim_lock` field; G9 gate; full properties table |

### Schema

| File | Relationship |
|------|-------------|
| [schema/002](../schema/002_account_json.md) | `owner`, `claim_lock` fields in `{name}.json` |
