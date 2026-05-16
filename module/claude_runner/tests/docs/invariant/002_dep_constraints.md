# Test: Invariant — Dependency Constraints

Test case planning for [invariant/002_dep_constraints.md](../../../../docs/invariant/002_dep_constraints.md). Tests validate that structural dependency constraint invariants are maintained: no consumer workspace deps, no prohibited files, and binary deps gated by feature flag.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `cargo check --no-default-features` succeeds (no required workspace deps) | No-Default-Features |
| IT-2 | `src/routines.rs` does not exist | Prohibited File |
| IT-3 | `build.rs` does not exist in crate root | Prohibited File |
| IT-4 | `Cargo.toml` gates binary deps under `enabled` feature | Feature Gating |

## Test Coverage Summary

- No-Default-Features: 1 test (IT-1)
- Prohibited File: 2 tests (IT-2, IT-3)
- Feature Gating: 1 test (IT-4)

**Total:** 4 tests


---

### IT-1: `cargo check --no-default-features` succeeds

- **Given:** clean environment
- **When:** `cargo check --no-default-features` run against the `claude_runner` crate
- **Then:** Exit 0; no compile errors; crate has zero required workspace-level consumer dependencies when default features are disabled
- **Exit:** 0
- **Source:** [invariant/002_dep_constraints.md](../../../../docs/invariant/002_dep_constraints.md)

---

### IT-2: `src/routines.rs` does not exist

- **Given:** clean environment
- **When:** check whether `src/routines.rs` exists in the `claude_runner` crate
- **Then:** File does not exist; `routines.rs` is a prohibited filename per dependency constraints
- **Exit:** N/A (filesystem assertion)
- **Source:** [invariant/002_dep_constraints.md](../../../../docs/invariant/002_dep_constraints.md)

---

### IT-3: `build.rs` does not exist in crate root

- **Given:** clean environment
- **When:** check whether `build.rs` exists at the `claude_runner` crate root
- **Then:** File does not exist; `build.rs` is prohibited per dependency constraints
- **Exit:** N/A (filesystem assertion)
- **Source:** [invariant/002_dep_constraints.md](../../../../docs/invariant/002_dep_constraints.md)

---

### IT-4: `Cargo.toml` gates binary deps under `enabled` feature

- **Given:** clean environment
- **When:** inspect `Cargo.toml` of the `claude_runner` crate
- **Then:** Any binary-only dependencies appear exclusively under `[features]` `enabled` feature or equivalent gating; no binary dep is unconditionally required
- **Exit:** N/A (static inspection)
- **Source:** [invariant/002_dep_constraints.md](../../../../docs/invariant/002_dep_constraints.md)
