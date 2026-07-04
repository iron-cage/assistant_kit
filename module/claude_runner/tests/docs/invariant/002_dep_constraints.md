# Test: Invariant — Dependency Constraints

Test case planning for [invariant/002_dep_constraints.md](../../../docs/invariant/002_dep_constraints.md). Tests validate that structural dependency constraint invariants are maintained: no consumer workspace deps, no prohibited files, and binary deps gated by feature flag.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IN-1 | `cargo check --no-default-features` succeeds (no required workspace deps) | No-Default-Features |
| IN-2 | `src/routines.rs` does not exist | Prohibited File |
| IN-3 | `build.rs` does not exist in crate root | Prohibited File |
| IN-4 | `Cargo.toml` gates binary deps under `enabled` feature | Feature Gating |

## Test Coverage Summary

- No-Default-Features: 1 test (IN-1)
- Prohibited File: 2 tests (IN-2, IN-3)
- Feature Gating: 1 test (IN-4)

**Total:** 4 tests


---

### IN-1: `cargo check --no-default-features` succeeds

- **Given:** clean environment
- **When:** `cargo check --no-default-features` run against the `claude_runner` crate
- **Then:** Exit 0; no compile errors; crate has zero required workspace-level consumer dependencies when default features are disabled
- **Exit:** 0
- **Source:** [invariant/002_dep_constraints.md](../../../docs/invariant/002_dep_constraints.md)

---

### IN-2: `src/routines.rs` does not exist

- **Given:** clean environment
- **When:** check whether `src/routines.rs` exists in the `claude_runner` crate
- **Then:** File does not exist; `routines.rs` is a prohibited filename per dependency constraints
- **Exit:** 0
- **Source:** [invariant/002_dep_constraints.md](../../../docs/invariant/002_dep_constraints.md)

---

### IN-3: `build.rs` does not exist in crate root

- **Given:** clean environment
- **When:** check whether `build.rs` exists at the `claude_runner` crate root
- **Then:** File does not exist; `build.rs` is prohibited per dependency constraints
- **Exit:** 0
- **Source:** [invariant/002_dep_constraints.md](../../../docs/invariant/002_dep_constraints.md)

---

### IN-4: `Cargo.toml` gates binary deps under `enabled` feature

- **Given:** clean environment
- **When:** inspect `Cargo.toml` of the `claude_runner` crate
- **Then:** Any binary-only dependencies appear exclusively under `[features]` `enabled` feature or equivalent gating; no binary dep is unconditionally required
- **Exit:** 0
- **Source:** [invariant/002_dep_constraints.md](../../../docs/invariant/002_dep_constraints.md)
