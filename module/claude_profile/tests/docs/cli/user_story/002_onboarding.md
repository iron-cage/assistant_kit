# Test: user story 2 — Account Onboarding and Lifecycle Management

User acceptance tests for the "Account Onboarding and Lifecycle Management" story. Each UA-N
case maps to one Acceptance Criterion from
[docs/cli/user_story/002_onboarding.md](../../../../docs/cli/user_story/002_onboarding.md).

**Persona:** Developer setting up Claude Code or managing saved account profiles.

### Test Case Index

| ID | Test Name | Acceptance Criterion |
|----|-----------|---------------------|
| UA-1 | `.account.save` captures credentials to the credential store | AC-1 |
| UA-2 | Name auto-inferred from `oauthAccount.emailAddress`; exits 1 if neither source present | AC-2 |
| UA-3 | `host::` and `role::` captured in `{name}.json`; `dry::1` previews without writing | AC-3 |
| UA-4 | `.account.delete` removes all account files from store | AC-4 |
| UA-5 | `.account.relogin` spawns `claude` with TTY; propagates fresh credentials | AC-5 |
| UA-6 | `.account.renewal` sets `_renewal_at` in `{name}.json` | AC-6 |

### Test Coverage Summary

- Credential capture: 1 test
- Name inference: 1 test
- Metadata + dry run: 1 test
- Deletion: 1 test
- Re-authentication: 1 test
- Renewal: 1 test

**Total:** 6 user acceptance tests

---

### UA-1: `.account.save` captures credentials to the credential store

- **Given:** `~/.claude/.credentials.json` exists with valid credentials. Credential store exists (may be empty).
- **When:** `clp .account.save name::alice@acme.com`
- **Then:** Exit 0. `{credential_store}/alice@acme.com.credentials.json` created as a copy of `~/.claude/.credentials.json`. `{credential_store}/alice@acme.com.json` created with supplementary metadata. stdout: `saved current credentials as 'alice@acme.com'`.
- **Exit:** 0
- **Source:** [002_onboarding.md — AC-1](../../../../docs/cli/user_story/002_onboarding.md)

---

### UA-2: Name auto-inferred from `oauthAccount.emailAddress`; exits 1 if neither source present

- **Given (a):** `~/.claude/.credentials.json` exists. `~/.claude.json` contains `oauthAccount.emailAddress = "alice@acme.com"`. No `name::` passed.
- **Given (b):** `~/.claude/.credentials.json` exists. `~/.claude.json` absent. No active marker. No `name::` passed.
- **When (a):** `clp .account.save`
- **When (b):** `clp .account.save`
- **Then (a):** Exit 0. Credentials saved as `alice@acme.com` (inferred from `oauthAccount.emailAddress`).
- **Then (b):** Exit 1. Error message referencing inability to infer account name.
- **Exit:** 0 (a), 1 (b)
- **Source:** [002_onboarding.md — AC-2](../../../../docs/cli/user_story/002_onboarding.md)

---

### UA-3: `host::` and `role::` captured in `{name}.json`; `dry::1` previews without writing

- **Given:** `~/.claude/.credentials.json` exists with valid credentials.
- **When (a):** `clp .account.save name::alice@acme.com host::laptop role::work`
- **When (b):** `clp .account.save name::alice@acme.com dry::1`
- **Then (a):** Exit 0. `{credential_store}/alice@acme.com.json` contains `host = "laptop"` and `role = "work"` fields.
- **Then (b):** Exit 0. stdout: `[dry-run] would save current credentials as 'alice@acme.com'`. No files created.
- **Exit:** 0
- **Source:** [002_onboarding.md — AC-3](../../../../docs/cli/user_story/002_onboarding.md)

---

### UA-4: `.account.delete` removes all account files from store

- **Given:** `alice@acme.com` profile exists in credential store (`.credentials.json` + `.json` present). Account is in saved state (not active).
- **When:** `clp .account.delete name::alice@acme.com`
- **Then:** Exit 0. `alice@acme.com.credentials.json` absent. `alice@acme.com.json` absent. No other accounts affected.
- **Exit:** 0
- **Source:** [002_onboarding.md — AC-4](../../../../docs/cli/user_story/002_onboarding.md)

---

### UA-5: `.account.relogin` spawns `claude` with TTY; propagates fresh credentials

- **Given:** `alice@acme.com` exists in credential store with expired credentials. `claude` binary on `$PATH`. TTY available. User completes OAuth flow.
- **When:** `clp .account.relogin name::alice@acme.com`
- **Then:** Exit 0. `alice@acme.com.credentials.json` updated with fresh tokens obtained from the browser OAuth flow. Account lifecycle state unchanged (remains saved if was saved).
- **Exit:** 0
- **Source:** [002_onboarding.md — AC-5](../../../../docs/cli/user_story/002_onboarding.md)

---

### UA-6: `.account.renewal` sets `_renewal_at` in `{name}.json`

- **Given:** `alice@acme.com` exists in credential store. `alice@acme.com.json` has no `_renewal_at` field.
- **When:** `clp .account.renewal name::alice@acme.com at::2026-08-01T00:00:00Z`
- **Then:** Exit 0. `alice@acme.com.json` `_renewal_at` = `"2026-08-01T00:00:00Z"`. All other fields in `{name}.json` preserved via read-merge.
- **Exit:** 0
- **Source:** [002_onboarding.md — AC-6](../../../../docs/cli/user_story/002_onboarding.md)
