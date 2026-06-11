# Test: `next::` Parameter

Edge case coverage for the `next::` parameter on `.usage`. See [param/032_next.md](../../../../docs/cli/param/032_next.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `next::endurance` accepted with empty credential store | Valid Value |
| EC-2 | `next::drain` accepted with empty credential store | Valid Value |
| EC-2b | `next::renew` accepted with empty credential store | Valid Value |
| EC-3 | `next::bogus` exits 1 naming all three valid values | Invalid Value |
| EC-4 | Footer omitted when no accounts have valid quota data | Footer Threshold |
| EC-5 | `next::drain format::json` — `next::` does not affect JSON output | JSON No-op |
| EC-6 | `next::all` exits 1 — formerly valid, now rejected (TSK-184) | Invalid Value |
| EC-7 | `next::endurance` — `→` on account with most 5h quota remaining | Behavioral Divergence |
| EC-8 | `next::drain` — `→` on account with least non-zero 5h quota remaining | Behavioral Divergence |

---

### EC-1: `next::endurance` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage next::endurance`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source:** [param/032_next.md](../../../../docs/cli/param/032_next.md)

---

### EC-2: `next::drain` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage next::drain`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source:** [param/032_next.md](../../../../docs/cli/param/032_next.md)

---

### EC-2b: `next::renew` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage next::renew`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source:** [param/032_next.md](../../../../docs/cli/param/032_next.md)

---

### EC-3: `next::bogus` exits 1 naming all three valid values

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage next::bogus`
- **Then:** Exits 1. Stderr contains all three valid values: "renew", "endurance", "drain".
- **Exit:** 1
- **Source:** [feature/023_next_account_strategies.md AC-05](../../../../docs/feature/023_next_account_strategies.md)

---

### EC-4: Footer omitted when no accounts have valid quota data

- **Given:** One saved account whose credential file has no `accessToken` (quota fetch fails; `valid_count = 0`).
- **When:** `clp .usage next::endurance`
- **Then:** Exits 0. Stdout does NOT contain "Next by strategy:" (footer suppressed when fewer than 2 accounts have valid quota data).
- **Exit:** 0
- **Source:** [feature/023_next_account_strategies.md AC-07](../../../../docs/feature/023_next_account_strategies.md)

---

### EC-5: `next::drain format::json` — `next::` does not affect JSON output

- **Given:** Two saved accounts with valid credential files (no accessToken — produces error rows in text, but JSON array is still emitted).
- **When-A:** `clp .usage format::json`
- **When-B:** `clp .usage next::drain format::json`
- **Then-A and Then-B:** Both produce identical JSON arrays (alphabetical, no `"->"` markers). `next::` has no effect on JSON output.
- **Exit:** 0 both cases
- **Source:** [feature/023_next_account_strategies.md AC-06](../../../../docs/feature/023_next_account_strategies.md)

---

### EC-6: `next::all` exits 1 — formerly valid, now rejected after TSK-184

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage next::all`
- **Then:** Exits 1. Stderr contains "renew", "endurance", and "drain" (the three valid values). Does NOT contain "all", "session", or "reset". TSK-184 reduced `NextStrategy` from 5 variants to 2; "all" is no longer a recognized value.
- **Exit:** 1
- **Source:** [feature/023_next_account_strategies.md AC-05](../../../../docs/feature/023_next_account_strategies.md)

---

### EC-7: `next::endurance` — `→` placed on account with most 5h quota remaining (Behavioral Divergence A)

- **Given:** Two saved accounts with valid quota: `alpha@test.com` (`five_hour.utilization=20.0` — 80% remaining) and `beta@test.com` (`five_hour.utilization=70.0` — 30% remaining). Neither is current.
- **When:** `clp .usage next::endurance`
- **Then:** Exits 0. The row for `alpha@test.com` contains `→` in the flag column (most 5h quota remaining → endurance winner). `beta@test.com` does NOT have `→`.
- **Exit:** 0
- **Live:** yes (requires live quota data)
- **Source:** [feature/023_next_account_strategies.md AC-03](../../../../docs/feature/023_next_account_strategies.md)

---

### EC-8: `next::drain` — `→` placed on account with least non-zero 5h quota remaining (Behavioral Divergence B)

- **Given:** Same two accounts as EC-7: `alpha@test.com` (`five_hour.utilization=20.0` — 80% remaining) and `beta@test.com` (`five_hour.utilization=70.0` — 30% remaining). Neither is current.
- **When:** `clp .usage next::drain`
- **Then:** Exits 0. The row for `beta@test.com` contains `→` in the flag column (least non-zero 5h quota remaining → drain winner). `alpha@test.com` does NOT have `→`. Divergence from EC-7: the SAME two accounts produce DIFFERENT winners under different `next::` values, proving the parameter governs behavior.
- **Exit:** 0
- **Live:** yes (requires live quota data)
- **Source:** [feature/023_next_account_strategies.md AC-04](../../../../docs/feature/023_next_account_strategies.md)
