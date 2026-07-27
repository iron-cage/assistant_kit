# Parameter: 67. `lock::`

Mutation param on `.accounts` and `.usage` that sets or clears the `claim_lock` field in `{name}.json`. A claim-locked account cannot become the active/switched-to account (footer recommendation, `.usage rotate::1`, auto-switch, `.account.use`, `.accounts assignee::` target) but remains fully readable — quota fetch, refresh, and touch are unaffected. Supports comma-list `name::X,Y,Z` for batch operations.

- **Default:** *(omit)* — no lock write when absent; `claim_lock` defaults to `false` for accounts that have never set it
- **Constraints:** `0`, `1`, `false`, `true`. `name::` supports comma-list (e.g., `name::X,Y,Z`) for batch; absent `name::` applies to all accounts in the current filtered set.
- **Purpose:** Mark an account as "not allowed to be taken" while keeping it usable for quota/refresh/touch — for accounts a caller wants to inspect or keep warm but never wants automatic (or accidental direct) selection to switch onto.

**Behavior:**

```text
lock::1 name::X          → write claim_lock = true  in X.json
lock::0 name::X          → write claim_lock = false in X.json
lock::1                  → batch-set claim_lock = true for all accounts in filtered set
lock::1 name::X,Y,Z      → batch-set claim_lock = true for X, Y, Z
lock::1 name::X dry::1   → preview without writing
```

**Values:**

| Value | Effect |
|-------|--------|
| `0` / `false` (default) | Clear `claim_lock`; account is eligible for selection again |
| `1` / `true` | Set `claim_lock`; account is excluded from Gate 9 (eligibility) and blocked at G9 (explicit-command) — see below |

**Two independent enforcement points** (same `claim_lock` field, two call-site families):

| Enforcement | Location | Scope | `force::1` bypass |
|---|---|---|---|
| Gate 9 (eligibility) | `find_first_eligible()` — feeds footer recommendation, `.usage rotate::1`, auto-switch | Unconditional — same tier as Gate 3 (Occupied); never bypassed | No — a locked account cannot be auto-selected under any parameter combination |
| G9 (explicit-command) | `.account.use` (direct target), `.accounts assignee::` (target-side `name::`) | Named-target override | Yes — `force::1` proceeds despite the lock |

See [algorithm/004_eligibility_gates.md](../../algorithm/004_eligibility_gates.md) (Gate 9) and [state_machine/004_ownership_lifecycle.md](../../state_machine/004_ownership_lifecycle.md) (G9) for the full mechanics of each.

**Read/quota operations are never gated:** fetch, refresh, and touch (G1/G1b/G2/G4) ignore `claim_lock` entirely — a locked account keeps reporting live quota and stays token-fresh. This is the defining difference from ownership: `claim_lock` restricts *becoming the active account*, not *being read*.

**Not an ownership gate:** `lock::`/`unlock` writes are NOT gated by account ownership — any caller may lock or unlock any account's `claim_lock` field, mirroring `assignee::`'s ungated write path rather than `owner::`'s G8-guarded one. `force::1` has no effect on the `lock::` write itself (nothing to bypass); it only affects whether G9 honors an existing lock at `.account.use`/`assignee::` time.

**Notes:**
- `lock::` and `.accounts owner::`/`assignee::` are independent fields — locking an account does not change its owner or assignee, and vice versa.
- Batch mode (`name::` absent) writes `claim_lock` to every account in the current filtered set (same filter params as other `.accounts` batch operations); no per-account gate can reject the write, so batch lock/unlock always succeeds for every matched account.
- `dry::1` previews the write without touching `{name}.json`.
- Does not touch credentials, active marker, or any file other than `{name}.json`.

### Referenced Type

- **Fundamental Type:** `bool`

### Referenced Parameter Groups

*(ungrouped — presented in the "Account Ownership" help group alongside `owner::`/`assignee::`/`force::`)*

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.accounts`](../command/001_account.md) | Primary host — mutation param (unified param set, Feature 037) |
| 2 | `.usage` | Shared unified param set (Feature 037) |

### See Also

- [cli/param/068_reserve.md](068_reserve.md) — `reserve::` — sibling account-management flag; soft rotation deprioritization instead of a hard claim gate
- [cli/param/062_owner.md](062_owner.md) — `owner::` — ownership field; independent of `claim_lock`
- [cli/param/058_force.md](058_force.md) — `force::` — bypasses G9 (explicit-command) only; never bypasses Gate 9 (eligibility)
- [feature/070_account_claim_and_reservation_control.md](../../feature/070_account_claim_and_reservation_control.md) — `claim_lock` design, full properties table
- [algorithm/004_eligibility_gates.md](../../algorithm/004_eligibility_gates.md) — Gate 9 (unconditional eligibility exclusion)
- [state_machine/004_ownership_lifecycle.md](../../state_machine/004_ownership_lifecycle.md) — G9 (explicit-command gate)
