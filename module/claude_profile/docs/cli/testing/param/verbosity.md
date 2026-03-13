# Test: `verbosity::` / `v::`

Edge case coverage for the `verbosity::` parameter. See [params.md](../../params.md#parameter--2-verbosity--v) and [types.md](../../types.md#type--2-verbositylevel) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `v::0` produces quiet output (bare values) | Valid Level |
| EC-2 | `v::1` produces default labeled output | Valid Level |
| EC-3 | `v::2` produces verbose output with full metadata | Valid Level |
| EC-4 | `v::3` exits 1 (out of range) | Out of Range |
| EC-5 | `v::abc` exits 1 (non-integer) | Invalid Type |
| EC-6 | `verbosity::1` long-form alias works same as `v::1` | Alias |
| EC-7 | Duplicate `v::0 v::2` — last wins (produces verbose) | Last Wins |
| EC-8 | Omitted `v::` defaults to `v::1` | Default |

## Test Coverage Summary

- Valid Level: 3 tests
- Out of Range: 1 test
- Invalid Type: 1 test
- Alias: 1 test
- Last Wins: 1 test
- Default: 1 test

**Total:** 8 edge cases

---

### EC-1: Valid Level — Quiet

**Goal:** Confirm that `v::0` suppresses labels and produces bare values only.
**Setup:** At least one saved account exists under `~/.claude/accounts/`.
**Command:** `clp .account.list v::0`
**Expected Output:** Bare account names only, one per line, no labels, no active indicator, no subscription metadata. Exit 0.
**Verification:**
- Exit code is 0
- Output contains account names as plain text
- Output does not contain `<- active`, subscription type, tier, or expiry information
**Pass Criteria:** Exit 0; output is bare values without any labels or metadata.
**Source:** [params.md -- verbosity::](../../params.md#parameter--2-verbosity--v)

---

### EC-2: Valid Level — Normal

**Goal:** Confirm that `v::1` produces labeled output with human context.
**Setup:** At least one saved account exists under `~/.claude/accounts/`, with one marked active.
**Command:** `clp .account.list v::1`
**Expected Output:** Account names with `<- active` marker on the active account and subscription type. Exit 0.
**Verification:**
- Exit code is 0
- Active account line contains `<- active`
- Output contains subscription type (e.g., `max`, `pro`)
- Output includes human-readable expiry (e.g., `expires in 47m`)
**Pass Criteria:** Exit 0; labeled output with active indicator and subscription type.
**Source:** [params.md -- verbosity::](../../params.md#parameter--2-verbosity--v)

---

### EC-3: Valid Level — Verbose

**Goal:** Confirm that `v::2` includes full metadata including tier and expiry details.
**Setup:** At least one saved account exists under `~/.claude/accounts/`.
**Command:** `clp .account.list v::2`
**Expected Output:** Account names with subscription type, rate-limit tier, and full expiry timestamp. Exit 0.
**Verification:**
- Exit code is 0
- Output contains subscription type (e.g., `max`, `pro`)
- Output contains rate-limit tier (e.g., `standard`)
- Output contains expiry information with more detail than `v::1`
**Pass Criteria:** Exit 0; verbose output includes all available metadata fields.
**Source:** [params.md -- verbosity::](../../params.md#parameter--2-verbosity--v)

---

### EC-4: Out of Range

**Goal:** Confirm that `v::3` is rejected as outside the valid 0-2 range.
**Setup:** At least one saved account exists under `~/.claude/accounts/`.
**Command:** `clp .account.list v::3`
**Expected Output:** Error message containing `verbosity must be 0-2` with exit 1.
**Verification:**
- Exit code is 1
- Stderr contains `verbosity must be 0-2`
- No account listing produced on stdout
**Pass Criteria:** Exit 1; out-of-range verbosity level rejected with descriptive error.
**Source:** [types.md -- VerbosityLevel](../../types.md#type--2-verbositylevel)

---

### EC-5: Invalid Type

**Goal:** Confirm that a non-integer verbosity value is rejected.
**Setup:** At least one saved account exists under `~/.claude/accounts/`.
**Command:** `clp .account.list v::abc`
**Expected Output:** Error message indicating non-integer value with exit 1.
**Verification:**
- Exit code is 1
- Stderr indicates the value is not a valid integer
- No account listing produced on stdout
**Pass Criteria:** Exit 1; non-integer verbosity value rejected.
**Source:** [types.md -- VerbosityLevel](../../types.md#type--2-verbositylevel)

---

### EC-6: Alias

**Goal:** Confirm that the long-form `verbosity::` alias produces identical output to the short-form `v::`.
**Setup:** At least one saved account exists under `~/.claude/accounts/`.
**Command:** `clp .account.list verbosity::1`
**Expected Output:** Same labeled output as `clp .account.list v::1`. Exit 0.
**Verification:**
- Exit code is 0
- Output matches the output of `clp .account.list v::1` exactly
- Active marker, subscription type, and expiry present as with `v::1`
**Pass Criteria:** Exit 0; `verbosity::1` output identical to `v::1` output.
**Source:** [params.md -- verbosity::](../../params.md#parameter--2-verbosity--v)

---

### EC-7: Last Wins

**Goal:** Confirm that when `v::` is specified multiple times, the last occurrence takes precedence.
**Setup:** At least one saved account exists under `~/.claude/accounts/`.
**Command:** `clp .account.list v::0 v::2`
**Expected Output:** Verbose output (matching `v::2` behavior), not quiet output. Exit 0.
**Verification:**
- Exit code is 0
- Output contains full metadata (subscription type, tier, expiry) consistent with `v::2`
- Output is not bare values (which would indicate `v::0` won)
**Pass Criteria:** Exit 0; last `v::` value (2) takes effect, producing verbose output.
**Source:** [params.md -- verbosity::](../../params.md#parameter--2-verbosity--v)

---

### EC-8: Default

**Goal:** Confirm that omitting `v::` defaults to `v::1` (normal labeled output).
**Setup:** At least one saved account exists under `~/.claude/accounts/`, with one marked active.
**Command:** `clp .account.list`
**Expected Output:** Same labeled output as `clp .account.list v::1`. Exit 0.
**Verification:**
- Exit code is 0
- Output contains `<- active` marker on the active account
- Output contains subscription type
- Output matches `v::1` behavior (not bare values, not full verbose metadata)
**Pass Criteria:** Exit 0; default output matches `v::1` labeled format.
**Source:** [params.md -- verbosity::](../../params.md#parameter--2-verbosity--v)
