# Test: `name::`

Edge case coverage for the `name::` parameter. See [params.md](../../../../docs/cli/params.md#parameter--1-name) and [types.md](../../../../docs/cli/types.md#type--1-accountname) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `name::work` — valid name accepted | Valid Name |
| EC-2 | `name::` (empty value) rejected with exit 1 | Empty Value |
| EC-3 | Omitted `name::` on `.account.use` exits 1 | Required Parameter |
| EC-17 | Omitted `name::` on `.account.save` with `emailAddress` in `~/.claude.json` — infers name | Name Inference |
| EC-4 | `name::` with `/` (no `@`) rejected with exit 1 | Forbidden Characters |
| EC-5 | `name::` with `\` (no `@`) rejected with exit 1 | Forbidden Characters |
| EC-6 | `name::` with `*` (no `@`) rejected with exit 1 | Forbidden Characters |
| EC-7 | `name::` with null byte rejected with exit 1 | Forbidden Characters |
| EC-18 | `name::` with `/` in email local part (`a/b@c.com`) rejected with exit 1 | Forbidden Characters (email) |
| EC-8 | `name::client-a` — hyphens accepted | Valid Characters |
| EC-9 | `name::my_account` — underscores accepted | Valid Characters |
| EC-10 | Very long name (>255 chars) handled without crash | Boundary Value |
| EC-11 | `.accounts name::work` — optional, scopes to named account | Optional on Accounts |
| EC-12 | `.accounts` without `name::` — omitting lists all accounts | Optional on Accounts |
| EC-13 | `.accounts name::ghost` — valid chars but unknown → exit 2 | NotFound on Accounts |
| EC-14 | `.account.limits name::work` — optional, queries named account (FR-18) | Optional on Limits |
| EC-15 | `.account.limits` without `name::` — omitting is valid (active-account path) | Optional on Limits |
| EC-16 | `.account.limits name::ghost` — valid chars but unknown → exit 2 (FR-18) | NotFound on Limits |

### Test Coverage Summary

- Valid Name: 1 test
- Empty Value: 1 test
- Required Parameter: 1 test
- Name Inference: 1 test
- Forbidden Characters: 5 tests
- Valid Characters: 2 tests
- Boundary Value: 1 test
- Optional on Accounts: 3 tests
- Optional on Limits (FR-18): 3 tests

**Total:** 18 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

---

### EC-1: Valid Name

- **Given:** Active credentials exist at `~/.claude/.credentials.json`.
- **When:** `clp .account.save name::work@acme.com`
- **Then:** `saved current credentials as 'work@acme.com'` with exit 0.; credential file created with correct name
- **Exit:** 0
- **Source:** [params.md -- name::](../../../../docs/cli/params.md#parameter--1-name)

---

### EC-2: Empty Value

- **Given:** Active credentials exist at `~/.claude/.credentials.json`.
- **When:** `clp .account.save name::`
- **Then:** Error message containing `name:: value cannot be empty` with exit 1.; empty name rejected with descriptive error
- **Exit:** 1
- **Source:** [types.md -- AccountName](../../../../docs/cli/types.md#type--1-accountname)

---

### EC-3: Required Parameter on `.account.use`

- **Given:** Active credentials exist at `~/.claude/.credentials.json`.
- **When:** `clp .account.use`
- **Then:** Error message indicating `name::` is required with exit 1.; missing required parameter clearly reported; `.account.use` has no inference fallback
- **Exit:** 1
- **Source:** [params.md -- name::](../../../../docs/cli/params.md#parameter--1-name)

---

### EC-4: Forbidden Characters — Forward Slash (no `@`)

- **Given:** Active credentials exist at `~/.claude/.credentials.json`.
- **When:** `clp .account.save name::foo/bar`
- **Then:** Error message containing `must contain '@'` with exit 1. (No `@` present — `@`-absence check fires before path-safety check.); name without `@` rejected as non-email
- **Exit:** 1
- **Source:** [types.md -- AccountName](../../../../docs/cli/types.md#type--1-accountname)

---

### EC-5: Forbidden Characters — Backslash (no `@`)

- **Given:** Active credentials exist at `~/.claude/.credentials.json`.
- **When:** `clp .account.save 'name::foo\bar'`
- **Then:** Error message containing `must contain '@'` with exit 1. (No `@` present — `@`-absence check fires before path-safety check.); name without `@` rejected as non-email
- **Exit:** 1
- **Source:** [types.md -- AccountName](../../../../docs/cli/types.md#type--1-accountname)

---

### EC-6: Forbidden Characters — Star (no `@`)

- **Given:** Active credentials exist at `~/.claude/.credentials.json`.
- **When:** `clp .account.save 'name::test*file'`
- **Then:** Error message containing `must contain '@'` with exit 1. (No `@` present — `@`-absence check fires before path-safety check.); name without `@` rejected as non-email
- **Exit:** 1
- **Source:** [types.md -- AccountName](../../../../docs/cli/types.md#type--1-accountname)

---

### EC-7: Forbidden Characters — Null Byte

- **Given:** Active credentials exist at `~/.claude/.credentials.json`.
- **When:** `clp .account.save "name::foo$(printf '\0')bar"`
- **Then:** Error message containing `contains invalid characters` with exit 1.; null byte in name rejected
- **Exit:** 1
- **Source:** [types.md -- AccountName](../../../../docs/cli/types.md#type--1-accountname)

---

### EC-8: Valid Characters — Hyphens

- **Given:** Active credentials exist at `~/.claude/.credentials.json`.
- **When:** `clp .account.save name::client-a@acme.com`
- **Then:** `saved current credentials as 'client-a@acme.com'` with exit 0.; hyphenated local part in email accepted and credential file created
- **Exit:** 0
- **Source:** [params.md -- name::](../../../../docs/cli/params.md#parameter--1-name)

---

### EC-9: Valid Characters — Underscores

- **Given:** Active credentials exist at `~/.claude/.credentials.json`.
- **When:** `clp .account.save name::my_account@acme.com`
- **Then:** `saved current credentials as 'my_account@acme.com'` with exit 0.; underscored local part in email accepted and credential file created
- **Exit:** 0
- **Source:** [params.md -- name::](../../../../docs/cli/params.md#parameter--1-name)

---

### EC-10: Boundary Value — Very Long Name

- **Given:** Active credentials exist at `~/.claude/.credentials.json`. Generate a 256-character alphabetic string as the name value.
- **When:** `clp .account.save name::$(python3 -c "print('a'*256)")`
- **Then:** Either succeeds (exit 0) creating the file, or exits 1 with a clear error about name length. Must not crash, panic, or segfault.; or 1; no crash or panic on boundary-length name
- **Exit:** 0
- **Source:** [params.md -- name::](../../../../docs/cli/params.md#parameter--1-name)

---

### EC-11: Optional on `.accounts` — scopes to named account

- **Given:** `work` account exists in `~/.persistent/claude/credential/`.
- **When:** `clp .accounts name::work`
- **Then:** Exit 0; output contains one indented block starting with `work`.; `name::` scopes output to exactly one account block
- **Exit:** 0
- **Source:** [params.md -- name::](../../../../docs/cli/params.md#parameter--1-name)

---

### EC-12: Optional on `.accounts` — omitting `name::` lists all

- **Given:** Two accounts exist: `work` and `personal`.
- **When:** `clp .accounts`
- **Then:** Exit 0; output contains two account blocks.; omitting `name::` produces full account list
- **Exit:** 0
- **Source:** [params.md -- name::](../../../../docs/cli/params.md#parameter--1-name)

---

### EC-13: NotFound on `.accounts` — valid email but unknown → exit 2

- **Given:** Do NOT create a `ghost@example.com` account.
- **When:** `clp .accounts name::ghost@example.com`
- **Then:** Exit 2; stderr contains `not found` or `ghost@example.com`.; not-found is a runtime error (2), not a usage error (1)
- **Exit:** 2
- **Source:** [params.md -- name::](../../../../docs/cli/params.md#parameter--1-name)

---

### EC-14: Optional on `.account.limits` — queries named account (FR-18)

- **Given:** `work` account exists in `~/.persistent/claude/credential/`; rate-limit data available.
- **When:** `clp .account.limits name::work`
- **Then:** Exit 0; output contains utilization data for `work`.; `name::` behaves as optional lookup selector on `.account.limits`
- **Exit:** 0
- **Source:** [params.md -- name::](../../../../docs/cli/params.md#parameter--1-name) (FR-18)

---

### EC-15: Optional on `.account.limits` — omitting `name::` is valid

- **Given:** Active account configured; rate-limit data available.
- **When:** `clp .account.limits`
- **Then:** Exit 0; output contains utilization data for the active account.; omitting optional `name::` routes to active-account path
- **Exit:** 0
- **Source:** [params.md -- name::](../../../../docs/cli/params.md#parameter--1-name) (FR-18 backward-compat)

---

### EC-16: NotFound on `.account.limits` — valid chars but unknown name → exit 2

- **Given:** Do NOT create a `ghost` account.
- **When:** `clp .account.limits name::ghost`
- **Then:** Exit 2; stderr contains `not found` or `ghost`.; not-found is a runtime error (2), not a usage error (1)
- **Exit:** 2
- **Source:** [params.md -- name::](../../../../docs/cli/params.md#parameter--1-name) (FR-18)

---

### EC-17: Name Inference on `.account.save` — `emailAddress` from `~/.claude.json`

- **Given:** Active credentials exist at `~/.claude/.credentials.json`. `~/.claude.json` exists and contains `"emailAddress": "alice@acme.com"`.
- **When:** `clp .account.save` (no `name::` argument)
- **Then:** Exit 0; stdout: `saved current credentials as 'alice@acme.com'`; credential file created using the inferred email as the account name.; `name::` behaves as optional on `.account.save` when `emailAddress` is readable
- **Exit:** 0
- **Source:** [params.md -- name::](../../../../docs/cli/params.md#parameter--1-name)

---

### EC-18: Forbidden Characters — Forward Slash in Email Local Part

- **Given:** Active credentials exist at `~/.claude/.credentials.json`.
- **When:** `clp .account.save name::a/b@c.com`
- **Then:** Error message containing `path-unsafe characters` with exit 1. (`@` is present — `@`-check passes; path-safety check fires next and rejects `/` in local part `a/b`.); path-unsafe char in email local part rejected before any filesystem operation
- **Exit:** 1
- **Source:** [types.md -- AccountName](../../../../docs/cli/types.md#type--1-accountname), [params.md -- name::](../../../../docs/cli/params.md#parameter--1-name)
