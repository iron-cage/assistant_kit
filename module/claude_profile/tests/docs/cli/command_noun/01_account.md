# Test: noun::account

Noun contract tests for the `account` domain noun. Verifies lifecycle state machine
correctness, JSON output schema fidelity, and error code contract as defined in
[docs/cli/command_noun/001_account.md](../../../../docs/cli/command_noun/001_account.md).

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| NC-1 | Full lifecycle round-trip: absent → saved → active → absent | Lifecycle |
| NC-2 | `.accounts format::json` output matches documented schema | Output Schema |
| NC-3 | Documented error codes produced for documented trigger conditions | Error Code Contract |

### Test Coverage Summary

- Lifecycle: 1 test
- Output Schema: 1 test
- Error Code Contract: 1 test

**Total:** 3 noun contract tests

---

### NC-1: Full lifecycle round-trip: absent → saved → active → absent

- **Given:** Clean credential store with no `alice@acme.com` profile. Active credentials in `~/.claude/.credentials.json`.
- **When:** Sequence:
  1. `clp .account.save name::alice@acme.com` (absent → saved)
  2. `clp .account.use name::alice@acme.com` (saved → active)
  3. `clp .account.use name::bob@acme.com` (alice: active → saved; bob: saved → active)
  4. `clp .account.delete name::alice@acme.com` (saved → absent)
- **Then:** Step 1: exit 0; `alice@acme.com.credentials.json` created. Step 2: exit 0; alice is active. Step 3: exit 0; alice is saved, bob is active. Step 4: exit 0; alice's files absent. Each transition matches the documented lifecycle diagram.
- **Exit:** 0
- **Source:** [001_account.md — Lifecycle](../../../../docs/cli/command_noun/001_account.md#lifecycle)

---

### NC-2: `.accounts format::json` output matches documented schema

- **Given:** `alice@acme.com` exists in credential store and is currently active.
- **When:** `clp .accounts format::json`
- **Then:** Exit 0. Output is valid JSON array. Each element contains at minimum: `name` (string), `active` (bool), `sub` (string), `tier` (string), `expires_in_secs` (number). No undocumented fields required but schema fields present must match documented types.
- **Exit:** 0
- **Source:** [001_account.md — Output Schema](../../../../docs/cli/command_noun/001_account.md#output-schema)

---

### NC-3: Documented error codes produced for documented trigger conditions

- **Given:** Credential store configured with at least one saved account.
- **When (a):** `clp .account.use name::notanemail` — invalid name format
- **When (b):** `clp .account.use name::nobody@acme.com` — account not found
- **When (c):** `clp .account.renewal name::alice@acme.com` — missing `at::`/`from_now::`/`clear::`
- **Then (a):** Exit 1. Error references invalid name format.
- **Then (b):** Exit 2. Error references account not found.
- **Then (c):** Exit 1. Error references missing required operation parameter.
- **Exit:** 1 (a/c), 2 (b)
- **Source:** [001_account.md — Error Codes](../../../../docs/cli/command_noun/001_account.md#error-codes)
