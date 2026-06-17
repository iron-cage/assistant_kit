# Collection Test: Design Decisions

### Scope

- **Purpose**: DD- test cases verifying key CLI design decisions are correctly implemented.
- **Responsibility**: Confirm that D3 (boolean 0/1), D4 (exit codes), D7 (per-command validation), and D8 (last-wins) are enforced.
- **In Scope**: Testable decisions: D3, D4, D7, D8.
- **Out of Scope**: Architectural decisions without directly observable test surface (D1, D2, D5, D6).

Collection test surface for design decisions. See [collection/001_design_decisions.md](../../../docs/collection/001_design_decisions.md) for specification.

## Test Case Index

| DD | Scenario | Decision | Source fn |
|----|----------|----------|-----------|
| DD-1 | `dry::1` accepted; `dry::true` rejected with exit 1 | D3 | ✅ |
| DD-2 | Repeated `v::` parameter: last occurrence wins | D8 | ✅ |
| DD-3 | `CommandNotImplemented` produces exit 2 | D4 | ✅ |
| DD-4 | `format::` on `.settings.set` (which doesn't accept it) rejected with exit 1 | D7 | ✅ |

**Total:** 4 tests

---

### DD-1: boolean parameters use 0/1 values only (D3)

- **Given:** the `.version.install` command with `dry` parameter
- **When:** `clv .version.install dry::true`
- **Then:** exit 1; stderr contains error indicating invalid boolean value

---

### DD-2: last `v::` occurrence wins (D8)

- **Given:** `v::` supplied twice with different values
- **When:** `clv .status v::0 v::2`
- **Then:** output matches v::2 verbosity (detailed); last value wins; exit 0

---

### DD-3: internal error produces exit 2 (D4)

- **Given:** a command invocation that triggers an internal unrecoverable error path
- **When:** `CommandNotImplemented` error is returned by the command routine
- **Then:** exit 2 (not exit 1); distinguishes internal failure from user input error

---

### DD-4: per-command parameter validation rejects unknown params (D7)

- **Given:** `format::` is not accepted by `.settings.set`
- **When:** `clv .settings.set format::json key::k value::v`
- **Then:** exit 1; stderr contains error indicating `format` is not valid for this command

---

### Source Functions

| Function | File |
|----------|------|
| `dd01_001_bool_true_rejected` | `integration/collection_surface_test.rs` |
| `dd02_001_last_v_wins` | `integration/collection_surface_test.rs` |
| `dd03_001_cmd_not_implemented_exit2` | `integration/collection_surface_test.rs` |
| `dd04_001_per_cmd_validation` | `integration/collection_surface_test.rs` |
