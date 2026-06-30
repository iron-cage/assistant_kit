# Test: Feature 038 — Usage Strategy Rotate

Feature behavioral requirement test cases for `docs/feature/038_usage_strategy_rotate.md`. Each FT case maps to one or more acceptance criteria.

### AC Coverage Index

| FT | Criterion | AC |
|----|-----------|-----|
| FT-01 | `rotate::1` switches to footer-recommended account and outputs "switched to '{name}'" | AC-01 |
| FT-02 | `rotate::1 dry::1` previews target without switching | AC-02 |
| FT-03 | No eligible candidate → exit 1, table still rendered | AC-03 |
| FT-04 | `rotate::1 live::1` → exit 1 before fetch | AC-04 |
| FT-05 | G5 gate: non-owned account skipped, owned account selected | AC-05 |
| FT-06 | `force::1` bypasses G5: non-owned account becomes eligible | AC-06 |
| FT-07 | `rotate::1 sort::renews` switches to soonest billing renewal winner | AC-07 |
| FT-08 | `rotate::1 format::json` executes switch; JSON unchanged | AC-08 |
| FT-09 | Post-switch touch uses in-memory quota (no extra API call) | AC-09 |
| FT-10 | Exit code 1 on ownership violation without force | AC-10 |
| FT-11 | Rotation touch re-syncs live credentials from store after `apply_touch` | AC-11 |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | `rotate::1` switches to footer-recommended account, output contains "switched to" | AC-01 | Core Switch |
| FT-02 | `rotate::1 dry::1` previews, credentials unchanged | AC-02 | Dry Run |
| FT-03 | No eligible account → exit 1, table rendered | AC-03 | No Candidate |
| FT-04 | `rotate::1 live::1` exits 1 immediately | AC-04 | Mutual Exclusion |
| FT-05 | G5 gate skips non-owned, selects next owned | AC-05 | Ownership Gate |
| FT-06 | `force::1` allows rotation to non-owned account | AC-06 | Force Bypass |
| FT-07 | `rotate::1 sort::renews` — soonest billing renewal winner selected | AC-07 | Strategy Selection |
| FT-08 | `rotate::1 format::json` — switch happens, JSON body unchanged | AC-08 | Format Interaction |
| FT-09 | Post-switch touch fires without extra quota API call | AC-09 | Touch Reuse |
| FT-10 | Non-owned target without force → exit 1 ownership violation | AC-10 | Ownership Gate |
| FT-11 | Rotation touch re-syncs live credentials after apply_touch | AC-11 | BUG-310 MRE |
| CC-01 | `rotate::1` offline (cache-eligible winner) — live creds replaced by winner | AC-01 | Offline / BUG-310 |
| CC-02 | `rotate::1 touch::1` offline — fs::copy re-sync fires, live creds still replaced | AC-11 | Offline / BUG-310 |
| CC-03 | `rotate::0` explicit — exit 0, no switch, credentials unchanged | AC-01 | Explicit Disable |
| CC-04 | `rotate::0` — usage table still rendered (no suppression) | AC-03 | Explicit Disable |
| CC-05 | `rotate::1 sort::name` — alphabetically-first eligible account wins | AC-07 | Strategy Selection |
| CC-06 | `rotate::1 format::tsv` offline — switch executes, format flag not rejected | AC-08 | Format Interaction |
| CC-07 | `rotate::1 dry::1` offline — `[dry-run]` output, credentials unchanged | AC-02 | Dry Run |

**Total:** 18 test cases (11 FT + 7 CC)

---

### FT-01: `rotate::1` switches to footer-recommended account, output contains "switched to"

- **Given:** Two owned accounts: `alpha@test.com` (h5_util=20.0, 80% left) and `beta@test.com` (h5_util=70.0, 30% left). Neither is current. `sort::renew` (default). `alpha` has soonest 7d renewal.
- **When:** `clp .usage rotate::1`
- **Then:** Exit 0. Credentials updated to `alpha@test.com` (renew winner). Output contains `switched to 'alpha@test.com'`. Footer `Next:` line shows `alpha@test.com`.
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

- **Given:** All accounts are either current, active, or h-exhausted (no eligible candidate for `sort::renew`).
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
- **Then:** Exit 0. Switches to `mine@test.com` (foreign skipped by G5). Footer `Next:` line shows `mine@test.com` (non-owned account excluded from footer recommendation too).
- **Exit:** 0
- **Source:** [038_usage_strategy_rotate.md AC-05](../../../docs/feature/038_usage_strategy_rotate.md)

---

### FT-06: `force::1` allows rotation to non-owned account

- **Given:** Same two accounts as FT-05: `foreign@test.com` (is_owned=false, best renew) and `mine@test.com` (is_owned=true). Neither current.
- **When:** `clp .usage rotate::1 force::1`
- **Then:** Exit 0. Switches to `foreign@test.com` (G5 bypassed). Footer `Next:` line shows `foreign@test.com`.
- **Exit:** 0
- **Source:** [038_usage_strategy_rotate.md AC-06](../../../docs/feature/038_usage_strategy_rotate.md)

---

### FT-07: `rotate::1 sort::renews` — soonest billing renewal winner selected

- **Given:** Two owned accounts with different billing renewal dates. Neither current.
- **When:** `clp .usage rotate::1 sort::renews`
- **Then:** Exit 0. Switches to the account with the soonest billing renewal. Footer `Next:` line shows that account. Output: `switched to '{name}'`.
- **Exit:** 0
- **Source:** [038_usage_strategy_rotate.md AC-07](../../../docs/feature/038_usage_strategy_rotate.md)

---

### FT-08: `rotate::1 format::json` — switch executes, JSON unchanged

- **Given:** Two owned accounts; one is the `sort::renew` winner.
- **When-A:** `clp .usage format::json`
- **When-B:** `clp .usage rotate::1 format::json`
- **Then-A:** Credentials unchanged. JSON array returned alphabetically.
- **Then-B:** Credentials updated (switch executed). JSON array identical to When-A (no `"switched_to"` or extra field). Exit 0.
- **Exit:** 0 both cases
- **Source:** [038_usage_strategy_rotate.md AC-08](../../../docs/feature/038_usage_strategy_rotate.md)

---

### FT-09: Post-switch touch fires without extra quota API call

- **Given:** One owned inactive account with no active 5h window (touch trigger condition). `rotate::1 touch::1`.
- **When:** `clp .usage rotate::1 touch::1`
- **Then:** Exit 0. Switch executed. Touch fires for the winner using in-memory `AccountQuota` — total API call count equals N accounts (not N+1).
- **Exit:** 0
- **Live:** yes (requires API access)
- **Source:** [038_usage_strategy_rotate.md AC-09](../../../docs/feature/038_usage_strategy_rotate.md)

---

### FT-10: Non-owned target without force → exit 1 ownership violation

- **Given:** Only one non-current, non-active account in the store: `foreign@test.com` (is_owned=false). `force::0` (default).
- **When:** `clp .usage rotate::1`
- **Then:** Exit 1. Error message contains `"ownership violation"` or `"no eligible account"`. Credentials unchanged.
- **Exit:** 1
- **Source:** [038_usage_strategy_rotate.md AC-10](../../../docs/feature/038_usage_strategy_rotate.md)

---

### FT-11: Rotation touch re-syncs live credentials from store after `apply_touch` (BUG-310 MRE)

- **Given:** Two accounts in a `TempDir` credential store: `current@test.com` (current, h-exhausted — triggers rotation) and `winner@test.com` (idle — `resets_at=None`, valid quota, owned). A `ClaudePaths` with a writable `~/.claude/.credentials.json`. The winner's store credentials contain `accessToken = "token_A"`. `rotate::1 touch::1`.
- **When:** The rotation dispatch executes: (1) `switch_account(winner)` copies `token_A` from store to live. (2) `apply_touch(winner)` spawns a subprocess that refreshes the token — `refresh_account_token` writes `token_B` to the STORE file `{name}.credentials.json` via `save(update_marker=false)`. (3) The re-sync step copies the updated store credentials to live.
- **Then:** The live session file (`~/.claude/.credentials.json`) contains `token_B` (post-touch refreshed token), NOT `token_A` (pre-touch stale token). Reading the live file and the store file yields the same `accessToken` value.
- **Exit:** N/A (unit test — no exit code; tests the `api.rs` rotation dispatch block)
- **Source fn:** `mre_bug310_rotation_touch_resyncs_live_credentials` (in `tests/usage/api_tests_b.rs`)
- **Note:** BUG-310 MRE. Before fix, `apply_touch` writes refreshed credentials to STORE only (via `refresh_account_token → save(update_marker=false)`); the live session retains pre-refresh `token_A`. If the OAuth server invalidates `token_A` during refresh, the live session dies. Fix: re-sync store → live after `apply_touch` at `api.rs:838`.
- **Source:** [038_usage_strategy_rotate.md AC-11](../../../docs/feature/038_usage_strategy_rotate.md)
