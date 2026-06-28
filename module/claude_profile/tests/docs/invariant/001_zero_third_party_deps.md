# Test: Invariant 001 — Zero Third-Party Dependencies

Property assertion cases for `docs/invariant/001_zero_third_party_deps.md`. Verifies that the
`claude_profile` library path carries zero crates.io deps and that the `enabled` feature
introduces only the four permitted workspace crates.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IN-1 | Library path cargo tree shows zero crates.io entries | Invariant holds (normal) |
| IN-2 | Enabled feature path contains only the four permitted workspace deps | Invariant holds (boundary) |

**Total:** 2 IN cases

---

### IN-1: Library path cargo tree shows zero crates.io entries

- **Given:** The `claude_profile` crate at the current HEAD with no feature flags activated
- **When:** `cargo tree --no-dev-dependencies` is run without `--features enabled`
- **Then:** The dependency tree output contains zero crates.io entries; only internal workspace
  crates (`claude_core`, `claude_profile_core`, or equivalent workspace members) appear as
  dependencies — no external registry packages are listed
- **Source:** [docs/invariant/001_zero_third_party_deps.md](../../../docs/invariant/001_zero_third_party_deps.md)

---

### IN-2: Enabled feature path contains only the four permitted workspace deps

- **Given:** The `claude_profile` crate with `--features enabled` activated
- **When:** `cargo tree --no-dev-dependencies --features enabled` is run
- **Then:** Every listed dependency is one of the four permitted workspace crates (`unilang`,
  `error_tools`, `claude_quota`, `data_fmt`) or a transitive dep of those crates; no unexpected
  crates.io entries appear beyond what those four crates bring in
- **Source:** [docs/invariant/001_zero_third_party_deps.md](../../../docs/invariant/001_zero_third_party_deps.md)
