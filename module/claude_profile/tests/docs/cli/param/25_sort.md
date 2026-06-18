# Test: `sort::` Parameter

Edge case coverage for the `sort::` parameter on `.usage`. See [param/025_sort.md](../../../../docs/cli/param/025_sort.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `sort::name` accepted with empty credential store | Valid Value |
| EC-2 | `sort::renew` accepted with empty credential store | Valid Value |
| EC-3 | `sort::renews` accepted with empty credential store | Valid Value |
| EC-4 | `sort::bogus` exits 1 and names all three valid values | Invalid Value |
| EC-5 | `sort::endurance` rejected — exits 1 | Rejected Value |
| EC-6 | `sort::drain` rejected — exits 1 | Rejected Value |
| EC-7 | `sort::next` rejected — exits 1 | Rejected Value |
| EC-8 | `sort::expires` rejected — exits 1 | Rejected Value |
| EC-9 | `sort::name` and no `sort::` produce identical JSON output | JSON No-op |

---

### EC-1: `sort::name` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage sort::name`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source:** [param/025_sort.md](../../../../docs/cli/param/025_sort.md)

---

### EC-2: `sort::renew` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage sort::renew`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source:** [param/025_sort.md](../../../../docs/cli/param/025_sort.md)

---

### EC-3: `sort::renews` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage sort::renews`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source:** [param/025_sort.md](../../../../docs/cli/param/025_sort.md)

---

### EC-4: `sort::bogus` exits 1 and names all three valid values

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage sort::bogus`
- **Then:** Exits 1. Stderr contains all three valid values: "name", "renew", "renews".
- **Exit:** 1
- **Source:** [feature/020_usage_sort_strategies.md AC-09](../../../../docs/feature/020_usage_sort_strategies.md)

---

### EC-5: `sort::endurance` rejected — exits 1

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage sort::endurance`
- **Then:** Exits 1. Stderr names valid values; `endurance` is not among them.
- **Exit:** 1
- **Source:** [feature/020_usage_sort_strategies.md AC-09](../../../../docs/feature/020_usage_sort_strategies.md)

---

### EC-6: `sort::drain` rejected — exits 1

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage sort::drain`
- **Then:** Exits 1. Stderr names valid values; `drain` is not among them.
- **Exit:** 1
- **Source:** [feature/020_usage_sort_strategies.md AC-09](../../../../docs/feature/020_usage_sort_strategies.md)

---

### EC-7: `sort::next` rejected — exits 1

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage sort::next`
- **Then:** Exits 1. Stderr names valid values; `next` is not among them.
- **Exit:** 1
- **Source:** [feature/020_usage_sort_strategies.md AC-09](../../../../docs/feature/020_usage_sort_strategies.md)

---

### EC-8: `sort::expires` rejected — exits 1

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage sort::expires`
- **Then:** Exits 1. Stderr names valid values; `expires` is not among them.
- **Exit:** 1
- **Source:** [feature/020_usage_sort_strategies.md AC-09](../../../../docs/feature/020_usage_sort_strategies.md)

---

### EC-9: `sort::name` and no `sort::` produce identical JSON output

- **Behavioral Note:** `sort::name format::json` vs `sort::renew format::json` — the JSON array order is identical regardless of `sort::` value (alphabetical in both cases). This contrasts with text output, where `sort::renew` reorders rows.
- **Given:** Two saved accounts: `b@x.com` and `a@x.com` (non-alpha creation order); both with valid credential files (no accessToken — will show error rows).
- **When-A:** `clp .usage sort::name format::json`
- **When-B:** `clp .usage sort::renew format::json`
- **Then-A:** JSON array: first element `"account":"a@x.com"`, second `"account":"b@x.com"`.
- **Then-B:** JSON array: same order as Then-A (alphabetical, unaffected by `sort::renew`).
- **Exit:** 0 both cases
- **Source:** [feature/020_usage_sort_strategies.md AC-13](../../../../docs/feature/020_usage_sort_strategies.md)
