# Test: `prefer::` Parameter

Edge case coverage for the `prefer::` parameter on `.usage`. See [param/027_prefer.md](../../../../docs/cli/param/027_prefer.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `prefer::any` accepted with empty store | Valid Value |
| EC-2 | `prefer::opus` accepted with empty store | Valid Value |
| EC-3 | `prefer::sonnet` accepted with empty store | Valid Value |
| EC-4 | `prefer::bogus` exits 1 and names valid values | Invalid Value |
| EC-5 | `prefer::` without `sort::` accepted (no-op with default sort::renew on empty store) | Isolation |

---

### EC-1: `prefer::any` accepted with empty store

- **Given:** Empty credential store.
- **When:** `clp .usage prefer::any`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source:** [param/027_prefer.md](../../../../docs/cli/param/027_prefer.md)

---

### EC-2: `prefer::opus` accepted with empty store

- **Given:** Empty credential store.
- **When:** `clp .usage prefer::opus`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source:** [param/027_prefer.md](../../../../docs/cli/param/027_prefer.md)

---

### EC-3: `prefer::sonnet` accepted with empty store

- **Given:** Empty credential store.
- **When:** `clp .usage prefer::sonnet`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source:** [param/027_prefer.md](../../../../docs/cli/param/027_prefer.md)

---

### EC-4: `prefer::bogus` exits 1 and names valid values

- **Given:** Any environment.
- **When:** `clp .usage prefer::bogus`
- **Then:** Exits 1. Stderr contains the three valid values: "any", "opus", "sonnet".
- **Exit:** 1
- **Source:** [feature/020_usage_sort_strategies.md AC-08](../../../../docs/feature/020_usage_sort_strategies.md)

---

### EC-5: `prefer::` without `sort::` accepted and does not break default output

- **Behavioral Note:** `prefer::` affects the `sort::renew` within-group tiebreak key and the footer recommendation eligibility gate. It does **not** affect the four-group status partition — group membership is always determined by raw `5h Left` and `7d Left` columns (AC-12), independent of `prefer::`. This case confirms `prefer::` is accepted with `sort::renew` (the default) without error.
- **Given:** Empty credential store.
- **When:** `clp .usage prefer::sonnet`
- **Then:** Exits 0 with "(no accounts configured)". `prefer::` is parsed silently — it only affects tiebreak and recommendation eligibility, not group membership or default renew sort ordering with empty store.
- **Exit:** 0
- **Source:** [param/027_prefer.md](../../../../docs/cli/param/027_prefer.md)

---

> **Note:** EC-6 removed — unit test of `sort_indices()` function return not directly observable via clp output — behavior only verifiable at unit-test level. Unit test lives in `src/usage/sort.rs` as `test_prefer_sonnet_qualifies_by_sonnet_quota`.
