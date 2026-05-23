# Test: `desc::` Parameter

Edge case coverage for the `desc::` parameter on `.usage`. See [param/026_desc.md](../../../../docs/cli/param/026_desc.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `desc::0` accepted with empty store | Valid Value |
| EC-2 | `desc::1` accepted with empty store | Valid Value |
| EC-3 | `desc::2` rejected (invalid bool) | Invalid Value |
| CC-1 | `sort::name desc::0` identical to `sort::name` (ascending default) | Context Default |
| CC-2 | `sort::name desc::1` reverses `sort::name` alphabetical order | Direction Override |

---

### EC-1: `desc::0` accepted with empty store

- **Given:** Empty credential store.
- **When:** `clp .usage desc::0`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source:** [param/026_desc.md](../../../../docs/cli/param/026_desc.md)

---

### EC-2: `desc::1` accepted with empty store

- **Given:** Empty credential store.
- **When:** `clp .usage desc::1`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source:** [param/026_desc.md](../../../../docs/cli/param/026_desc.md)

---

### EC-3: `desc::2` rejected — invalid bool

- **Given:** Any environment.
- **When:** `clp .usage desc::2`
- **Then:** Exits 1. Stderr is non-empty (invalid argument).
- **Exit:** 1
- **Source:** [param/026_desc.md](../../../../docs/cli/param/026_desc.md)

---

### CC-1: `sort::name desc::0` identical to `sort::name` (ascending default)

- **Behavioral Divergence:** `sort::name` (implicit `desc::0`) vs `sort::name desc::1` produce different row orders. This case confirms the *same* direction.
- **Given:** Two saved accounts: `b@x.com` and `a@x.com`; both with credential files (no accessToken — error rows, but still rendered).
- **When-A:** `clp .usage sort::name`
- **When-B:** `clp .usage sort::name desc::0`
- **Then-A:** Rows appear `a@x.com`, `b@x.com` (alphabetical ascending).
- **Then-B:** Identical row order to Then-A.
- **Exit:** 0 both cases
- **Source:** [feature/020_usage_sort_strategies.md AC-06](../../../../docs/feature/020_usage_sort_strategies.md)

---

### CC-2: `sort::name desc::1` reverses alphabetical order

- **Behavioral Divergence:** `sort::name desc::0` produces A→Z; `sort::name desc::1` produces Z→A — different row positions for the same two accounts.
- **Given:** Two saved accounts: `a@x.com` and `z@x.com`; both with credential files.
- **When-A:** `clp .usage sort::name desc::0`
- **When-B:** `clp .usage sort::name desc::1`
- **Then-A:** Row order: `a@x.com` before `z@x.com`.
- **Then-B:** Row order: `z@x.com` before `a@x.com` (reversed).
- **Exit:** 0 both cases
- **Source:** [feature/020_usage_sort_strategies.md AC-05](../../../../docs/feature/020_usage_sort_strategies.md)
