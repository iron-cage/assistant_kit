# Feature Test: CLI Design

### Scope

- **Purpose**: FT- test cases for the 5-phase unilang pipeline, parameter validation, and exit codes.
- **Responsibility**: Acceptance criteria verifying unknown param rejection, missing value rejection, last-wins semantics, empty argv help, and help-anywhere behavior.
- **In Scope**: Unknown parameter exit 1, missing value exit 1, last-wins for repeated params, empty argv → help, `.help` anywhere → help.
- **Out of Scope**: Individual command semantics (-> other feature/ instances), type inference (-> `../../algorithm/`).

Feature test surface for CLI design. See [feature/005_cli_design.md](../../../docs/feature/005_cli_design.md) for specification.

## Behavioral Divergence Pair

Two valid invocations with different verbosity produce distinct output lengths:

- **Input A:** `clv .version.install version::stable v::0 dry::1` → brief output (v::0 suppresses extra lines)
- **Input B:** `clv .version.install version::stable v::2 dry::1` → verbose output (v::2 adds full detail)

Both are valid invocations; output length differs.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FT-1 | Unknown parameter → exit 1 with error mentioning param name | Unknown Param |
| FT-2 | Parameter present but value missing (`dry::`) → exit 1 | Missing Value |
| FT-3 | Repeated parameter → last occurrence wins | Last-Wins |
| FT-4 | Empty argv → help output, exit 0 | Empty Argv |
| FT-5 | `.help` anywhere in argv → help output wins | Help Anywhere |
| FT-6 | `dry::true` rejected with exit 1; boolean params use 0/1 only (D3) | Design Decision |
| FT-7 | Repeated `v::` parameter: last occurrence wins (D8) | Design Decision |
| FT-8 | `CommandNotImplemented` produces exit 2 (D4) | Design Decision |
| FT-9 | `format::` on `.settings.set` rejected with exit 1; per-command validation (D7) | Design Decision |

## Test Coverage Summary

- Unknown Param: 1 test (FT-1)
- Missing Value: 1 test (FT-2)
- Last-Wins: 2 tests (FT-3, FT-7)
- Empty Argv: 1 test (FT-4)
- Help Anywhere: 1 test (FT-5)
- Design Decision: 4 tests (FT-6, FT-7, FT-8, FT-9)

**Total:** 9 tests

---

### FT-1: Unknown parameter → exit 1 with error mentioning param name

- **Given:** clean environment
- **When:** `clv .status bogus::x`
- **Then:** exit 1; stderr contains `"bogus"`
- **Exit:** 1
- **Source:** [feature/005_cli_design.md — Parameter rules](../../../docs/feature/005_cli_design.md)

---

### FT-2: Parameter present but value missing (`dry::`) → exit 1

- **Given:** clean environment
- **When:** `clv .version.install dry::`
- **Then:** exit 1; stderr or stdout contains error about `"dry"`
- **Exit:** 1
- **Source:** [feature/005_cli_design.md — Parameter rules](../../../docs/feature/005_cli_design.md)

---

### FT-3: Repeated parameter → last occurrence wins

- **Given:** clean environment
- **When:** `clv .version.install version::stable version::month dry::1`
- **Then:** stdout contains `"2.1.74"` (month resolution wins); exit 0
- **Exit:** 0
- **Source:** [feature/005_cli_design.md — Parameter rules: last occurrence wins](../../../docs/feature/005_cli_design.md)

---

### FT-4: Empty argv → help output, exit 0

- **Given:** clean environment
- **When:** `clv` (no arguments)
- **Then:** stdout is non-empty help text; exit 0
- **Exit:** 0
- **Source:** [feature/005_cli_design.md — Help listing](../../../docs/feature/005_cli_design.md)

---

### FT-5: `.help` anywhere in argv → help output wins

- **Given:** clean environment
- **When:** `clv .status .help`
- **Then:** stdout shows help output (not `.status` output); exit 0
- **Exit:** 0
- **Source:** [feature/005_cli_design.md — Help listing](../../../docs/feature/005_cli_design.md)

---

### Source Functions

| Function | File |
|----------|------|
| `ft005_1_unknown_param_exits_1` | `tests/cli/feature_surface_test.rs` |
| `ft005_2_empty_bool_param_value_exits_1` | `tests/cli/feature_surface_test.rs` |
| `ft005_3_last_param_wins` | `tests/cli/feature_surface_test.rs` |
| `tc093_empty_args_exits_0` | `tests/cli/framework_test.rs` |
| `ft005_6_bool_true_rejected` | `tests/cli/catalog_surface_test.rs` |
| `ft005_7_last_v_wins` | `tests/cli/catalog_surface_test.rs` |
| `ft005_8_cmd_not_implemented_exit2` | `tests/cli/catalog_surface_test.rs` |
| `ft005_9_per_cmd_validation` | `tests/cli/catalog_surface_test.rs` |

---

### FT-6: boolean parameters use 0/1 values only (D3)

- **Given:** the `.version.install` command with `dry` parameter
- **When:** `clv .version.install dry::true`
- **Then:** exit 1; stderr contains error indicating invalid boolean value
- **Exit:** 1
- **Source:** [feature/005_cli_design.md — D3](../../../docs/feature/005_cli_design.md)

---

### FT-7: last `v::` occurrence wins (D8)

- **Given:** `v::` supplied twice with different values
- **When:** `clv .status v::0 v::2`
- **Then:** output matches v::2 verbosity (detailed); last value wins; exit 0
- **Exit:** 0
- **Source:** [feature/005_cli_design.md — D8](../../../docs/feature/005_cli_design.md)

---

### FT-8: internal error produces exit 2 (D4)

- **Given:** a command invocation that triggers an internal unrecoverable error path
- **When:** `CommandNotImplemented` error is returned by the command routine
- **Then:** exit 2 (not exit 1); distinguishes internal failure from user input error
- **Exit:** 2
- **Source:** [feature/005_cli_design.md — D4](../../../docs/feature/005_cli_design.md)

---

### FT-9: per-command parameter validation rejects unknown params (D7)

- **Given:** `format::` is not accepted by `.settings.set`
- **When:** `clv .settings.set format::json key::k value::v`
- **Then:** exit 1; stderr contains error indicating `format` is not valid for this command
- **Exit:** 1
- **Source:** [feature/005_cli_design.md — D7](../../../docs/feature/005_cli_design.md)
