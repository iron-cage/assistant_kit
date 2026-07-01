# Test: `prefer::` Parameter

Edge case coverage for the `prefer::` parameter on `.usage`. See [param/027_prefer.md](../../../../docs/cli/param/027_prefer.md) for specification.

**Behavioral Divergence Pair:** EC-1 ↔ EC-4 — `prefer::any` (valid value) exits 0 with normal output; `prefer::bogus` (invalid value) exits 1 with error listing all three valid values.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `prefer::any` accepted with empty store | Valid Value |
| EC-2 | `prefer::opus` accepted with empty store | Valid Value |
| EC-3 | `prefer::sonnet` accepted with empty store | Valid Value |
| EC-4 | `prefer::bogus` exits 1 and names valid values | Invalid Value |
| EC-5 | `prefer::` without `sort::` accepted (no-op with default sort::renew on empty store) | Isolation |
| EC-6 | `prefer::opus` vs `prefer::sonnet` select different `-> Next` accounts when quota profiles differ | Behavioral Divergence |

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

---

### EC-6: `prefer::opus` vs `prefer::sonnet` select different `-> Next` recommendations

- **Behavioral Divergence:** Two valid `prefer::` values produce observably different footer `-> Next` recommendations when two accounts tie on renewal time but differ in quota profile.
- **Given:** Two accounts in the same renewal group (same `renewal_event_secs`, so tiebreak applies). Account P: high `7d Left` (overall quota), low `7d(Son) Left` (sonnet quota). Account Q: moderate `7d Left`, high `7d(Son) Left`.
- **When-A:** `clp .usage prefer::opus`
- **When-B:** `clp .usage prefer::sonnet`
- **Then-A:** Footer `-> Next` selects P — `prefer::opus` weights on overall `7d Left`; P has more.
- **Then-B:** Footer `-> Next` selects Q — `prefer::sonnet` weights on `7d(Son) Left`; Q has more.
- **Exit:** 0 both cases (unit-level assertion via `sort_indices()` in `src/usage/sort.rs`)
- **Source fn:** `test_prefer_sonnet_qualifies_by_sonnet_quota` (in `src/usage/sort.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-05/AC-06](../../../../docs/feature/020_usage_sort_strategies.md)
