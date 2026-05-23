# Test: `prefer::` Parameter

Edge case coverage for the `prefer::` parameter on `.usage`. See [param/027_prefer.md](../../../../docs/cli/param/027_prefer.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `prefer::any` accepted with empty store | Valid Value |
| EC-2 | `prefer::opus` accepted with empty store | Valid Value |
| EC-3 | `prefer::sonnet` accepted with empty store | Valid Value |
| EC-4 | `prefer::bogus` exits 1 and names valid values | Invalid Value |
| CC-1 | `prefer::` without `sort::` accepted (no-op on default sort::name) | Isolation |

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
- **Source:** [feature/020_usage_sort_strategies.md AC-10](../../../../docs/feature/020_usage_sort_strategies.md)

---

### CC-1: `prefer::` without `sort::` accepted and does not break default output

- **Behavioral Divergence:** `prefer::sonnet sort::endurance` uses `7d(Son)` for qualification; `prefer::any sort::endurance` uses `min(7d Left, 7d(Son))`. The divergence is in which weekly column governs endurance classification — not tested here (unit-level); this case just confirms `prefer::` is accepted with `sort::name` (the default) without error.
- **Given:** Empty credential store.
- **When:** `clp .usage prefer::sonnet`
- **Then:** Exits 0 with "(no accounts configured)". `prefer::` is parsed silently — it only affects sort heuristics, not name/default ordering.
- **Exit:** 0
- **Source:** [param/027_prefer.md](../../../../docs/cli/param/027_prefer.md)
