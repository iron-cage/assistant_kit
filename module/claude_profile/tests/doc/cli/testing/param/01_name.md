# Test: `name::`

Edge case coverage for the `name::` parameter. See [params.md](../../../../../docs/cli/params.md#parameter--1-name) and [types.md](../../../../../docs/cli/types.md#type--1-accountname) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `name::work` — valid name accepted | Valid Name |
| EC-2 | `name::` (empty value) rejected with exit 1 | Empty Value |
| EC-3 | Omitted `name::` on required command exits 1 | Required Parameter |
| EC-4 | `name::` with `/` rejected with exit 1 | Forbidden Characters |
| EC-5 | `name::` with `\` rejected with exit 1 | Forbidden Characters |
| EC-6 | `name::` with `*?"<>\|` rejected with exit 1 | Forbidden Characters |
| EC-7 | `name::` with null byte rejected with exit 1 | Forbidden Characters |
| EC-8 | `name::client-a` — hyphens accepted | Valid Characters |
| EC-9 | `name::my_account` — underscores accepted | Valid Characters |
| EC-10 | Very long name (>255 chars) handled without crash | Boundary Value |
| EC-11 | `.account.status name::work` — optional, queries named account (FR-16) | Optional on Status |
| EC-12 | `.account.status` without `name::` — omitting is valid (active-account path) | Optional on Status |
| EC-13 | `.account.status name::ghost` — valid chars but unknown → exit 2 (FR-16) | NotFound on Status |
| EC-14 | `.account.limits name::work` — optional, queries named account (FR-18) | Optional on Limits |
| EC-15 | `.account.limits` without `name::` — omitting is valid (active-account path) | Optional on Limits |
| EC-16 | `.account.limits name::ghost` — valid chars but unknown → exit 2 (FR-18) | NotFound on Limits |

### Test Coverage Summary

- Valid Name: 1 test
- Empty Value: 1 test
- Required Parameter: 1 test
- Forbidden Characters: 4 tests
- Valid Characters: 2 tests
- Boundary Value: 1 test
- Optional on Status (FR-16): 3 tests
- Optional on Limits (FR-18): 3 tests

**Total:** 16 edge cases

---

### EC-1: Valid Name

**Goal:** Confirm that a simple alphanumeric name is accepted and the command succeeds.
**Setup:** Active credentials exist at `~/.claude/.credentials.json`.
**Command:** `clp .account.save name::work`
**Expected Output:** `saved current credentials as 'work'` with exit 0.
**Verification:**
- Exit code is 0
- Output contains `saved current credentials as 'work'`
- File `~/.claude/accounts/work.credentials.json` exists after execution
**Pass Criteria:** Exit 0; credential file created with correct name.
**Source:** [params.md -- name::](../../../../../docs/cli/params.md#parameter--1-name)

---

### EC-2: Empty Value

**Goal:** Confirm that an empty `name::` value is rejected as a usage error.
**Setup:** Active credentials exist at `~/.claude/.credentials.json`.
**Command:** `clp .account.save name::`
**Expected Output:** Error message containing `account name must not be empty` with exit 1.
**Verification:**
- Exit code is 1
- Stderr contains `account name must not be empty`
- No file created under `~/.claude/accounts/`
**Pass Criteria:** Exit 1; empty name rejected with descriptive error.
**Source:** [types.md -- AccountName](../../../../../docs/cli/types.md#type--1-accountname)

---

### EC-3: Required Parameter

**Goal:** Confirm that omitting the required `name::` parameter produces a usage error.
**Setup:** Active credentials exist at `~/.claude/.credentials.json`.
**Command:** `clp .account.save`
**Expected Output:** Error message indicating `name::` is required with exit 1.
**Verification:**
- Exit code is 1
- Stderr references the missing `name::` parameter
- No file created under `~/.claude/accounts/`
**Pass Criteria:** Exit 1; missing required parameter clearly reported.
**Source:** [params.md -- name::](../../../../../docs/cli/params.md#parameter--1-name)

---

### EC-4: Forbidden Characters — Forward Slash

**Goal:** Confirm that a name containing `/` is rejected to prevent path traversal.
**Setup:** Active credentials exist at `~/.claude/.credentials.json`.
**Command:** `clp .account.save name::foo/bar`
**Expected Output:** Error message containing `contains invalid characters` with exit 1.
**Verification:**
- Exit code is 1
- Stderr contains `contains invalid characters`
- No file created under `~/.claude/accounts/`
**Pass Criteria:** Exit 1; forward slash in name rejected.
**Source:** [types.md -- AccountName](../../../../../docs/cli/types.md#type--1-accountname)

---

### EC-5: Forbidden Characters — Backslash

**Goal:** Confirm that a name containing `\` is rejected to prevent filesystem issues.
**Setup:** Active credentials exist at `~/.claude/.credentials.json`.
**Command:** `clp .account.save 'name::foo\bar'`
**Expected Output:** Error message containing `contains invalid characters` with exit 1.
**Verification:**
- Exit code is 1
- Stderr contains `contains invalid characters`
- No file created under `~/.claude/accounts/`
**Pass Criteria:** Exit 1; backslash in name rejected.
**Source:** [types.md -- AccountName](../../../../../docs/cli/types.md#type--1-accountname)

---

### EC-6: Forbidden Characters — Special Characters

**Goal:** Confirm that names containing any of `*?"<>|` are rejected.
**Setup:** Active credentials exist at `~/.claude/.credentials.json`.
**Command:** `clp .account.save 'name::test*file'`
**Expected Output:** Error message containing `contains invalid characters` with exit 1.
**Verification:**
- Exit code is 1
- Stderr contains `contains invalid characters`
- No file created under `~/.claude/accounts/`
- Repeat conceptually for each forbidden character: `?`, `"`, `<`, `>`, `|`
**Pass Criteria:** Exit 1; each forbidden special character in name rejected.
**Source:** [types.md -- AccountName](../../../../../docs/cli/types.md#type--1-accountname)

---

### EC-7: Forbidden Characters — Null Byte

**Goal:** Confirm that a name containing a null byte (`\0`) is rejected.
**Setup:** Active credentials exist at `~/.claude/.credentials.json`.
**Command:** `clp .account.save "name::foo$(printf '\0')bar"`
**Expected Output:** Error message containing `contains invalid characters` with exit 1.
**Verification:**
- Exit code is 1
- Stderr contains `contains invalid characters`
- No file created under `~/.claude/accounts/`
**Pass Criteria:** Exit 1; null byte in name rejected.
**Source:** [types.md -- AccountName](../../../../../docs/cli/types.md#type--1-accountname)

---

### EC-8: Valid Characters — Hyphens

**Goal:** Confirm that hyphens are accepted in account names.
**Setup:** Active credentials exist at `~/.claude/.credentials.json`.
**Command:** `clp .account.save name::client-a`
**Expected Output:** `saved current credentials as 'client-a'` with exit 0.
**Verification:**
- Exit code is 0
- Output contains `saved current credentials as 'client-a'`
- File `~/.claude/accounts/client-a.credentials.json` exists after execution
**Pass Criteria:** Exit 0; hyphenated name accepted and credential file created.
**Source:** [params.md -- name::](../../../../../docs/cli/params.md#parameter--1-name)

---

### EC-9: Valid Characters — Underscores

**Goal:** Confirm that underscores are accepted in account names.
**Setup:** Active credentials exist at `~/.claude/.credentials.json`.
**Command:** `clp .account.save name::my_account`
**Expected Output:** `saved current credentials as 'my_account'` with exit 0.
**Verification:**
- Exit code is 0
- Output contains `saved current credentials as 'my_account'`
- File `~/.claude/accounts/my_account.credentials.json` exists after execution
**Pass Criteria:** Exit 0; underscored name accepted and credential file created.
**Source:** [params.md -- name::](../../../../../docs/cli/params.md#parameter--1-name)

---

### EC-10: Boundary Value — Very Long Name

**Goal:** Confirm that a name exceeding 255 characters does not crash the binary.
**Setup:** Active credentials exist at `~/.claude/.credentials.json`. Generate a 256-character alphabetic string as the name value.
**Command:** `clp .account.save name::$(python3 -c "print('a'*256)")`
**Expected Output:** Either succeeds (exit 0) creating the file, or exits 1 with a clear error about name length. Must not crash, panic, or segfault.
**Verification:**
- Exit code is 0 or 1 (not a signal-based termination)
- No panic backtrace in stderr
- If exit 0: corresponding credential file exists
- If exit 1: error message references name length or filesystem limitation
**Pass Criteria:** Exit 0 or 1; no crash or panic on boundary-length name.
**Source:** [params.md -- name::](../../../../../docs/cli/params.md#parameter--1-name)

---

### EC-11: Optional on `.account.status` — queries named account (FR-16)

**Goal:** Confirm that `name::work` on `.account.status` succeeds and shows the named account's state.
**Setup:** Write `work` as active account with valid credentials.
**Command:** `clp .account.status name::work`
**Expected Output:** Exit 0; output contains `work` and `valid`.
**Verification:**
- Exit code is 0
- Stdout contains `work`
- Stdout contains `valid`
**Pass Criteria:** Exit 0; `name::` behaves as optional lookup selector on `.account.status`.
**Source:** [params.md -- name::](../../../../../docs/cli/params.md#parameter--1-name) (FR-16)

---

### EC-12: Optional on `.account.status` — omitting `name::` is valid

**Goal:** Confirm that `.account.status` without `name::` still works (backward-compat active-account path).
**Setup:** Write `work` as active account with valid credentials.
**Command:** `clp .account.status`
**Expected Output:** Exit 0; output contains `work` and `valid`.
**Verification:**
- Exit code is 0
- Stdout contains `work`
**Pass Criteria:** Exit 0; omitting optional `name::` routes to active-account path.
**Source:** [params.md -- name::](../../../../../docs/cli/params.md#parameter--1-name) (FR-16 backward-compat)

---

### EC-13: NotFound on `.account.status` — valid chars but unknown name → exit 2

**Goal:** Confirm that a syntactically valid but non-existent account name exits 2, not 1.
**Setup:** Write `work` as active. Do NOT create a `ghost` account.
**Command:** `clp .account.status name::ghost`
**Expected Output:** Exit 2; stderr contains `not found` or `ghost`.
**Verification:**
- Exit code is 2
- Stderr contains `not found` or `ghost`
- Stdout is empty
**Pass Criteria:** Exit 2; not-found is a runtime error (2), not a usage error (1).
**Source:** [params.md -- name::](../../../../../docs/cli/params.md#parameter--1-name) (FR-16)

---

### EC-14: Optional on `.account.limits` — queries named account (FR-18)

**Goal:** Confirm that `name::work` on `.account.limits` succeeds and shows that account's limits.
**Setup:** `work` account exists in `~/.claude/accounts/`; rate-limit data available.
**Command:** `clp .account.limits name::work`
**Expected Output:** Exit 0; output contains utilization data for `work`.
**Verification:**
- Exit code is 0
- Output is well-formed (contains percentage values)
**Pass Criteria:** Exit 0; `name::` behaves as optional lookup selector on `.account.limits`.
**Source:** [params.md -- name::](../../../../../docs/cli/params.md#parameter--1-name) (FR-18)

---

### EC-15: Optional on `.account.limits` — omitting `name::` is valid

**Goal:** Confirm that `.account.limits` without `name::` shows the active account's limits.
**Setup:** Active account configured; rate-limit data available.
**Command:** `clp .account.limits`
**Expected Output:** Exit 0; output contains utilization data for the active account.
**Verification:**
- Exit code is 0
- Output is well-formed
**Pass Criteria:** Exit 0; omitting optional `name::` routes to active-account path.
**Source:** [params.md -- name::](../../../../../docs/cli/params.md#parameter--1-name) (FR-18 backward-compat)

---

### EC-16: NotFound on `.account.limits` — valid chars but unknown name → exit 2

**Goal:** Confirm that a syntactically valid but non-existent account name exits 2, not 1.
**Setup:** Do NOT create a `ghost` account.
**Command:** `clp .account.limits name::ghost`
**Expected Output:** Exit 2; stderr contains `not found` or `ghost`.
**Verification:**
- Exit code is 2
- Stderr contains `not found` or `ghost`
**Pass Criteria:** Exit 2; not-found is a runtime error (2), not a usage error (1).
**Source:** [params.md -- name::](../../../../../docs/cli/params.md#parameter--1-name) (FR-18)
