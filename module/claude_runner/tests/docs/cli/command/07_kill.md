# Test: `kill`

Integration test planning for the `kill` command. See [command/07_kill.md](../../../../docs/cli/command/07_kill.md) for specification.

`kill` sends SIGTERM to a running Claude Code process by PID. Tests verify PID
validation, error handling for missing/invalid/non-Claude PIDs, SIGTERM delivery to a
real fake-claude process, help text, and typo guard. The fake-claude approach uses
`fake_claude_binary_dir()` + `spawn_fake_claude()` to create a detectable `/proc`-visible
ELF process named `claude`.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `clr kill` with no PID → exit 1, stderr: missing PID | Missing PID |
| IT-2 | `clr kill abc` with non-numeric PID → exit 1 | Invalid PID |
| IT-3 | `clr kill 999999` with non-Claude PID → exit 1, not-a-session message | Not a Claude process |
| IT-4 | `clr kill <pid>` with valid running claude PID → exit 0, "Sent SIGTERM" | Successful kill |
| IT-5 | `clr kill --help` → exit 0, help text present | Help flag |
| IT-6 | `clr kill -h` → exit 0, help text present | Help short flag |
| IT-7 | `clr --help` lists `kill` subcommand | Help listing |
| IT-8 | `clr kil` (typo) → exit 1, "Did you mean 'kill'?" | Typo guard |
| IT-9 | `clr kill 1234 extra` → exit 1, unexpected argument | Extra argument |
| IT-10 | `clr kill --stale` with no stale sessions → exit 0, "No stale" message | Stale: empty |
| IT-11 | `clr kill --stale` with one stale print-mode session → exit 0, killed | Stale: kill |
| IT-12 | `clr kill --stale` skips interactive sessions regardless of age | Stale: mode filter |
| IT-13 | `clr kill --stale 1234` → exit 1, mutually exclusive | Stale: conflict |
| IT-14 | `clr kill --help` lists `--stale` | Stale: help |

## Test Coverage Summary

- Missing PID: 1 test (IT-1)
- Invalid PID: 1 test (IT-2)
- Not a Claude process: 1 test (IT-3)
- Successful kill: 1 test (IT-4) — Linux/Unix only; requires fake claude process
- Help flag: 2 tests (IT-5, IT-6)
- Help listing: 1 test (IT-7)
- Typo guard: 1 test (IT-8)
- Extra argument: 1 test (IT-9)
- Stale: 5 tests (IT-10, IT-11, IT-12, IT-13, IT-14)

**Total:** 14 tests

---

### IT-1: Missing PID

- **Command:** `clr kill` (no arguments)
- **Expected behavior:** stderr contains `"missing PID"`; exit 1
- **Exit:** 1
- **Source:** [command/07_kill.md](../../../../docs/cli/command/07_kill.md)

---

### IT-2: Non-numeric PID

- **Command:** `clr kill abc`
- **Expected behavior:** stderr contains `"invalid PID"`; exit 1
- **Exit:** 1
- **Source:** [command/07_kill.md](../../../../docs/cli/command/07_kill.md)

---

### IT-3: PID not a Claude process

- **Command:** `clr kill 999999` (PID guaranteed not to be a running claude process)
- **Expected behavior:** stderr contains `"999999"` and `"not a running Claude Code session"`; exit 1
- **Exit:** 1
- **Source:** [command/07_kill.md](../../../../docs/cli/command/07_kill.md)

---

### IT-4: Successful SIGTERM delivery

- **Setup:** fake `claude` binary placed in temp dir; PATH prepended; process spawned with a 30-second sleep duration; PID recorded from spawned `Child`
- **Command:** `clr kill <pid>`
- **Expected behavior:** exit 0; stdout contains `"Sent SIGTERM"`; fake claude process terminates
- **Exit:** 0
- **Source:** [command/07_kill.md](../../../../docs/cli/command/07_kill.md)
- **Platform:** Linux/Unix only (`#[cfg(unix)]`)

---

### IT-5: `--help` flag

- **Command:** `clr kill --help`
- **Expected behavior:** exit 0; stdout contains `"SIGTERM"` and `"<PID>"`
- **Exit:** 0
- **Source:** [command/07_kill.md](../../../../docs/cli/command/07_kill.md)

---

### IT-6: `-h` short flag

- **Command:** `clr kill -h`
- **Expected behavior:** exit 0; stdout contains `"SIGTERM"` and `"<PID>"`
- **Exit:** 0
- **Source:** [command/07_kill.md](../../../../docs/cli/command/07_kill.md)

---

### IT-7: `clr --help` lists `kill`

- **Command:** `clr --help`
- **Expected behavior:** stdout contains `kill`; exit 0
- **Exit:** 0
- **Source:** [command/07_kill.md](../../../../docs/cli/command/07_kill.md)

---

### IT-8: Typo `clr kil`

- **Command:** `clr kil`
- **Expected behavior:** stderr contains `"Did you mean"`; exit 1
- **Exit:** 1
- **Source:** [command/07_kill.md](../../../../docs/cli/command/07_kill.md)

---

### IT-9: Extra argument

- **Command:** `clr kill 1234 extra`
- **Expected behavior:** stderr contains `"unexpected argument"`; exit 1
- **Exit:** 1
- **Source:** [command/07_kill.md](../../../../docs/cli/command/07_kill.md)

---

### IT-10: `--stale` with no stale sessions

- **Setup:** no running claude processes (or all are recent / interactive)
- **Command:** `clr kill --stale`
- **Expected behavior:** exit 0; stdout contains `"No stale print-mode sessions found"`
- **Exit:** 0
- **Source:** [command/07_kill.md](../../../../docs/cli/command/07_kill.md)

---

### IT-11: `--stale` kills stale print-mode session

- **Setup:** fake `claude` process spawned with print-mode args (`--print`); process age simulated > 3600s via `_CLR_STALE_THRESHOLD=1` test override (kills sessions older than 1s); short sleep to ensure age > 1s
- **Command:** `clr kill --stale` with `_CLR_STALE_THRESHOLD=1`
- **Expected behavior:** exit 0; stdout contains `"Killed PID"` with the fake claude PID; fake process is dead after command completes
- **Exit:** 0
- **Source:** [command/07_kill.md](../../../../docs/cli/command/07_kill.md)
- **Platform:** Linux/Unix only (`#[cfg(unix)]`)

---

### IT-12: `--stale` skips interactive sessions

- **Setup:** fake `claude` process spawned WITHOUT `--print` (interactive mode); process age > threshold
- **Command:** `clr kill --stale` with `_CLR_STALE_THRESHOLD=1`
- **Expected behavior:** exit 0; stdout contains `"No stale print-mode sessions found"`; fake interactive claude process is still alive
- **Exit:** 0
- **Source:** [command/07_kill.md](../../../../docs/cli/command/07_kill.md)
- **Platform:** Linux/Unix only (`#[cfg(unix)]`)

---

### IT-13: `--stale` with PID is mutually exclusive

- **Command:** `clr kill --stale 1234`
- **Expected behavior:** exit 1; stderr indicates `--stale` and `<PID>` are mutually exclusive
- **Exit:** 1
- **Source:** [command/07_kill.md](../../../../docs/cli/command/07_kill.md)

---

### IT-14: `--help` lists `--stale`

- **Command:** `clr kill --help`
- **Expected behavior:** exit 0; stdout contains `"--stale"`
- **Exit:** 0
- **Source:** [command/07_kill.md](../../../../docs/cli/command/07_kill.md)
