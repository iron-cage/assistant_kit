# Test: `sort::` Parameter

Edge case coverage for the `sort::` parameter on `.usage`. See [param/025_sort.md](../../../../docs/cli/param/025_sort.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `sort::name` accepted with empty credential store | Valid Value |
| EC-2 | `sort::endurance` accepted with empty credential store | Valid Value |
| EC-3 | `sort::drain` accepted with empty credential store | Valid Value |
| EC-4 | `sort::renew` accepted with empty credential store | Valid Value |
| EC-5 | `sort::bogus` exits 1 and names all five valid values | Invalid Value |
| EC-7 | `sort::next` accepted with empty credential store | Valid Value |
| EC-8 | `sort::name` and no `sort::` produce identical JSON output | JSON No-op |

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

### EC-4: `sort::renew` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage sort::renew`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source:** [param/025_sort.md](../../../../docs/cli/param/025_sort.md)

---

### EC-5: `sort::bogus` exits 1 and names all five valid values

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage sort::bogus`
- **Then:** Exits 1. Stderr contains all five valid values: "name", "endurance", "drain", "renew", "next".
- **Exit:** 1
- **Source:** [feature/020_usage_sort_strategies.md AC-09](../../../../docs/feature/020_usage_sort_strategies.md)

---

### EC-7: `sort::next` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage sort::next`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source:** [feature/020_usage_sort_strategies.md AC-15](../../../../docs/feature/020_usage_sort_strategies.md)

---

> **Note:** EC-6 removed — unit test of `sort_indices()` function return not directly observable via clp output — behavior only verifiable at unit-test level. Unit tests live in `src/usage/sort.rs` as `test_sort_name_alphabetical` and `test_sort_endurance_default_equals_desc1`.

---

### EC-8: `sort::name` and no `sort::` produce identical JSON output

- **Behavioral Divergence:** `sort::name format::json` vs `sort::endurance format::json` — the JSON array order is identical regardless of `sort::` value (alphabetical in both cases). This contrasts with text output, where `sort::endurance` reorders rows.
- **Given:** Two saved accounts: `b@x.com` and `a@x.com` (non-alpha creation order); both with valid credential files (no accessToken — will show error rows).
- **When-A:** `clp .usage sort::name format::json`
- **When-B:** `clp .usage sort::endurance format::json`
- **Then-A:** JSON array: first element `"account":"a@x.com"`, second `"account":"b@x.com"`.
- **Then-B:** JSON array: same order as Then-A (alphabetical, unaffected by `sort::endurance`).
- **Exit:** 0 both cases
- **Source:** [feature/020_usage_sort_strategies.md AC-13](../../../../docs/feature/020_usage_sort_strategies.md)
