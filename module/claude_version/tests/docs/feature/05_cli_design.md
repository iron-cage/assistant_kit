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

## Test Coverage Summary

- Unknown Param: 1 test (FT-1)
- Missing Value: 1 test (FT-2)
- Last-Wins: 1 test (FT-3)
- Empty Argv: 1 test (FT-4)
- Help Anywhere: 1 test (FT-5)

**Total:** 5 tests

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
- **When:** `cm` (no arguments)
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
| `ft005_1_unknown_param_exits_1` | `integration/feature_surface_test.rs` |
| `ft005_2_empty_bool_param_value_exits_1` | `integration/feature_surface_test.rs` |
| `ft005_3_last_param_wins` | `integration/feature_surface_test.rs` |
| `tc093_empty_args_exits_0` | `integration/framework_test.rs` |
| `tc04_help_anywhere_wins` | `integration/read_commands_test.rs` |
| `dd01_001_bool_true_rejected` | `integration/catalog_surface_test.rs` |
| `dd02_001_last_v_wins` | `integration/catalog_surface_test.rs` |
| `dd03_001_cmd_not_implemented_exit2` | `integration/catalog_surface_test.rs` |
| `dd04_001_per_cmd_validation` | `integration/catalog_surface_test.rs` |

---

## Design Decision Tests

Verify that testable design decisions (D3, D4, D7, D8) are enforced. Non-testable decisions (D1, D2, D5, D6) are structural and have no directly observable test surface.

| DD | Scenario | Decision |
|----|----------|----------|
| DD-1 | `dry::1` accepted; `dry::true` rejected with exit 1 | D3 |
| DD-2 | Repeated `v::` parameter: last occurrence wins | D8 |
| DD-3 | `CommandNotImplemented` produces exit 2 | D4 |
| DD-4 | `format::` on `.settings.set` rejected with exit 1 | D7 |

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
