# Test: `sessions`

Integration test planning for the `sessions` command. See [command/15_sessions.md](../../../../docs/cli/command/15_sessions.md) for specification.

`sessions` lists what the daemon is hosting. Tests verify the help surface, the
unknown-option guard, both shapes of the empty answer (no daemon, and a daemon with
nothing in it), the stream discipline that keeps stdout countable, and the property that
asking the question does not change the answer.

Every test runs against its own `HOME` in a temporary directory, so a daemon started here
can neither collide with a developer's own nor contend with another test for the instance
lock. A `DaemonGuard` stops whatever was started, whatever the outcome — a leaked daemon
holding a lock under a deleted temp directory is a confusing thing to inherit in the next
run.

## What is deliberately not tested here

A listing with sessions in it. Filling one needs a real `claude` on `PATH` answering on a
real terminal, which is an end-to-end concern rather than a CLI one. The daemon's own
`serve_test.rs` covers the session table with real PTY-attached children in it, and its
`table_test.rs` covers summary ordering and keying — both against real implementations
rather than mocks.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| SC-1 | `clr sessions help` documents all three forms | Documentation |
| SC-2 | Unknown option → exit 1, names it | Validation |
| SC-3 | No daemon → stderr explains, exit 0 | Absent daemon |
| SC-4 | No daemon, `--json` → `[]`, exit 0 | Absent daemon |
| SC-5 | No daemon → stdout stays empty | Stream discipline |
| SC-6 | Daemon running, nothing hosted → `No hosted sessions.` | Empty daemon |
| SC-7 | Daemon running, nothing hosted, `--json` → `[]` | Empty daemon |
| SC-8 | Listing does not start a daemon | No side effects |

## Test Coverage Summary

- Documentation: 1 test (SC-1)
- Validation: 1 test (SC-2)
- Absent daemon: 2 tests (SC-3, SC-4)
- Stream discipline: 1 test (SC-5)
- Empty daemon: 2 tests (SC-6, SC-7)
- No side effects: 1 test (SC-8)

**Total:** 8 test functions

---

### SC-1: Help documents all three forms

- **Setup:** `HOME` set to an empty temporary directory
- **Command:** `clr sessions help`
- **Expected behavior:** stdout contains `clr sessions`, `clr sessions --json`, and `clr sessions help`
- **Exit:** 0
- **Source:** [command/15_sessions.md](../../../../docs/cli/command/15_sessions.md)

---

### SC-2: Unknown option is rejected by name

- **Setup:** `HOME` set to an empty temporary directory
- **Command:** `clr sessions --everything`
- **Expected behavior:** stderr names `--everything` and points at `clr sessions help`
- **Exit:** 1
- **Source:** [command/15_sessions.md](../../../../docs/cli/command/15_sessions.md)

---

### SC-3: No daemon is an answer, not a failure

- **Setup:** `HOME` set to an empty temporary directory
- **Command:** `clr sessions`
- **Expected behavior:** stderr contains `No session daemon is running`; exit 0, because "nothing is hosted" is a complete and correct answer to the question asked
- **Exit:** 0
- **Source:** [command/15_sessions.md](../../../../docs/cli/command/15_sessions.md)

---

### SC-4: `--json` is parseable even with no daemon

- **Setup:** `HOME` set to an empty temporary directory
- **Command:** `clr sessions --json`
- **Expected behavior:** stdout parses as JSON and equals `[]`
- **Rationale:** a consumer piping this into a parser should not have to special-case the daemon being down — an empty array is the right shape for "nothing hosted"
- **Exit:** 0
- **Source:** [command/15_sessions.md](../../../../docs/cli/command/15_sessions.md)

---

### SC-5: The explanation goes to stderr

- **Setup:** `HOME` set to an empty temporary directory
- **Command:** `clr sessions`
- **Expected behavior:** stdout is empty after trimming
- **Rationale:** `clr sessions | wc -l` must report zero sessions and not one line of prose
- **Exit:** 0
- **Source:** [command/15_sessions.md](../../../../docs/cli/command/15_sessions.md)

---

### SC-6: A running daemon with nothing in it says so plainly

- **Setup:** `HOME` set to an empty temporary directory; `clr daemon start` first; `DaemonGuard` stops it afterwards
- **Command:** `clr sessions`
- **Expected behavior:** stdout contains `No hosted sessions.` — on stdout this time, because there *is* a daemon and this is its answer rather than an explanation of its absence
- **Exit:** 0
- **Source:** [command/15_sessions.md](../../../../docs/cli/command/15_sessions.md)

---

### SC-7: The same, in JSON

- **Setup:** as SC-6
- **Command:** `clr sessions --json`
- **Expected behavior:** stdout parses as JSON and equals `[]` — the same output as SC-4, which is the point: a consumer cannot tell "no daemon" from "empty daemon", and does not need to
- **Exit:** 0
- **Source:** [command/15_sessions.md](../../../../docs/cli/command/15_sessions.md)

---

### SC-8: Asking what is hosted does not change what is hosted

- **Setup:** `HOME` set to an empty temporary directory, no daemon started; `DaemonGuard` cleans up in case one appears
- **Command:** `clr sessions`
- **Expected behavior:** `$HOME/.claude/-daemon/daemon.sock` does not exist afterwards
- **Rationale:** the distinction from `clr chat`, which does auto-start. A question that starts a process to answer itself has changed the thing it was asking about
- **Exit:** 0
- **Source:** [command/15_sessions.md](../../../../docs/cli/command/15_sessions.md)
