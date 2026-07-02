# Test: `.help` (universal help override)

Edge case coverage for the `.help` parameter. See [param/readme.md](../../../../docs/cli/param/readme.md) for specification.

### Scope

- **Purpose**: Edge case tests for the `.help` universal override parameter.
- **Responsibility**: Verifies that `.help` triggers help display and exits regardless of which command or other parameters are present.
- **Commands:** all commands (universal override)
- **In Scope**: Override behavior, position independence, side-effect suppression, command scope.
- **Out of Scope**: Help content correctness (→ `../command/01_help.md`), group interactions (→ `../param_group/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `.help` alone → help shown, exit 0 | Basic Invocation |
| EC-2 | `.help` combined with read command → overrides command | Override |
| EC-3 | `.help` combined with mutation command → no side effects | Override + Guard |
| EC-4 | `.help` position independence — first arg vs last arg | Position |
| EC-5 | Default (absent) → help not triggered | Default Behavior |
| EC-6 | `.help` output contains command names or usage text | Content Check |
| EC-7 | `.help` universally accepted by all commands | Command Scope |
| EC-8 | `.help` with other params — still shows help, ignores params | Override Priority |

## Test Coverage Summary

- Basic Invocation: 1 test
- Override behavior: 3 tests
- Position independence: 1 test
- Default behavior: 1 test
- Content validation: 1 test
- Command scope: 1 test

**Total:** 8 edge cases

**Behavioral Divergence Pair:** EC-2 (`.help` present → command not executed, exit 0) ↔ EC-5 (absent → command executes normally)

---

### EC-1: `.help` alone → help shown, exit 0

- **Given:** clean environment
- **When:** `clv .help`
- **Then:** exit 0; stdout contains help text; no error on stderr
- **Exit:** 0
- **Source:** [param/readme.md — .help standalone](../../../../docs/cli/param/readme.md)

---

### EC-2: `.help` combined with read command → overrides command

- **Given:** clean environment
- **When:** `clv .version.list .help`
- **Then:** exit 0; help text shown instead of version list output; `.help` overrides the command
- **Exit:** 0
- **Source:** [param/10_help.md — universal override](../../../../docs/cli/param/10_help.md)

---

### EC-3: `.help` combined with mutation command → no side effects

- **Given:** `HOME=<tmp>` with no settings.json
- **When:** `clv .settings.set key::theme value::dark .help`
- **Then:** exit 0; help text shown; settings.json NOT created (mutation suppressed)
- **Exit:** 0
- **Source:** [param/10_help.md — universal override](../../../../docs/cli/param/10_help.md)

---

### EC-4: `.help` position independence — first vs last

- **Given:** clean environment
- **When:** run `.help` as first arg and as last arg: `clv .help .version.list` and `clv .version.list .help`
- **Then:** both produce identical help output; exit 0 in both cases
- **Exit:** 0
- **Source:** [param/10_help.md — present anywhere in argv](../../../../docs/cli/param/10_help.md)

---

### EC-5: Default (absent) → help not triggered

- **Given:** clean environment
- **When:** `clv .version.list` (no `.help` present)
- **Then:** exit 0; version list shown, NOT help text; help is not the default behavior
- **Exit:** 0
- **Source:** [param/readme.md — .help default: false](../../../../docs/cli/param/readme.md)

---

### EC-6: `.help` output contains command names or usage text

- **Given:** clean environment
- **When:** `clv .help`
- **Then:** exit 0; stdout contains at least one recognized command name (e.g., `.status`, `.version`) or the word "usage" or "commands"
- **Exit:** 0
- **Source:** [command/root.md — .help command](../../../../docs/cli/command/root.md)

---

### EC-7: `.help` accepted by all commands

- **Given:** clean environment
- **When:** `clv .settings.show .help`, `clv .processes .help`, `clv .config .help`
- **Then:** all exit 0; help text shown in each case; no "unknown parameter" error
- **Exit:** 0
- **Source:** [param/10_help.md — commands: all](../../../../docs/cli/param/10_help.md)

---

### EC-8: `.help` with other params — help wins, params ignored

- **Given:** clean environment
- **When:** `clv .version.list format::json v::0 .help`
- **Then:** exit 0; help text shown; output is NOT a JSON array (help output, not version list)
- **Exit:** 0
- **Source:** [param/10_help.md — universal override priority](../../../../docs/cli/param/10_help.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc001_empty_argv_shows_help` | `cli_args_test/help_test.rs` |
| `tc002_dot_help` | `cli_args_test/help_test.rs` |
| `tc038_help_in_second_position` | `cli_args_test/help_test.rs` |
| `tc039_help_after_multi_part_command` | `cli_args_test/help_test.rs` |
| `tc040_help_after_params` | `cli_args_test/help_test.rs` |
| `ec3_help_mutation_no_side_effects` | `cli_args_test/help_test.rs` |
| `ec4_help_position_first_arg` | `cli_args_test/help_test.rs` |
| `ec5_absent_help_not_triggered` | `cli_args_test/help_test.rs` |
| `ec6_help_output_contains_commands` | `cli_args_test/help_test.rs` |
| `ec7_help_accepted_by_all_commands` | `cli_args_test/help_test.rs` |
| `ec8_help_wins_over_params` | `cli_args_test/help_test.rs` |
