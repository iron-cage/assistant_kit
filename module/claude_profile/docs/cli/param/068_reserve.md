# Parameter: 68. `reserve::`

Mutation param on `.accounts` and `.usage` that sets or clears the `reserve` field in `{name}.json`. A reserved account remains fully eligible for selection — it is deprioritized in sort order, not excluded — so it is only picked by the footer recommendation, `.usage rotate::1`, or auto-switch when no non-reserved eligible account remains. Supports comma-list `name::X,Y,Z` for batch operations.

- **Default:** *(omit)* — no reserve write when absent; `reserve` defaults to `false` for accounts that have never set it
- **Constraints:** `0`, `1` (exact string match against `"1"`; any other value, including `"true"`/`"false"`, is silently treated as `0`/clear — see Values table below). `name::` supports comma-list (e.g., `name::X,Y,Z`) for batch; absent `name::` applies to all accounts in the current filtered set.
- **Purpose:** Deprioritize an account for automatic rotation while keeping it a usable fallback — for accounts a caller wants touched deliberately (via `.account.use`) but not consumed first by unattended rotation, "unless there is no left."

**Behavior:**

```text
reserve::1 name::X          → write reserve = true  in X.json
reserve::0 name::X          → write reserve = false in X.json
reserve::1                  → batch-set reserve = true for all accounts in filtered set
reserve::1 name::X,Y,Z      → batch-set reserve = true for X, Y, Z
reserve::1 name::X dry::1   → preview without writing
```

**Values:**

| Value | Effect |
|-------|--------|
| `0` (default); any value other than the exact string `1` | Account sorts by strategy order alone (name/renew/renews) |
| `1` (exact string match) | Account sorts after all non-reserved accounts, regardless of strategy order — see below |

**Sort-key mechanics (soft deprioritization, not a gate):** `reserve` is a leading sort key prepended to every strategy in `find_next_for_strategy()` — `(reserve, <strategy key>)` — so non-reserved accounts (`reserve=false`) always sort before reserved ones (`reserve=true`), and accounts within each group retain their existing strategy-relative order. No eligibility gate is added or changed: a reserved account still passes Gates 1–9 like any other candidate. The existing "first eligible wins, else `None`" walk is untouched — it naturally lands on a reserved account only when every non-reserved candidate has already been excluded by an eligibility gate or exhausted. See [algorithm/007_sort_strategies.md](../../algorithm/007_sort_strategies.md) for the full sort-key table.

**Not a claim lock:** `reserve` never removes an account from footer recommendation, `rotate::1`, or auto-switch outright — it only lowers its sort priority. Contrast with [`lock::`](067_lock.md), which is a hard, unconditional exclusion. The two flags are independent and may be combined (e.g., `reserve::1 lock::0` — deprioritized but still autoswitchable as a last resort).

**Not an ownership gate:** `reserve::` writes are NOT gated by account ownership — any caller may reserve or unreserve any account, mirroring `assignee::`'s ungated write path. `force::1` has no effect on `reserve::` — there is no gate for it to bypass.

**Notes:**
- `reserve::` and `.accounts owner::`/`assignee::`/`lock::` are independent fields.
- Batch mode (`name::` absent) writes `reserve` to every account in the current filtered set; no per-account gate can reject the write.
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

- [cli/param/067_lock.md](067_lock.md) — `lock::` — sibling account-management flag; hard claim gate instead of a soft sort deprioritization
- [cli/param/062_owner.md](062_owner.md) — `owner::` — ownership field; independent of `reserve`
- [feature/070_account_claim_and_reservation_control.md](../../feature/070_account_claim_and_reservation_control.md) — `reserve` design, full properties table
- [algorithm/007_sort_strategies.md](../../algorithm/007_sort_strategies.md) — leading sort-key mechanics across all three strategies
- [algorithm/005_next_account_selection.md](../../algorithm/005_next_account_selection.md) — positive selection algorithm `reserve` participates in
