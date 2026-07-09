# Command :: 16. `.account.assign` — Redirect Stub

`.account.assign` is registered as a **redirect stub** by Feature 037 (shipped).
It exits 1 with a targeted migration hint pointing to `assignee::USER@MACHINE name::X`.
All marker-write behavior is now in `.accounts assignee::USER@MACHINE name::X` (or `assignee::0 name::X` for current machine) (Feature 065 — `assign::1` and `active::` are both REMOVED) — see `03_accounts.md` IT-43 through IT-46.

| # | Test | Conditions | Exit |
|---|------|-----------|------|
| IT-1 | Redirect stub — targeted `assignee::` migration hint | `clp .account.assign name::alice@acme.com`; stderr contains `"assignee::"` migration hint; does NOT produce a generic "unknown command" error | 1 |
| IT-2 | No-args invocation — same targeted hint | `clp .account.assign` (no args); stderr contains `"assignee::"` migration hint, identical to IT-1 | 1 |
| IT-3 | `dry::1` ignored — registered but unread parameter | `clp .account.assign name::alice@acme.com dry::1`; identical exit/message to IT-1 — routine never reads its arguments | 1 |
| IT-4 | Exact migration message text | Full literal stderr string matches redirector message verbatim, not just the `assignee::` substring | 1 |
| IT-5 | No account-store mutation | No `{name}.json`, marker file, or `.credentials.json` written or modified across the call | 1 |
| IT-6 | Repeated invocation is idempotent | Two consecutive calls produce identical exit code, identical message, no state drift | 1 |
| IT-7 | `name::` value does not affect outcome | Nonexistent/arbitrary account name produces the identical exit code and message as a known account name | 1 |
| IT-8 | N/A — no further distinguishing surface | See N/A entry below | 1 |

**Source:** [feature/037_accounts_usage_param_unification.md AC-12](../../../../docs/feature/037_accounts_usage_param_unification.md),
[cli/command_verb/009_assign.md — Migration (Feature 037)](../../../../docs/cli/command_verb/009_assign.md#migration-feature-037-feature-064-feature-065)

---

### IT-1: Redirect stub — targeted `assignee::` migration hint

- **Given:** No accounts required (command is REMOVED — always redirects)
- **When:** `clp .account.assign name::alice@acme.com`
- **Then:** Exits 1. stderr contains `"assignee::"` migration hint string. Does NOT produce a generic "unknown command" error — the message is targeted.
- **Exit:** 1
- **Source:** [feature/037_accounts_usage_param_unification.md](../../../../docs/feature/037_accounts_usage_param_unification.md) AC-12, [command_verb/009_assign.md](../../../../docs/cli/command_verb/009_assign.md)

---

### IT-2: No-args invocation — same targeted hint

- **Given:** No accounts required
- **When:** `clp .account.assign` (no arguments at all)
- **Then:** Exits 1. stderr contains the `"assignee::"` migration hint — identical to IT-1's output despite the missing `name::` argument; the redirect stub ignores all arguments unconditionally.
- **Exit:** 1
- **Source:** [command_verb/009_assign.md](../../../../docs/cli/command_verb/009_assign.md); proven by `ft12b_account_assign_no_args` in `tests/cli/accounts_ft_test.rs`

---

### IT-3: `dry::1` ignored — registered but unread parameter

- **Given:** `.account.assign` is registered with `nam(), dry(), trc()` (`src/registry.rs` line 217) — `dry::` IS a recognized parameter, unlike `13_account_rotate.md`'s zero-param case
- **When:** `clp .account.assign name::alice@acme.com dry::1`
- **Then:** Exits 1 with the identical targeted `assignee::` hint as IT-1 — `dry::1` is accepted by argument validation (it is registered) but has zero observable effect because the routine (`account_assign_redirector`) never reads its `_cmd`/`_ctx` parameters.
- **Exit:** 1
- **Source:** `src/registry.rs` lines 41-47 (`_cmd`, `_ctx` unused in `account_assign_redirector`) and line 217 (`dry()` in the registered parameter list)

---

### IT-4: Exact migration message text

- **Given:** Any invocation
- **When:** `clp .account.assign name::alice@acme.com`
- **Then:** Exits 1. stderr contains the complete literal message `'.account.assign' is removed — use 'assignee::USER@MACHINE name::X' (or 'assignee::0 name::X' for current machine) on '.accounts' or '.usage' instead` verbatim (not merely the `assignee::` substring already covered by IT-1).
- **Exit:** 1
- **Source:** `src/registry.rs` line 45 (`account_assign_redirector` — exact `ErrorData::new` message string)

---

### IT-5: No account-store mutation

- **Given:** One account saved (`alice@acme.com`), no marker files present
- **When:** `clp .account.assign name::alice@acme.com`
- **Then:** Exits 1. `{name}.json`, any `_active*` marker file, and `~/.claude/.credentials.json` are all unmodified — the routine cannot write any file since it never reads its inputs.
- **Exit:** 1
- **Source:** `src/registry.rs` lines 41-47 (`_cmd`, `_ctx` unused — routine has no filesystem access path); mirrors the no-mutation pattern already proven for `.account.rotate` in `13_account_rotate.md` IT-3

---

### IT-6: Repeated invocation is idempotent

- **Given:** One account saved
- **When:** `clp .account.assign name::alice@acme.com` invoked twice in sequence
- **Then:** Both calls exit 1 with the identical message; no marker file or `{name}.json` state drift between the two calls.
- **Exit:** 1
- **Source:** `src/registry.rs` lines 41-47 (routine ignores `_cmd`/`_ctx` entirely, so repetition cannot introduce state drift by construction)

---

### IT-7: `name::` value does not affect outcome

- **Given:** Credential store contains only `alice@acme.com`
- **When:** `clp .account.assign name::ghost@nonexistent.example`
- **Then:** Exits 1 with the identical `assignee::` migration hint as IT-1 (which used a real, known account name) — the redirect is unconditional regardless of whether the named account exists, because `name::`'s value is never read by the routine.
- **Exit:** 1
- **Source:** `src/registry.rs` lines 41-47 (`_cmd` — which carries the parsed `name::` value — is never read)

---

### IT-8: N/A — no further distinguishing surface

> **N/A** — The redirector is a pure, argument-independent function (`_cmd`/`_ctx` unused, always the same `Err`) already exercised by IT-1 through IT-7 (default invocation, no-args, ignored-parameter, exact message text, no-mutation, idempotency, and value-independence). No further genuinely distinguishable behavior exists to test without inventing an unsubstantiated claim (e.g., positional bare-argument routing to `name` was investigated via the shared `nam()` helper used elsewhere, but no direct test evidence exists for this specific command, so it is not asserted here).
> Becomes testable when: no committed task.
