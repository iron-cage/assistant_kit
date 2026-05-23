# Test: Invariant — Command Naming

Test case planning for [invariant/003_command_naming.md](../../../../docs/invariant/003_command_naming.md). Tests validate that commands are bare words, parameters use `--`/`-` prefix, and `KNOWN_SUBCOMMANDS` dispatch is correct.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `clr help` (bare word) → exit 0, prints usage including `ask` | Bare Word Command |
| IT-2 | `clr --help` (parameter alias) → exit 0, same output | Parameter Alias |
| IT-3 | `clr` (no args) → interactive REPL, not help | Default Dispatch |
| IT-4 | `clr run "msg"` (explicit) → dispatches `run` command | Bare Word Command |
| IT-5 | `clr unknowncmd` → exit 1, unrecognized subcommand error | Unknown Command |
| IT-6 | `KNOWN_SUBCOMMANDS` contains `ask`; no entry begins with `--` | Naming Invariant |

## Test Coverage Summary

- Bare Word Command: 2 tests (IT-1, IT-4)
- Parameter Alias: 1 test (IT-2)
- Default Dispatch: 1 test (IT-3)
- Unknown Command Rejection: 1 test (IT-5)
- Naming Invariant: 1 test (IT-6)

**Total:** 6 tests

---

### IT-1: `clr help` (bare word) → exit 0, prints usage including `ask`

- **Given:** clean environment
- **When:** `clr help`
- **Then:** exit 0; stdout contains usage information listing `run`, `isolated`, `refresh`, `ask`, `help`; bare word dispatch works
- **Exit:** 0
- **Source:** [invariant/003_command_naming.md](../../../../docs/invariant/003_command_naming.md), [001_command.md — help](../../../../docs/cli/001_command.md#command--4-help)

---

### IT-2: `clr --help` (parameter alias) → exit 0, same output

- **Given:** clean environment
- **When:** `clr --help` (also: `clr -h`)
- **Then:** exit 0; stdout matches `clr help` output; `--help`/`-h` are parameter aliases that trigger identical help behavior
- **Exit:** 0
- **Source:** [invariant/003_command_naming.md](../../../../docs/invariant/003_command_naming.md)

---

### IT-3: `clr` (no args) → interactive REPL, not help

- **Given:** clean environment; TTY available
- **When:** `clr` (no arguments)
- **Then:** does NOT print help; enters interactive REPL mode (dispatches `run` default with no message); help requires explicit `clr help` or `clr --help`
- **Exit:** 0 (when REPL exits)
- **Source:** [invariant/003_command_naming.md](../../../../docs/invariant/003_command_naming.md), [001_command.md — help Notes](../../../../docs/cli/001_command.md#command--4-help)

---

### IT-4: `clr run "msg"` (explicit bare word) → dispatches run command

- **Given:** clean environment
- **When:** `clr run --dry-run "Fix bug"`
- **Then:** stdout contains assembled command (same as `clr --dry-run "Fix bug"`); `run` bare-word prefix accepted and dispatched to the run command
- **Exit:** 0
- **Source:** [invariant/003_command_naming.md](../../../../docs/invariant/003_command_naming.md)

---

### IT-5: `clr unknowncmd` → exit 1, unrecognized subcommand error

- **Given:** clean environment
- **When:** `clr unknowncmd "test"`
- **Then:** exit 1; stderr contains message indicating unrecognized subcommand or similar; `KNOWN_SUBCOMMANDS` guard rejects unknown bare words
- **Exit:** 1
- **Source:** [invariant/003_command_naming.md](../../../../docs/invariant/003_command_naming.md)

---

### IT-6: `KNOWN_SUBCOMMANDS` contains `ask`; no entry begins with `--`

- **Given:** static analysis of `run_cli()` dispatch in `src/lib.rs`
- **When:** inspect `KNOWN_SUBCOMMANDS` constant
- **Then:** `KNOWN_SUBCOMMANDS` includes `ask`; none of the entries starts with `--` or `-`; all are bare words (`run`, `isolated`, `refresh`, `ask`, `help`)
- **Exit:** N/A (static check)
- **Source:** [invariant/003_command_naming.md](../../../../docs/invariant/003_command_naming.md)
