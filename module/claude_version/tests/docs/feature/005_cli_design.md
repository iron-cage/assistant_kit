# Feature Test: CLI Design

### Scope

- **Purpose**: FT- test cases for the 5-phase unilang pipeline, parameter validation, and exit codes.
- **Responsibility**: Acceptance criteria verifying unknown param rejection, missing value rejection, last-wins semantics, empty argv help, and help-anywhere behavior.
- **In Scope**: Unknown parameter exit 1, missing value exit 1, last-wins for repeated params, empty argv → help, `.help` anywhere → help.
- **Out of Scope**: Individual command semantics (-> other feature/ instances), type inference (-> `../../algorithm/`).

Feature test surface for CLI design. See [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md) for specification.

## Behavioral Divergence Pair

Two valid invocations with different verbosity produce distinct output lengths:

- **Input A:** `cm .version.install version::stable v::0 dry::1` → brief output (v::0 suppresses extra lines)
- **Input B:** `cm .version.install version::stable v::2 dry::1` → verbose output (v::2 adds full detail)

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
- **When:** `cm .status bogus::x`
- **Then:** exit 1; stderr contains `"bogus"`
- **Exit:** 1
- **Source:** [feature/005_cli_design.md — Parameter rules](../../../../docs/feature/005_cli_design.md)

---

### FT-2: Parameter present but value missing (`dry::`) → exit 1

- **Given:** clean environment
- **When:** `cm .version.install dry::`
- **Then:** exit 1; stderr or stdout contains error about `"dry"`
- **Exit:** 1
- **Source:** [feature/005_cli_design.md — Parameter rules](../../../../docs/feature/005_cli_design.md)

---

### FT-3: Repeated parameter → last occurrence wins

- **Given:** clean environment
- **When:** `cm .version.install version::stable version::month dry::1`
- **Then:** stdout contains `"2.1.74"` (month resolution wins); exit 0
- **Exit:** 0
- **Source:** [feature/005_cli_design.md — Parameter rules: last occurrence wins](../../../../docs/feature/005_cli_design.md)

---

### FT-4: Empty argv → help output, exit 0

- **Given:** clean environment
- **When:** `cm` (no arguments)
- **Then:** stdout is non-empty help text; exit 0
- **Exit:** 0
- **Source:** [feature/005_cli_design.md — Help listing](../../../../docs/feature/005_cli_design.md)

---

### FT-5: `.help` anywhere in argv → help output wins

- **Given:** clean environment
- **When:** `cm .status .help`
- **Then:** stdout shows help output (not `.status` output); exit 0
- **Exit:** 0
- **Source:** [feature/005_cli_design.md — Help listing](../../../../docs/feature/005_cli_design.md)

---

### Source Functions

| Function | File |
|----------|------|
| TBD (ft001_unknown_param_exits_1) | `integration/feature_surface_test.rs` |
| TBD (ft002_missing_value_exits_1) | `integration/feature_surface_test.rs` |
| TBD (ft003_last_param_wins) | `integration/feature_surface_test.rs` |
| `tc093_empty_args_exits_0` | `integration/framework_test.rs` |
| `tc04_help_anywhere_wins` | `integration/read_commands_test.rs` |
