# Parameter :: `.help`

Edge case tests for the `.help` parameter. Tests validate help trigger behavior, output stream, and stability.

### Scope

- **Purpose**: Edge case tests for the `.help` parameter.
- **Responsibility**: Help trigger behavior, output stream routing, stability, and visibility rules.
- **Commands:** all 12 commands (universal override)
- **In Scope**: Single-parameter edge cases, argv position independence, output verification.
- **Out of Scope**: Command integration (→ `../command/`), group interactions (→ `../param_group/`).

**Source:** [005_params.md](../../../../docs/cli/param/readme.md#parameter--10-help)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `.help` present → help output shown, exit 0 | Happy Path |
| EC-2 | `.help` absent → normal command processing (not help) | Default |
| EC-3 | `.help` anywhere in argv triggers help | Anywhere-In-Argv |
| EC-4 | `.help` output goes to stdout; stderr is empty | Output Stream |
| EC-5 | `.help` output is stable across repeated invocations | Stability |
| EC-6 | `.help` does not appear in its own command listing | Visibility |

## Test Coverage Summary

- Happy Path: 1 test (EC-1)
- Default: 1 test (EC-2)
- Anywhere-In-Argv: 1 test (EC-3)
- Output Stream: 1 test (EC-4)
- Stability: 1 test (EC-5)
- Visibility: 1 test (EC-6)

**Total:** 6 edge cases

**Behavioral Divergence Pair:** EC-1 (`.help` present → help listing, exit 0) ↔ EC-2 (`.help` absent → normal `.status` output, exit 0)

## Test Cases
---

### EC-1: `.help` → help output, exit 0:

- **Given:** clean environment
- **When:** `clv .help`
- **Then:** Help listing printed to stdout; exit 0
- **Exit:** 0
- **Source:** [005_params.md](../../../../docs/cli/param/readme.md#parameter--10-help)
---

### EC-2: Without `.help` → normal command processing:

- **Given:** clean environment
- **When:** `clv .status`
- **Then:** `.status` output shown (not help); exit 0
- **Exit:** 0
- **Source:** [005_params.md](../../../../docs/cli/param/readme.md#parameter--10-help)
---

### EC-3: `.help` anywhere in argv triggers help:

- **Given:** clean environment
- **When:** `clv .status .help`
- **Then:** Help output shown; `.status` command NOT executed
- **Exit:** 0
- **Source:** [005_params.md](../../../../docs/cli/param/readme.md#parameter--10-help)
---

### EC-4: `.help` output to stdout; stderr empty:

- **Given:** clean environment
- **When:** `clv .help`
- **Then:** stdout is non-empty; stderr is empty
- **Exit:** 0
- **Source:** [005_params.md](../../../../docs/cli/param/readme.md#parameter--10-help)
---

### EC-5: `.help` output stable across invocations:

- **Given:** clean environment
- **When:** `clv .help` (run 3 times)
- **Then:** All 3 stdout captures are byte-identical
- **Exit:** 0
- **Source:** [005_params.md](../../../../docs/cli/param/readme.md#parameter--10-help)
---

### EC-6: `.help` not in its own listing:

- **Given:** clean environment
- **When:** `clv .help`
- **Then:** The command listing does NOT contain a `.help` entry
- **Exit:** 0
- **Source:** [005_params.md](../../../../docs/cli/param/readme.md#parameter--10-help)

---

### Source Functions

| Function | File |
|----------|------|
| `tc001_empty_argv_shows_help` | `cli_args_test.rs` |
| `tc002_dot_help` | `cli_args_test.rs` |
| `tc026_help_subcommand_explicitly` | `cli_args_test.rs` |
| `tc038_help_in_second_position` | `cli_args_test.rs` |
| `tc039_help_after_multi_part_command` | `cli_args_test.rs` |
| `tc040_help_after_params` | `cli_args_test.rs` |
| `tc489_bare_help_after_command_routes_to_help` | `cli_args_test.rs` |
| `tc490_bare_help_after_params_routes_to_help` | `cli_args_test.rs` |
| `tc079_help_command_exits_0` | `integration/framework_test.rs` |
