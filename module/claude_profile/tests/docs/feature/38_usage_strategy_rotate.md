# Test: Feature 038 — Usage Strategy Rotate

Feature behavioral requirement test cases for `docs/feature/038_usage_strategy_rotate.md`. Each FT case maps to one or more acceptance criteria.

### AC Coverage Index

| FT | Criterion | AC |
|----|-----------|-----|
| FT-01 | `rotate::1` switches to `→` winner and outputs "switched to '{name}'" | AC-01 |
| FT-02 | `rotate::1 dry::1` previews target without switching | AC-02 |
| FT-03 | No eligible candidate → exit 1, table still rendered | AC-03 |
| FT-04 | `rotate::1 live::1` → exit 1 before fetch | AC-04 |
| FT-05 | G5 gate: non-owned account skipped, owned account selected | AC-05 |
| FT-06 | `force::1` bypasses G5: non-owned account becomes eligible | AC-06 |
| FT-07 | `rotate::1 next::endurance` switches to endurance winner | AC-07 |
| FT-08 | `rotate::1 next::drain` switches to drain winner | AC-08 |
| FT-09 | `rotate::1 format::json` executes switch; JSON unchanged | AC-09 |
| FT-10 | Post-switch touch uses in-memory quota (no extra API call) | AC-10 |
| FT-11 | Exit code 1 on ownership violation without force | AC-11 |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | `rotate::1` switches to → winner, output contains "switched to" | AC-01 | Core Switch |
| FT-02 | `rotate::1 dry::1` previews, credentials unchanged | AC-02 | Dry Run |
| FT-03 | No eligible account → exit 1, table rendered | AC-03 | No Candidate |
| FT-04 | `rotate::1 live::1` exits 1 immediately | AC-04 | Mutual Exclusion |
| FT-05 | G5 gate skips non-owned, selects next owned | AC-05 | Ownership Gate |
| FT-06 | `force::1` allows rotation to non-owned account | AC-06 | Force Bypass |
| FT-07 | `rotate::1 next::endurance` — endurance winner selected | AC-07 | Strategy Selection |
| FT-08 | `rotate::1 next::drain` — drain winner selected | AC-08 | Strategy Selection |
| FT-09 | `rotate::1 format::json` — switch happens, JSON body unchanged | AC-09 | Format Interaction |
| FT-10 | Post-switch touch fires without extra quota API call | AC-10 | Touch Reuse |
| FT-11 | Non-owned target without force → exit 1 ownership violation | AC-11 | Ownership Gate |

**Total:** 11 FT cases

---

### FT-01: `rotate::1` switches to → winner, output contains "switched to"

- **Given:** Two owned accounts: `alpha@test.com` (h5_util=20.0, 80% left) and `beta@test.com` (h5_util=70.0, 30% left). Neither is current. `next::renew` (default). `alpha` has soonest 7d renewal.
- **When:** `clp .usage rotate::1`
- **Then:** Exit 0. Credentials updated to `alpha@test.com` (renew winner). Output contains `switched to 'alpha@test.com'`. `→` on alpha in table.
- **Exit:** 0
- **Source:** [038_usage_strategy_rotate.md AC-01](../../../docs/feature/038_usage_strategy_rotate.md)

---

### FT-02: `rotate::1 dry::1` previews, credentials unchanged

- **Given:** Two owned accounts; one is the renew winner.
- **When:** `clp .usage rotate::1 dry::1`
- **Then:** Exit 0. Output contains `[dry-run] would switch to '{winner}'`. Credentials file unchanged. Active marker unchanged.
- **Exit:** 0
- **Source:** [038_usage_strategy_rotate.md AC-02](../../../docs/feature/038_usage_strategy_rotate.md)

---

### FT-03: No eligible account → exit 1, table rendered

- **Given:** All accounts are either current, active, or h-exhausted (no eligible candidate for `next::renew`).
- **When:** `clp .usage rotate::1`
- **Then:** Exit 1. Table still rendered. Stderr (or stdout) contains `"no eligible account to rotate to"`. Credentials unchanged.
- **Exit:** 1
- **Source:** [038_usage_strategy_rotate.md AC-03](../../../docs/feature/038_usage_strategy_rotate.md)

---

### FT-04: `rotate::1 live::1` exits 1 immediately

- **Given:** Any environment.
- **When:** `clp .usage rotate::1 live::1`
- **Then:** Exit 1 before any quota fetch. Stderr contains mutual-exclusion error message referencing both params.
- **Exit:** 1
- **Source:** [038_usage_strategy_rotate.md AC-04](../../../docs/feature/038_usage_strategy_rotate.md)

---

### FT-05: G5 gate skips non-owned, selects next owned

- **Given:** Two accounts: `foreign@test.com` (is_owned=false, best renew) and `mine@test.com` (is_owned=true, second renew). Neither current. `force::0` (default).
- **When:** `clp .usage rotate::1`
- **Then:** Exit 0. Switches to `mine@test.com` (foreign skipped by G5). `→` on `mine@test.com` in table (non-owned account excluded from → too).
- **Exit:** 0
- **Source:** [038_usage_strategy_rotate.md AC-05](../../../docs/feature/038_usage_strategy_rotate.md)

---

### FT-06: `force::1` allows rotation to non-owned account

- **Given:** Same two accounts as FT-05: `foreign@test.com` (is_owned=false, best renew) and `mine@test.com` (is_owned=true). Neither current.
- **When:** `clp .usage rotate::1 force::1`
- **Then:** Exit 0. Switches to `foreign@test.com` (G5 bypassed). `→` on `foreign@test.com` in table.
- **Exit:** 0
- **Source:** [038_usage_strategy_rotate.md AC-06](../../../docs/feature/038_usage_strategy_rotate.md)

---

### FT-07: `rotate::1 next::endurance` — endurance winner selected

- **Given:** Two owned accounts: `alpha@test.com` (h5_util=20.0, 80% session left — endurance winner) and `beta@test.com` (h5_util=70.0, 30% session left). Neither current.
- **When:** `clp .usage rotate::1 next::endurance`
- **Then:** Exit 0. Switches to `alpha@test.com` (most 5h quota remaining). `→` on alpha. Output: `switched to 'alpha@test.com'`.
- **Exit:** 0
- **Source:** [038_usage_strategy_rotate.md AC-07](../../../docs/feature/038_usage_strategy_rotate.md)

---

### FT-08: `rotate::1 next::drain` — drain winner selected

- **Given:** Same two accounts as FT-07. `beta@test.com` has least non-zero 5h quota (30% — drain winner).
- **When:** `clp .usage rotate::1 next::drain`
- **Then:** Exit 0. Switches to `beta@test.com` (least 5h left, drain strategy). `→` on beta. Output: `switched to 'beta@test.com'`.
- **Exit:** 0
- **Source:** [038_usage_strategy_rotate.md AC-08](../../../docs/feature/038_usage_strategy_rotate.md)

---

### FT-09: `rotate::1 format::json` — switch executes, JSON unchanged

- **Given:** Two owned accounts; one is the `next::renew` winner.
- **When-A:** `clp .usage format::json`
- **When-B:** `clp .usage rotate::1 format::json`
- **Then-A:** Credentials unchanged. JSON array returned alphabetically.
- **Then-B:** Credentials updated (switch executed). JSON array identical to When-A (no `"switched_to"` or extra field). Exit 0.
- **Exit:** 0 both cases
- **Source:** [038_usage_strategy_rotate.md AC-09](../../../docs/feature/038_usage_strategy_rotate.md)

---

### FT-10: Post-switch touch fires without extra quota API call

- **Given:** One owned inactive account with no active 5h window (touch trigger condition). `rotate::1 touch::1`.
- **When:** `clp .usage rotate::1 touch::1`
- **Then:** Exit 0. Switch executed. Touch fires for the winner using in-memory `AccountQuota` — total API call count equals N accounts (not N+1).
- **Exit:** 0
- **Live:** yes (requires API access)
- **Source:** [038_usage_strategy_rotate.md AC-10](../../../docs/feature/038_usage_strategy_rotate.md)

---

### FT-11: Non-owned target without force → exit 1 ownership violation

- **Given:** Only one non-current, non-active account in the store: `foreign@test.com` (is_owned=false). `force::0` (default).
- **When:** `clp .usage rotate::1`
- **Then:** Exit 1. Error message contains `"ownership violation"` or `"no eligible account"`. Credentials unchanged.
- **Exit:** 1
- **Source:** [038_usage_strategy_rotate.md AC-11](../../../docs/feature/038_usage_strategy_rotate.md)
