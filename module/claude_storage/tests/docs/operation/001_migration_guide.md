# Operation :: Migration Guide

### Scope

- **Purpose**: OP- test cases verifying each step of the `claude_storage` → `claude_storage_core` migration procedure produces its documented outcome.
- **Responsibility**: Acceptance criteria confirming Cargo.toml and import updates, successful compilation/test pass, and reversible rollback.
- **In Scope**: `Cargo.toml` dependency swap, `use claude_storage::` import replacement, post-migration build and test pass, rollback to prior state.
- **Out of Scope**: CLI usage changes (CLI users unaffected by this migration -> `../feature/001_cli_tool.md`), core library design rationale (-> `../../../../claude_storage_core/docs/feature/001_core_library.md`).

Operational test cases for the `claude_storage` → `claude_storage_core` migration procedure.
Tests validate that each procedure step produces the documented outcome and that the migration
is reversible.

**Source:** [001_migration_guide.md](../../../docs/operation/001_migration_guide.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| OP-1 | Cargo.toml updated: old dep removed, new dep added | Procedure Step 1 |
| OP-2 | Use statements updated: no `claude_storage::` imports remain | Procedure Step 2 |
| OP-3 | Crate compiles after Cargo.toml and import migration | Procedure Step 3 |
| OP-4 | Test suite passes after migration (API identical) | Procedure Step 3 |
| OP-5 | Rollback restores compilation from the previous state | Rollback |

## Test Coverage Summary

- Procedure Step 1: 1 test (OP-1)
- Procedure Step 2: 1 test (OP-2)
- Procedure Step 3: 2 tests (OP-3, OP-4)
- Rollback: 1 test (OP-5)

**Total:** 5 operation cases

## Test Cases

---

### OP-1: Cargo.toml updated: old dep removed, new dep added

- **Given:** A `Cargo.toml` that contains `claude_storage = { path = "../claude_storage" }` as a dependency
- **When:** Step 1 of the migration procedure is applied — the `claude_storage` entry is replaced with `claude_storage_core = { path = "../claude_storage_core" }`
- **Then:** `Cargo.toml` contains `claude_storage_core` dependency; the `claude_storage` dependency entry is absent; no other dependencies are modified

---

### OP-2: Use statements updated: no `claude_storage::` imports remain

- **Given:** A Rust source tree with `use claude_storage::` import statements after completing OP-1
- **When:** Step 2 of the procedure is applied — all `use claude_storage::` occurrences are replaced with `use claude_storage_core::`
- **Then:** `grep -rn "use claude_storage::" src/` returns no output; `grep -rn "use claude_storage_core::" src/` returns all previously-changed import lines

---

### OP-3: Crate compiles after Cargo.toml and import migration

- **Given:** A Rust crate where OP-1 and OP-2 have been completed; `claude_storage_core` crate available at the sibling path
- **When:** Step 3 of the procedure is applied — `cargo build` is run
- **Then:** `cargo build` exits 0; no compilation errors; no `could not find module claude_storage` errors in output

---

### OP-4: Test suite passes after migration (API identical)

- **Given:** The same crate after OP-1, OP-2, and OP-3 have succeeded
- **When:** `cargo test` is run (or `cargo nextest run`)
- **Then:** All previously-passing tests continue to pass; no test failures introduced by the migration; exit 0

---

### OP-5: Rollback restores compilation from the previous state

- **Given:** A crate where the migration (OP-1, OP-2) has been applied but the `claude_storage` crate is still available at its original path
- **When:** The rollback procedure is followed — `Cargo.toml` reverted to `claude_storage = { path = "../claude_storage" }` and all `use claude_storage_core::` statements reverted to `use claude_storage::`
- **Then:** `cargo build` exits 0 with the reverted configuration; no `claude_storage_core` references remain; the crate is in the same state as before the migration started
