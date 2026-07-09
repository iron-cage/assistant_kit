# Command :: 18. `.account.unclaim` — Redirect Stub

`.account.unclaim` is registered as a **redirect stub** by Feature 037 (shipped).
It exits 1 with a targeted migration hint pointing to `owner::0 name::X`.
All ownership-release behavior is now in `.accounts owner::0 name::X` (Feature 064 — `unclaim::1` is REMOVED) — see `03_accounts.md` IT-44 through IT-45.

| # | Test | Conditions | Exit |
|---|------|-----------|------|
| IT-1 | Redirect stub — targeted `owner::0` migration hint | `clp .account.unclaim name::alice@acme.com`; stderr contains `"owner::0"` migration hint; does NOT produce a generic "unknown command" error | 1 |
| IT-2 | No-args invocation — same targeted hint | `clp .account.unclaim` (no args); stderr contains `"owner::0"` migration hint, identical to IT-1 | 1 |
| IT-3 | `dry::1` ignored — registered but unread parameter | `clp .account.unclaim name::alice@acme.com dry::1`; identical exit/message to IT-1 — routine never reads its arguments | 1 |
| IT-4 | Exact migration message text | Full literal stderr string matches redirector message verbatim, not just the `owner::0` substring | 1 |
| IT-5 | No account-store mutation | No `{name}.json`, marker file, or `.credentials.json` written or modified across the call | 1 |
| IT-6 | Repeated invocation is idempotent | Two consecutive calls produce identical exit code, identical message, no state drift | 1 |
| IT-7 | `name::` value does not affect outcome | Nonexistent/arbitrary account name produces the identical exit code and message as a known account name | 1 |
| IT-8 | N/A — no further distinguishing surface | See N/A entry below | 1 |

**Source:** [feature/037_accounts_usage_param_unification.md AC-11](../../../../docs/feature/037_accounts_usage_param_unification.md),
[cli/command_verb/011_unclaim.md — Migration (Feature 037)](../../../../docs/cli/command_verb/011_unclaim.md#migration-feature-037-feature-064)

---

### IT-1: Redirect stub — targeted `owner::0` migration hint

- **Given:** No accounts required (command is REMOVED — always redirects)
- **When:** `clp .account.unclaim name::alice@acme.com`
- **Then:** Exits 1. stderr contains `"owner::0"` migration hint string. Does NOT produce a generic "unknown command" error — the message is targeted.
- **Exit:** 1
- **Source:** [feature/037_accounts_usage_param_unification.md](../../../../docs/feature/037_accounts_usage_param_unification.md) AC-11, [command_verb/011_unclaim.md](../../../../docs/cli/command_verb/011_unclaim.md)

---

### IT-2: No-args invocation — same targeted hint

- **Given:** No accounts required
- **When:** `clp .account.unclaim` (no arguments at all)
- **Then:** Exits 1. stderr contains the `"owner::0"` migration hint — identical to IT-1's output despite the missing `name::` argument; the redirect stub ignores all arguments unconditionally.
- **Exit:** 1
- **Source:** [command_verb/011_unclaim.md](../../../../docs/cli/command_verb/011_unclaim.md); proven by `ft11b_account_unclaim_no_args` in `tests/cli/accounts_ft_test.rs`

---

### IT-3: `dry::1` ignored — registered but unread parameter

- **Given:** `.account.unclaim` is registered with `nam(), dry(), trc()` (`src/registry.rs` line 218) — `dry::` IS a recognized parameter, unlike `13_account_rotate.md`'s zero-param case
- **When:** `clp .account.unclaim name::alice@acme.com dry::1`
- **Then:** Exits 1 with the identical targeted `owner::0` hint as IT-1 — `dry::1` is accepted by argument validation (it is registered) but has zero observable effect because the routine (`account_unclaim_redirector`) never reads its `_cmd`/`_ctx` parameters.
- **Exit:** 1
- **Source:** `src/registry.rs` lines 49-58 (`_cmd`, `_ctx` unused in `account_unclaim_redirector`) and line 218 (`dry()` in the registered parameter list)

---

### IT-4: Exact migration message text

- **Given:** Any invocation
- **When:** `clp .account.unclaim name::alice@acme.com`
- **Then:** Exits 1. stderr contains the complete literal message `'.account.unclaim' is removed — use 'owner::0 name::X' (or 'owner::0' alone to batch-clear) on '.accounts' or '.usage' instead` verbatim (not merely the `owner::0` substring already covered by IT-1).
- **Exit:** 1
- **Source:** `src/registry.rs` line 56 (`account_unclaim_redirector` — exact `ErrorData::new` message string)

---

### IT-5: No account-store mutation

- **Given:** One account saved (`alice@acme.com`) with a non-empty `owner` field
- **When:** `clp .account.unclaim name::alice@acme.com`
- **Then:** Exits 1. `{name}.json` (including its `owner` field), any `_active*` marker file, and `~/.claude/.credentials.json` are all unmodified — the routine cannot write any file since it never reads its inputs.
- **Exit:** 1
- **Source:** `src/registry.rs` lines 49-58 (`_cmd`, `_ctx` unused — routine has no filesystem access path); mirrors the no-mutation pattern already proven for `.account.rotate` in `13_account_rotate.md` IT-3

---

### IT-6: Repeated invocation is idempotent

- **Given:** One account saved
- **When:** `clp .account.unclaim name::alice@acme.com` invoked twice in sequence
- **Then:** Both calls exit 1 with the identical message; no marker file or `{name}.json` state drift between the two calls.
- **Exit:** 1
- **Source:** `src/registry.rs` lines 49-58 (routine ignores `_cmd`/`_ctx` entirely, so repetition cannot introduce state drift by construction)

---

### IT-7: `name::` value does not affect outcome

- **Given:** Credential store contains only `alice@acme.com`
- **When:** `clp .account.unclaim name::ghost@nonexistent.example`
- **Then:** Exits 1 with the identical `owner::0` migration hint as IT-1 (which used a real, known account name) — the redirect is unconditional regardless of whether the named account exists, because `name::`'s value is never read by the routine.
- **Exit:** 1
- **Source:** `src/registry.rs` lines 49-58 (`_cmd` — which carries the parsed `name::` value — is never read)

---

### IT-8: N/A — no further distinguishing surface

> **N/A** — The redirector is a pure, argument-independent function (`_cmd`/`_ctx` unused, always the same `Err`) already exercised by IT-1 through IT-7 (default invocation, no-args, ignored-parameter, exact message text, no-mutation, idempotency, and value-independence). No further genuinely distinguishable behavior exists to test without inventing an unsubstantiated claim (e.g., the G8 ownership gate documented in `docs/cli/command_verb/011_unclaim.md` for the *replacement* `.accounts owner::0` command does not apply here — this redirect stub never evaluates ownership, since it never reads its arguments — and positional bare-argument routing to `name` was investigated via the shared `nam()` helper used elsewhere, but no direct test evidence exists for this specific command, so neither is asserted here).
> Becomes testable when: no committed task.
