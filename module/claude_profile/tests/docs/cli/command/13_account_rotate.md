# Command :: 13. `.account.rotate` — Integration Tests

> **REMOVED** — `.account.rotate` was a redirector (Feature 016) that always exited 1 with a migration message. Task 409 deleted the redirector function backing `.account.rotate` and its registration entirely; rotation lives at `.usage rotate::1` (Feature 038). `clp .account.rotate` now fails at the CLI-parse stage with a generic unknown-command error, identical to any other unrecognized command name — none of the redirector-specific behavior IT-1 through IT-7 below documented is observable any longer. See [command_verb/006_rotate.md](../../../../docs/cli/command_verb/006_rotate.md).

| # | Test | Conditions | Exit |
|---|------|-----------|------|
| IT-1 | N/A — redirector fully removed (Task 409) | See N/A entry below | — |
| IT-2 | N/A — redirector fully removed (Task 409) | See N/A entry below | — |
| IT-3 | N/A — redirector fully removed (Task 409) | See N/A entry below | — |
| IT-4 | N/A — redirector fully removed (Task 409) | See N/A entry below | — |
| IT-5 | N/A — redirector fully removed (Task 409) | See N/A entry below | — |
| IT-6 | N/A — redirector fully removed (Task 409) | See N/A entry below | — |
| IT-7 | N/A — redirector fully removed (Task 409) | See N/A entry below | — |
| IT-8 | N/A — redirector fully removed (Task 409) | See N/A entry below | — |

---

### IT-1: N/A — redirector fully removed (Task 409)

> **N/A** — This case verified that the redirector always exited 1 for any invocation. Task 409 deleted the redirector function backing `.account.rotate` and its registration entirely; the command now fails at the CLI-parse stage with a generic unknown-command error before any command-specific code runs, so there is no redirector behavior left to assert.
> Becomes testable when: no committed task.

---

### IT-2: N/A — redirector fully removed (Task 409)

> **N/A** — This case verified the error message referenced `.usage rotate`. The redirector that emitted that message no longer exists; the generic unknown-command error is framework-produced and identical across every unregistered command name, carrying no `.usage rotate`-specific text.
> Becomes testable when: no committed task.

---

### IT-3: N/A — redirector fully removed (Task 409)

> **N/A** — This case verified no account-store mutation occurred on the redirector's exit-1 path. With the command fully deregistered, the CLI-parse stage rejects the invocation before any command routine — redirector or otherwise — could run, so there is no command-specific code path left whose mutation behavior could be asserted.
> Becomes testable when: no committed task.

---

### IT-4: N/A — redirector fully removed (Task 409)

> **N/A** — This case verified `dry::1` was accepted but ignored because the redirector was registered with zero declared parameters. Registration itself is gone — `dry::1` is no longer a recognized parameter for a nonexistent command, and the CLI-parse-stage rejection does not distinguish between argument sets.
> Becomes testable when: no committed task.

---

### IT-5: N/A — redirector fully removed (Task 409)

> **N/A** — This case verified the redirector exited 1 (not the pre-deprecation algorithm's exit 2) regardless of account-store shape. There is no command-specific exit code left to distinguish — every invocation now fails identically at the CLI-parse stage regardless of how many accounts exist.
> Becomes testable when: no committed task.

---

### IT-6: N/A — redirector fully removed (Task 409)

> **N/A** — This case verified the exact literal migration-hint message text. The redirector function that produced that exact string has been deleted from `src/registry.rs`; no code path in the crate emits that message any longer.
> Becomes testable when: no committed task.

---

### IT-7: N/A — redirector fully removed (Task 409)

> **N/A** — This case verified repeated invocation produced identical output with no state drift. With no redirector code path remaining, there is no command-specific output or state to compare across repeated calls — only the framework's generic unknown-command error, already covered structurally by IT-1's former scope.
> Becomes testable when: no committed task.

---

### IT-8: N/A — redirector fully removed (Task 409)

> **N/A** — This case was already N/A before Task 409 (the redirector was a pure, argument-independent function with no further distinguishable behavior beyond IT-1 through IT-7). Task 409 has since deleted the redirector itself, so the original reasoning is superseded by the same full-removal rationale as IT-1 through IT-7 above.
> Becomes testable when: no committed task.
