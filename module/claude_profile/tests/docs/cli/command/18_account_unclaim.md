# Command :: 18. `.account.unclaim` — Redirect Stub

> **REMOVED** — `.account.unclaim` was a redirector (Feature 037) that always exited 1 with a migration message. Task 409 deleted the redirector function backing `.account.unclaim` and its registration entirely; ownership-release behavior lives at `.accounts owner::0 name::X` (Feature 064 — see `03_accounts.md` IT-44 through IT-45). `clp .account.unclaim` now fails at the CLI-parse stage with a generic unknown-command error, identical to any other unrecognized command name — none of the redirector-specific behavior IT-1 through IT-7 below documented is observable any longer.

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

**Source:** [feature/037_accounts_usage_param_unification.md AC-11](../../../../docs/feature/037_accounts_usage_param_unification.md),
[cli/command_verb/011_unclaim.md — Migration (Feature 037)](../../../../docs/cli/command_verb/011_unclaim.md#migration-feature-037-feature-064)

---

### IT-1: N/A — redirector fully removed (Task 409)

> **N/A** — This case verified the redirector exited 1 with a targeted `owner::0` migration hint. Task 409 deleted the redirector function backing `.account.unclaim` and its registration entirely; the command now fails at the CLI-parse stage with a generic unknown-command error before any command-specific code runs, so there is no targeted-hint behavior left to assert.
> Becomes testable when: no committed task.

---

### IT-2: N/A — redirector fully removed (Task 409)

> **N/A** — This case verified the no-args invocation produced the identical targeted hint as IT-1. The redirect stub that ignored arguments unconditionally no longer exists; the generic unknown-command error is framework-produced and does not vary by argument presence.
> Becomes testable when: no committed task.

---

### IT-3: N/A — redirector fully removed (Task 409)

> **N/A** — This case verified `dry::1` was accepted but ignored because the routine never read its `_cmd`/`_ctx` parameters. Registration itself is gone — `dry::1` is no longer a recognized parameter for a nonexistent command, and the CLI-parse-stage rejection does not distinguish between argument sets.
> Becomes testable when: no committed task.

---

### IT-4: N/A — redirector fully removed (Task 409)

> **N/A** — This case verified the exact literal migration-hint message text. The redirector function that produced that exact string has been deleted from `src/registry.rs`; no code path in the crate emits that message any longer.
> Becomes testable when: no committed task.

---

### IT-5: N/A — redirector fully removed (Task 409)

> **N/A** — This case verified no account-store mutation occurred on the redirector's exit-1 path. With the command fully deregistered, the CLI-parse stage rejects the invocation before any command routine — redirector or otherwise — could run, so there is no command-specific code path left whose mutation behavior could be asserted.
> Becomes testable when: no committed task.

---

### IT-6: N/A — redirector fully removed (Task 409)

> **N/A** — This case verified repeated invocation produced identical output with no state drift. With no redirector code path remaining, there is no command-specific output or state to compare across repeated calls — only the framework's generic unknown-command error, already covered structurally by IT-1's former scope.
> Becomes testable when: no committed task.

---

### IT-7: N/A — redirector fully removed (Task 409)

> **N/A** — This case verified the redirect was unconditional regardless of whether the named account existed. There is no command-specific argument handling left to distinguish — every invocation now fails identically at the CLI-parse stage regardless of `name::`'s value.
> Becomes testable when: no committed task.

---

### IT-8: N/A — redirector fully removed (Task 409)

> **N/A** — This case was already N/A before Task 409 (the redirector was a pure, argument-independent function with no further distinguishable behavior beyond IT-1 through IT-7). Task 409 has since deleted the redirector itself, so the original reasoning is superseded by the same full-removal rationale as IT-1 through IT-7 above.
> Becomes testable when: no committed task.
