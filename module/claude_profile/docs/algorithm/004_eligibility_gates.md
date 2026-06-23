# Algorithm: Next-Account Eligibility Gates

### Purpose

Filter candidates for next-account recommendation and auto-switch. An account is **skipped** when any gate fires. Only accounts passing all gates are eligible.

### Entry Points

- `src/usage/sort_next.rs:24-35` — `find_first_eligible()` (gates 1–6)
- `src/usage/sort_next.rs:59` — `extra` closure passed by `find_next_for_strategy()` (gates 7–8)

### Gate Table

| # | Gate | Skip condition | Source |
|---|------|---------------|--------|
| 1 | Current | `is_current = true` | `sort_next.rs:27` |
| 2 | Active | `is_active = true` | `sort_next.rs:27` |
| 3 | Occupied | `is_occupied_elsewhere = true` | `sort_next.rs:28` |
| 4 | Error | `result = Err(...)` | `sort_next.rs:29` |
| 5 | h-exhausted | `five_hour.utilization >= 85.0` (≤ 15% left) | `sort_next.rs:30` |
| 6 | Expired | `expires_at_ms / 1000 ≤ now_secs` | `sort_next.rs:31` |
| 7 | Weekly-exhausted | `prefer_weekly(aq, prefer) ≤ 5.0` | `sort_next.rs:59` (extra) |
| 8 | Foreign-owned | `is_owned = false AND gate_ownership = true` | `sort_next.rs:59` (extra) |

### Gate 8 Context — `gate_ownership` varies by call site

| Call site | `gate_ownership` | Effect |
|---|---|---|
| Auto-switch execution (`api.rs:789`) | `!params.force` | Ownership required unless `force::1` |
| Table marker render (`render.rs:53`) | `rotate && !force` | Only when rotate active and not forced |
| Footer recommendation (`render.rs:242`) | `false` (hardcoded) | Ownership never checked for footer |

### `is_owned` Semantics

`is_owned = true` when `owner` field is empty OR matches `current_identity()` (`{user}@{hostname}`). `is_owned = false` when a different machine owns the account. Source: `types.rs:193-195`.

### Cross-References

| File | Relationship |
|------|-------------|
| [feature/039_decision_algorithms.md](../feature/039_decision_algorithms.md) | Table 4 (legacy reference) |
| [algorithm/003](003_quota_status_groups.md) | Status groups — same 5h/7d thresholds |
| [algorithm/005](005_next_account_selection.md) | Positive selection uses these gates |
| [feature/036_account_ownership.md](../feature/036_account_ownership.md) | `is_owned` field semantics |
| [feature/061_solo_token_conservation.md](../feature/061_solo_token_conservation.md) | Solo gate (before G1 in fetch/refresh/touch) |
