# Command :: 13. `.account.rotate` — Integration Tests

> **DEPRECATED** — `.account.rotate` is now a redirector (Feature 016). Always exits 1 with migration message. Rotation moved to `.usage rotate::1` (Feature 038). Test cases below verify redirector behavior only.

| # | Test | Conditions | Exit |
|---|------|-----------|------|
| IT-1 | Always exits 1 | Any invocation of `.account.rotate` | 1 |
| IT-2 | Error message references `.usage rotate` | No args; stderr/stdout contains `.usage rotate` | 1 |
| IT-3 | No mutation on exit 1 | Two accounts present; `_active` unchanged after deprecated call | 1 |
| IT-4 | `dry::1` ignored — no registered params, still exits 1 | `dry::1` passed to a zero-param command; identical outcome to no-arg call | 1 |
| IT-5 | Exits 1 with zero/one accounts (not exit 2) | Single active account, no inactive accounts; old "no inactive account" precondition never evaluated | 1 |
| IT-6 | Exact migration message text | Full literal stderr/stdout string matches redirector message verbatim | 1 |
| IT-7 | Repeated invocation is idempotent | Two consecutive calls produce identical exit code, identical message, no state drift | 1 |
| IT-8 | N/A — no further distinguishing surface | See N/A entry below | 1 |

---

### IT-1: Always exits 1

- **Given:** Any credential store state (zero or more accounts)
- **When:** `clp .account.rotate`
- **Then:** Exits 1. Command is DEPRECATED — always redirects with exit code 1.
- **Exit:** 1
- **Source:** [command_verb/006_rotate.md](../../../../docs/cli/command_verb/006_rotate.md)

---

### IT-2: Error message references `.usage rotate`

- **Given:** Any credential store state
- **When:** `clp .account.rotate` (no arguments)
- **Then:** Exits 1. stderr or stdout contains the string `.usage rotate`.
- **Exit:** 1
- **Source:** [command_verb/006_rotate.md](../../../../docs/cli/command_verb/006_rotate.md), [feature/038_usage_strategy_rotate.md](../../../../docs/feature/038_usage_strategy_rotate.md)

---

### IT-3: No mutation on exit 1

- **Given:** Two accounts saved; one is active (has `_active` per-machine marker set)
- **When:** `clp .account.rotate`
- **Then:** Exits 1. The `_active` marker is unchanged — no account file was mutated.
- **Exit:** 1
- **Source:** [command_verb/006_rotate.md](../../../../docs/cli/command_verb/006_rotate.md)

---

### IT-4: `dry::1` ignored — no registered params, still exits 1

- **Given:** `.account.rotate` is registered with zero declared parameters (`vec![]` in `registry.rs`); credential store has at least one active and one inactive account
- **When:** `clp .account.rotate dry::1`
- **Then:** Exits 1 — identical to the no-arg invocation. `dry::1` is not a registered parameter for this command and has no effect on the outcome; the redirector short-circuits before any parameter is consulted.
- **Exit:** 1
- **Source:** [command_verb/006_rotate.md](../../../../docs/cli/command_verb/006_rotate.md); `src/registry.rs` line 216 (`vec![]`) confirms zero declared params; proven by `rotation_ua3_dry_run_previews_without_switching` in `tests/cli/user_story_test.rs`

---

### IT-5: Exits 1 with zero/one accounts (not exit 2)

- **Given:** Exactly one account saved and active; zero inactive accounts exist in the credential store
- **When:** `clp .account.rotate`
- **Then:** Exits 1 — not exit 2 (the pre-deprecation rotation algorithm's "at least one inactive account" precondition is never evaluated because the redirector returns before any account-state logic runs).
- **Exit:** 1
- **Source:** [command_verb/006_rotate.md](../../../../docs/cli/command_verb/006_rotate.md) § Behavioral Contract (documents the old precondition, now unreachable); proven by `rotation_ua5_no_inactive_accounts_exits_2` in `tests/cli/user_story_test.rs` and `rotate_bv3_no_inactive_accounts_exits_2` in `tests/cli/command_verb_test.rs`

---

### IT-6: Exact migration message text

- **Given:** Any credential store state
- **When:** `clp .account.rotate`
- **Then:** Exits 1. stderr/stdout contains the complete literal message `'.account.rotate' is deprecated — use '.usage rotate::1' instead` verbatim (not merely the `.usage rotate` substring already covered by IT-2).
- **Exit:** 1
- **Source:** `src/registry.rs` line 34 (`account_rotate_redirector` — exact `ErrorData::new` message string)

---

### IT-7: Repeated invocation is idempotent

- **Given:** Two accounts saved; one active
- **When:** `clp .account.rotate` invoked twice in sequence
- **Then:** Both calls exit 1 with the identical message; the `_active` marker file is byte-identical before the first call, between the two calls, and after the second call — no cumulative side effects across repetition.
- **Exit:** 1
- **Source:** [command_verb/006_rotate.md](../../../../docs/cli/command_verb/006_rotate.md); routine ignores `_cmd`/`_ctx` entirely (`src/registry.rs` lines 30-36), so repetition cannot introduce state drift by construction

---

### IT-8: N/A — no further distinguishing surface

> **N/A** — The redirector is a pure, argument-independent function (`_cmd`/`_ctx` unused, always the same `Err`) already exercised by IT-1 through IT-7 (default invocation, message substring, no-mutation, ignored-parameter, zero-account edge, exact message text, and idempotency); no further genuinely distinguishable behavior exists to test without inventing an unsubstantiated claim (e.g., malformed positional-argument parsing was investigated but no direct evidence was found for a zero-declared-parameter command, so it is not asserted here).
> Becomes testable when: no committed task.
