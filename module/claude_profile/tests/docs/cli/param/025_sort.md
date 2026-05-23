# Test: `sort::` Parameter

Edge case coverage for the `sort::` parameter on `.usage`. See [param/025_sort.md](../../../../docs/cli/param/025_sort.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `sort::name` accepted with empty credential store | Valid Value |
| EC-2 | `sort::endurance` accepted with empty credential store | Valid Value |
| EC-3 | `sort::drain` accepted with empty credential store | Valid Value |
| EC-4 | `sort::reset` accepted with empty credential store | Valid Value |
| EC-5 | `sort::bogus` exits 1 and names all valid values | Invalid Value |
| EC-6 | `sort::name` vs `sort::endurance` produces different TEXT row order | Behavioral Divergence |
| CC-1 | `sort::name` and no `sort::` produce identical JSON output | JSON No-op |

---

### EC-1: `sort::name` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage sort::name`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source:** [param/025_sort.md](../../../../docs/cli/param/025_sort.md)

---

### EC-2: `sort::endurance` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage sort::endurance`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source:** [param/025_sort.md](../../../../docs/cli/param/025_sort.md)

---

### EC-3: `sort::drain` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage sort::drain`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source:** [param/025_sort.md](../../../../docs/cli/param/025_sort.md)

---

### EC-4: `sort::reset` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage sort::reset`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source:** [param/025_sort.md](../../../../docs/cli/param/025_sort.md)

---

### EC-5: `sort::bogus` exits 1 and names all valid values

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage sort::bogus`
- **Then:** Exits 1. Stderr contains the four valid values: "name", "endurance", "drain", "reset".
- **Exit:** 1
- **Source:** [feature/020_usage_sort_strategies.md AC-09](../../../../docs/feature/020_usage_sort_strategies.md)

---

### EC-6: `sort::name` vs `sort::endurance` produces different TEXT row order

- **Behavioral Divergence:** `sort::name` orders rows alphabetically; `sort::endurance` moves the qualified account to the top regardless of name. Two accounts where the endurance-qualified account is not alphabetically first will produce different TEXT output under the two strategies.
- **Given:** Two `AccountQuota` structs: `z_qualified@test.com` (5h_reset within 30 min, `7d Left`=40% — endurance-qualified) and `a_plain@test.com` (5h_reset in 3h — endurance-unqualified). Input order: `a_plain`, `z_qualified`.
- **When-A:** `sort_indices(&accounts, SortStrategy::Name, None, PreferStrategy::Any, now_secs)` — alphabetical.
- **When-B:** `sort_indices(&accounts, SortStrategy::Endurance, None, PreferStrategy::Any, now_secs)` — qualified first.
- **Then-A:** Row order: `a_plain@test.com`, `z_qualified@test.com`.
- **Then-B:** Row order: `z_qualified@test.com`, `a_plain@test.com` (qualified surfaced to top).
- **Exit:** n/a (unit test — function return assertion)
- **Source fn:** `test_sort_name_alphabetical`, `test_sort_endurance_default_equals_desc1` (in `src/usage.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-01, AC-02](../../../../docs/feature/020_usage_sort_strategies.md)

---

### CC-1: `sort::name` and no `sort::` produce identical JSON output

- **Behavioral Divergence:** `sort::name format::json` vs `sort::endurance format::json` — the JSON array order is identical regardless of `sort::` value (alphabetical in both cases). This contrasts with text output, where `sort::endurance` reorders rows.
- **Given:** Two saved accounts: `b@x.com` and `a@x.com` (non-alpha creation order); both with valid credential files (no accessToken — will show error rows).
- **When-A:** `clp .usage sort::name format::json`
- **When-B:** `clp .usage sort::endurance format::json`
- **Then-A:** JSON array: first element `"account":"a@x.com"`, second `"account":"b@x.com"`.
- **Then-B:** JSON array: same order as Then-A (alphabetical, unaffected by `sort::endurance`).
- **Exit:** 0 both cases
- **Source:** [feature/020_usage_sort_strategies.md AC-13](../../../../docs/feature/020_usage_sort_strategies.md)
