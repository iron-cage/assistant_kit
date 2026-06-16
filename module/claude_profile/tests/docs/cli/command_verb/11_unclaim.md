# Test: verb::unclaim

Behavioral contract tests for the `unclaim` verb. Verifies idempotency on already-unowned accounts,
metadata-only write behavior (no credential touch), and G8 ownership gate enforcement as defined in
[docs/cli/command_verb/011_unclaim.md](../../../../docs/cli/command_verb/011_unclaim.md).

**Idempotency:** Yes — unclaiming an already-unowned account produces identical stored state.
**State Pattern:** Metadata-only mutation — writes `owner: ""` to `{name}.json` only; `{name}.credentials.json` untouched; active marker untouched.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| BV-1 | Re-unclaiming an already-unowned account is idempotent | Idempotency |
| BV-2 | Unclaim writes `owner: ""` without touching credential files | State Transition |
| BV-3 | Unclaim on account owned by different identity exits 1 | Pre-condition (G8) |

### Test Coverage Summary

- Idempotency: 1 test
- State Transition: 1 test
- Pre-condition: 1 test

**Total:** 3 behavioral contract tests

---

### BV-1: Re-unclaiming an already-unowned account is idempotent

- **Given:** `alice@acme.com` exists in credential store. `alice.json` already contains `"owner": ""` (unowned). Current identity = `testuser@testmachine`.
- **When:** `clp .accounts unclaim::1 name::alice@acme.com`
- **Then:** Exit 0. `alice.json` still contains `"owner": ""` (unchanged). `alice.credentials.json` unmodified. No error output.
- **Exit:** 0
- **Source:** [011_unclaim.md — Idempotency](../../../../docs/cli/command_verb/011_unclaim.md#idempotency)

---

### BV-2: Unclaim writes `owner: ""` without touching credential files

- **Given:** `alice@acme.com` exists. `alice.json` has `"owner": "testuser@testmachine"`. `current_identity()` = `testuser@testmachine`. Record mtime of `alice.credentials.json` and `~/.claude/.credentials.json`.
- **When:** `clp .accounts unclaim::1 name::alice@acme.com`
- **Then:** Exit 0. `alice.json` contains `"owner": ""`. mtime of `alice.credentials.json` unchanged. mtime of `~/.claude/.credentials.json` unchanged (no credential rotation). Active marker `_active_{hostname}_{user}` unchanged.
- **Exit:** 0
- **Source:** [011_unclaim.md — State Transition Pattern](../../../../docs/cli/command_verb/011_unclaim.md#state-transition-pattern)

---

### BV-3: Unclaim on account owned by different identity exits 1

- **Given:** `alice@acme.com` exists. `alice.json` has `"owner": "other@remote"`. Current identity ≠ `other@remote`.
- **When:** `clp .accounts unclaim::1 name::alice@acme.com`
- **Then:** Exit 1. Stderr contains `ownership violation: this account is owned by other@remote`. `alice.json` unchanged (`owner` remains `"other@remote"`). No file written.
- **Exit:** 1
- **Source:** [011_unclaim.md — Ownership Gate (G8)](../../../../docs/cli/command_verb/011_unclaim.md#ownership-gate-g8)
