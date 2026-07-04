# Test: Feature 019 — Browser Re-Authentication for Named Account

Feature behavioral requirement test cases for `docs/feature/019_account_relogin.md`. Each FT case maps to one or more acceptance criteria.

### AC Coverage Index

| FT | Criterion | AC |
|----|-----------|-----|
| FT-01 | `dry::1` with explicit name exits 0 with dry-run message, no mutation | AC-01 |
| FT-02 | `dry::1` without `name::` uses active account | AC-02 |
| FT-03 | No `name::` and no active account → exit 2 | AC-03 |
| FT-04 | Non-existent `name::` → exit 2 not-found | AC-04 |
| FT-05 | Positional bare arg accepted | AC-05 |
| FT-06 | Prefix form resolves to matching account | AC-06 |
| FT-07 | After successful login, credential store is updated | AC-07 |
| FT-08 | Original active account restored after re-authentication | AC-08 |
| FT-09 | `claude` exits without credential change → diagnostic + exit 3 | AC-09 |
| FT-10 | Non-owned account: `.account.relogin` exits 1 with ownership violation message | AC-10 |
| FT-11 | Ownership check fires before `dry::1` — exits 1 even with `dry::1` set | AC-11 |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | `dry::1` explicit name → dry-run message, exit 0 | AC-01 | Dry Run |
| FT-02 | `dry::1` no `name::` → uses active account | AC-02 | Dry Run |
| FT-03 | No `name::`, no active account → exit 2 | AC-03 | Error Handling |
| FT-04 | Non-existent account → exit 2 not-found | AC-04 | Error Handling |
| FT-05 | Positional bare arg accepted | AC-05 | Shortcut Syntax |
| FT-06 | Prefix `car` resolves to `carol@example.com` | AC-06 | Shortcut Syntax |
| FT-07 | Credential store updated after successful TTY login | AC-07 | Side Effects |
| FT-08 | Original active account restored after login | AC-08 | Restore |
| FT-09 | No credential change → diagnostic stderr + exit 3 | AC-09 | Abandoned Login |
| FT-10 | Non-owned account exits 1 with ownership violation message | AC-10 | Ownership Guard |
| FT-11 | Ownership check fires before `dry::1` — exits 1 regardless of dry-run | AC-11 | Ownership Guard |

**Total:** 11 FT cases

---

### FT-01: `dry::1` explicit name → dry-run message, exit 0

- **Given:** `carol@example.com` exists in the credential store.
- **When:** `clp .account.relogin name::carol@example.com dry::1`
- **Then:** Outputs `[dry-run] would re-authenticate 'carol@example.com' via browser login`. No files mutated. Exit 0.
- **Exit:** 0
- **Source fn:** `ar05_relogin_dry_explicit_name`
- **Source:** [019_account_relogin.md AC-01](../../../docs/feature/019_account_relogin.md)

---

### FT-02: `dry::1` without `name::` → uses active account

- **Given:** An active account `alice@acme.com` per per-machine active marker.
- **When:** `clp .account.relogin dry::1`
- **Then:** Outputs `[dry-run] would re-authenticate 'alice@acme.com' via browser login`. Exit 0.
- **Exit:** 0
- **Source fn:** `relogin_mre_no_name_uses_active`
- **Source:** [019_account_relogin.md AC-02](../../../docs/feature/019_account_relogin.md)

---

### FT-03: No `name::` and no active account → exit 2

- **Given:** No per-machine active marker exists.
- **When:** `clp .account.relogin` (no `name::`)
- **Then:** Exits 2 with an actionable message indicating no active account.
- **Exit:** 2
- **Source fn:** `relogin_mre_no_name_no_active_exits2`
- **Source:** [019_account_relogin.md AC-03](../../../docs/feature/019_account_relogin.md)

---

### FT-04: Non-existent account → exit 2 not-found

- **Given:** `ghost@example.com` does not exist in the credential store.
- **When:** `clp .account.relogin name::ghost@example.com dry::1`
- **Then:** Exits 2 with a not-found error.
- **Exit:** 2
- **Source fn:** `ar04_relogin_not_found_exits_2`
- **Source:** [019_account_relogin.md AC-04](../../../docs/feature/019_account_relogin.md)

---

### FT-05: Positional bare arg accepted

- **Given:** `carol@example.com` exists in the credential store.
- **When:** `clp .account.relogin carol@example.com dry::1`
- **Then:** Positional arg is accepted; dry-run output shows `carol@example.com`. Exit 0.
- **Exit:** 0
- **Source fn:** `ar07_relogin_positional_bare_arg`
- **Source:** [019_account_relogin.md AC-05](../../../docs/feature/019_account_relogin.md)

---

### FT-06: Prefix `car` resolves to `carol@example.com`

- **Given:** `carol@example.com` is the only saved account whose local part starts with `car`.
- **When:** `clp .account.relogin car dry::1`
- **Then:** Resolves to `carol@example.com`; dry-run output shows the resolved name. Exit 0.
- **Exit:** 0
- **Source fn:** `ar08_relogin_prefix_resolves`
- **Source:** [019_account_relogin.md AC-06](../../../docs/feature/019_account_relogin.md)

---

### FT-07: Credential store updated after successful TTY login

- **Given:** `carol@example.com` is in the store with stale credentials. A successful `claude` TTY spawn updates `~/.claude/.credentials.json`.
- **When:** `clp .account.relogin name::carol@example.com` (live, interactive)
- **Then:** `{credential_store}/carol@example.com.credentials.json` is updated with the new credentials (same as if `.account.save` had run). Exit 0.
- **Exit:** 0
- **Source fn:** manual — IT-5 in `tests/manual/readme.md`
- **Source:** [019_account_relogin.md AC-07](../../../docs/feature/019_account_relogin.md)

---

### FT-08: Original active account restored after login

- **Given:** `alice@acme.com` is the active account. Re-login is requested for `carol@example.com`.
- **When:** `clp .account.relogin name::carol@example.com` (live, interactive)
- **Then:** After re-authentication completes, the active account is restored to `alice@acme.com`. The user's session context is unchanged. Exit 0.
- **Exit:** 0
- **Source fn:** manual — IT-6 in `tests/manual/readme.md`
- **Source:** [019_account_relogin.md AC-08](../../../docs/feature/019_account_relogin.md)

---

### FT-09: No credential change → diagnostic stderr + exit 3

- **Given:** `carol@example.com` is in the store. `claude` is spawned but exits without updating `~/.claude/.credentials.json` (user abandoned login).
- **When:** `clp .account.relogin name::carol@example.com` (live, interactive)
- **Then:** A diagnostic message is printed to stderr indicating credentials were unchanged. Process exits 3 (not 0 or 2).
- **Exit:** 3
- **Source fn:** manual — IT-7 in `tests/manual/readme.md`
- **Source:** [019_account_relogin.md AC-09](../../../docs/feature/019_account_relogin.md)

---

### FT-10: Non-owned account exits 1 with ownership violation message

- **Given:** Account `alice@corp.com` has `{credential_store}/alice@corp.com.json` with `"owner": "other@remote"`. The current machine's `current_identity()` is `"user1@thishost"` — not equal to `"other@remote"`.
- **When:** `clp .account.relogin name::alice@corp.com`
- **Then:** Exits 1. Stderr contains `"ownership violation: this account is owned by other@remote"`. The 6-step relogin procedure is NOT started — no `switch_account()` call, no `claude` spawn, no credential comparison.
- **Exit:** 1
- **Source fn:** `ft10_relogin_exits_1_when_not_owned` (in `tests/cli/account_mutations_test.rs`)
- **Note:** G7 ownership gate from Feature 036 AC-10. Shared with Feature 036 FT-10.
- **Source:** [019_account_relogin.md AC-10](../../../docs/feature/019_account_relogin.md)

---

### FT-11: Ownership check fires before `dry::1` — exits 1 even with `dry::1` set

- **Given:** Account `alice@corp.com` is owned by `"other@remote"`. Current identity ≠ `"other@remote"`.
- **When:** `clp .account.relogin name::alice@corp.com dry::1`
- **Then:** Exits 1. The ownership violation message is printed to stderr. The `[dry-run] would re-authenticate 'alice@corp.com' via browser login` message is NOT printed. No files are modified and no procedure steps are executed.
- **Exit:** 1
- **Source fn:** `ft13_dry_run_does_not_skip_ownership` (in `tests/cli/account_mutations_test.rs`)
- **Note:** G7 + dry-run ordering gate from Feature 036 AC-13. Ownership guard runs before `dry::1` evaluation.
- **Source:** [019_account_relogin.md AC-11](../../../docs/feature/019_account_relogin.md)
