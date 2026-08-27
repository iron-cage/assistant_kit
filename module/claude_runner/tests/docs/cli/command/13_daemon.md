# Test: `daemon`

Integration test planning for the `daemon` command. See [command/13_daemon.md](../../../../docs/cli/command/13_daemon.md) for specification.

`daemon` manages the single long-lived process that hosts interactive sessions. Tests
verify the help surface, the unknown-subcommand and typo guards, what `status` and `log`
report when nothing is running, the full `start`/`status`/`stop` lifecycle against a real
detached daemon, and the process-group property that detachment actually rests on.

Every test runs against its own `HOME` in a temporary directory, so a daemon started here
can neither collide with a developer's own nor contend with another test for the instance
lock. No `claude` binary is involved: none of the five subcommands spawns a session, and
sessions are `claude_daemon_core`'s own test surface (`serve_test.rs`, against real
PTY-attached children).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `clr daemon help` documents all four subcommands | Documentation |
| IT-2 | `clr daemon --help` and `-h` match the positional form | Documentation |
| IT-3 | `clr daemon restart` → exit 1, names the rejected token | Validation |
| IT-4 | `clr daemon status` with nothing running → exit 1 | Absent daemon |
| IT-5 | bare `clr daemon` is `clr daemon status` | Defaulting |
| IT-6 | `clr daemon log` prints the path and nothing else | Path contract |
| IT-7 | `clr help` lists the `daemon` usage line | Help listing |
| IT-8 | `clr daemn` (typo) → exit 1, Did you mean | Typo guard |
| IT-9 | start → status → start → stop → status → stop | Lifecycle |
| IT-10 | the started daemon is in a process group of its own | Detachment |

## Test Coverage Summary

- Documentation: 2 tests (IT-1, IT-2)
- Validation: 1 test (IT-3)
- Absent daemon: 1 test (IT-4)
- Defaulting: 1 test (IT-5)
- Path contract: 1 test (IT-6)
- Help listing: 1 test (IT-7)
- Typo guard: 1 test (IT-8)
- Lifecycle: 1 test (IT-9, covers 6 sequenced scenarios)
- Detachment: 1 test (IT-10, Linux only)

**Total:** 10 test functions

---

### IT-1: `help` documents all four subcommands

- **Command:** `clr daemon help`
- **Expected behavior:** stdout contains `clr daemon [status]`, `clr daemon start`, `clr daemon stop`, `clr daemon log` — `status` bracketed, because bare `clr daemon` is it
- **Exit:** 0
- **Source:** [command/13_daemon.md](../../../../docs/cli/command/13_daemon.md)

---

### IT-2: The flag spellings of help agree with the positional one

- **Command:** `clr daemon --help`, `clr daemon -h`
- **Expected behavior:** both produce stdout byte-identical to `clr daemon help`
- **Rationale:** three spellings that can disagree is three chances to document a subcommand in one and not the others
- **Exit:** 0
- **Source:** [command/13_daemon.md](../../../../docs/cli/command/13_daemon.md)

---

### IT-3: Unknown subcommand is rejected by name

- **Command:** `clr daemon restart`
- **Expected behavior:** stderr names `restart` and points at `clr daemon help`
- **Exit:** 1
- **Source:** [command/13_daemon.md](../../../../docs/cli/command/13_daemon.md)

---

### IT-4: `status` without a daemon exits 1

- **Setup:** `HOME` set to an empty temporary directory
- **Command:** `clr daemon status`
- **Expected behavior:** stdout reports `not running`; the exit code is the part scripts read — `clr daemon status || clr daemon start` only works because absence is a failure and not a 0 with a message
- **Exit:** 1
- **Source:** [command/13_daemon.md](../../../../docs/cli/command/13_daemon.md)

---

### IT-5: Bare `clr daemon` is `status`

- **Setup:** `HOME` set to an empty temporary directory
- **Command:** `clr daemon` and `clr daemon status`
- **Expected behavior:** identical stdout and identical exit codes
- **Exit:** 1 (both — nothing is running)
- **Source:** [command/13_daemon.md](../../../../docs/cli/command/13_daemon.md)

---

### IT-6: `log` prints only the path

- **Setup:** `HOME` set to an empty temporary directory
- **Command:** `clr daemon log`
- **Expected behavior:** stdout is exactly one line, equal to `<HOME>/.claude/-daemon/daemon.log`. `tail -f "$( clr daemon log )"` is the intended use and only works if the whole of stdout is the path — a heading or a trailing note would break it
- **Exit:** 0
- **Source:** [command/13_daemon.md](../../../../docs/cli/command/13_daemon.md)

---

### IT-7: `clr help` lists `daemon`

- **Command:** `clr help`
- **Expected behavior:** stdout contains the usage line `clr daemon [start | status | stop | log]`
- **Exit:** 0
- **Source:** [command/02_help.md](../../../../docs/cli/command/02_help.md), [command/13_daemon.md](../../../../docs/cli/command/13_daemon.md)

---

### IT-8: Typo caught by the known-subcommand guard

- **Command:** `clr daemn`
- **Expected behavior:** exit 1; stderr contains `Did you mean` and suggests `daemon`. Without the guard, `clr daemn "..."` would be a `run` with a stray positional — starting a real session instead of reporting a typo
- **Exit:** 1
- **Source:** [command/13_daemon.md](../../../../docs/cli/command/13_daemon.md)

---

### IT-9: Full lifecycle

- **Setup:** `HOME` set to a fresh temporary directory; a drop guard stops the daemon however the test ends
- **Commands, in order:**
  1. `clr daemon start` — exit 0, stdout contains `daemon started`
  2. `clr daemon status` — exit 0, stdout contains `daemon   : running` and `sessions : 0`
  3. `clr daemon start` — exit 0, stdout contains `already running` (idempotent, not a lock error)
  4. `clr daemon stop` — exit 0, stdout contains `daemon stopped`
  5. `clr daemon status` — exit 1
  6. `clr daemon stop` — exit 0, stdout contains `not running` (idempotent)
- **Also asserts:** after the stop, `<HOME>/.claude/-daemon/daemon.sock` is gone and `daemon.log` remains — a daemon that keeps dying at startup is only debuggable if stopping does not erase what it wrote
- **Rationale for one test rather than six:** each would otherwise pay for its own daemon, and the sequence *is* the contract
- **Source:** [command/13_daemon.md](../../../../docs/cli/command/13_daemon.md)

---

### IT-10: The daemon has its own process group

- **Setup:** `HOME` set to a fresh temporary directory; `clr daemon start`
- **Command:** read the daemon's pid from its own `listening on … (pid N)` log line, then read `/proc/<pid>/stat`
- **Expected behavior:** the `pgrp` field equals the pid — the daemon leads a process group of its own. This is the property that makes Ctrl-C in the starting shell not reach it: a terminal signals its *foreground process group*, and the daemon is not in it
- **Rationale:** read rather than inferred, because `CommandExt::process_group( 0 )` is the whole of the mechanism, and a silent regression there looks exactly like a working daemon until the day a terminal closes
- **Parsing note:** `comm` in `/proc/<pid>/stat` is parenthesised and may itself contain spaces and parens, so the numeric fields start after its *last* closing paren, never at a fixed index
- **Exit:** 0
- **Platform:** Linux only (`/proc`)
- **Source:** [command/13_daemon.md](../../../../docs/cli/command/13_daemon.md)
