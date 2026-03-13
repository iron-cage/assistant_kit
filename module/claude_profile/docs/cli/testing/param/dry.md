# Test: `dry::`

Edge case coverage for the `dry::` parameter. See [params.md](../../params.md#parameter--5-dry) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `dry::1` prints `[dry-run]` prefixed message | Dry Output |
| EC-2 | `dry::0` executes normally (default) | Normal Execution |
| EC-3 | `dry::true` equivalent to `dry::1` | Boolean Alias |
| EC-4 | `dry::false` equivalent to `dry::0` | Boolean Alias |
| EC-5 | `dry::abc` exits 1 (invalid boolean) | Invalid Value |
| EC-6 | `dry::1` does not create, modify, or delete any files | No Side Effects |
| EC-7 | `dry::1` validation matches `dry::0` — if dry succeeds, execute succeeds | Fidelity |
| EC-8 | Omitted `dry::` defaults to `dry::0` | Default |

## Test Coverage Summary

- Dry Output: 1 test
- Normal Execution: 1 test
- Boolean Alias: 2 tests
- Invalid Value: 1 test
- No Side Effects: 1 test
- Fidelity: 1 test
- Default: 1 test

**Total:** 8 edge cases

---

### EC-1: Dry Output

**Goal:** Confirm that `dry::1` prints a `[dry-run]` prefixed message describing the intended action.
**Setup:** Active credentials exist at `~/.claude/.credentials.json`. No account named `preview` exists.
**Command:** `clp .account.save name::preview dry::1`
**Expected Output:** `[dry-run] would save current credentials as 'preview'` with exit 0.
**Verification:**
- Exit code is 0
- Output starts with `[dry-run]`
- Output contains `would save current credentials as 'preview'`
- File `~/.claude/accounts/preview.credentials.json` does not exist after execution
**Pass Criteria:** Exit 0; output has `[dry-run]` prefix and no file is created.
**Source:** [params.md -- dry::](../../params.md#parameter--5-dry)

---

### EC-2: Normal Execution

**Goal:** Confirm that `dry::0` executes the command normally, modifying files.
**Setup:** Active credentials exist at `~/.claude/.credentials.json`. No account named `real` exists.
**Command:** `clp .account.save name::real dry::0`
**Expected Output:** `saved current credentials as 'real'` with exit 0.
**Verification:**
- Exit code is 0
- Output contains `saved current credentials as 'real'`
- Output does not contain `[dry-run]`
- File `~/.claude/accounts/real.credentials.json` exists after execution
**Pass Criteria:** Exit 0; command executes normally with file created.
**Source:** [params.md -- dry::](../../params.md#parameter--5-dry)

---

### EC-3: Boolean Alias — True

**Goal:** Confirm that `dry::true` is equivalent to `dry::1`.
**Setup:** Active credentials exist at `~/.claude/.credentials.json`. No account named `alias_test` exists.
**Command:** `clp .account.save name::alias_test dry::true`
**Expected Output:** `[dry-run] would save current credentials as 'alias_test'` with exit 0.
**Verification:**
- Exit code is 0
- Output starts with `[dry-run]`
- Output matches the output of `clp .account.save name::alias_test dry::1`
- File `~/.claude/accounts/alias_test.credentials.json` does not exist after execution
**Pass Criteria:** Exit 0; `dry::true` behaves identically to `dry::1`.
**Source:** [params.md -- dry::](../../params.md#parameter--5-dry)

---

### EC-4: Boolean Alias — False

**Goal:** Confirm that `dry::false` is equivalent to `dry::0`.
**Setup:** Active credentials exist at `~/.claude/.credentials.json`. No account named `alias_test2` exists.
**Command:** `clp .account.save name::alias_test2 dry::false`
**Expected Output:** `saved current credentials as 'alias_test2'` with exit 0.
**Verification:**
- Exit code is 0
- Output contains `saved current credentials as 'alias_test2'`
- Output does not contain `[dry-run]`
- File `~/.claude/accounts/alias_test2.credentials.json` exists after execution
**Pass Criteria:** Exit 0; `dry::false` behaves identically to `dry::0`.
**Source:** [params.md -- dry::](../../params.md#parameter--5-dry)

---

### EC-5: Invalid Value

**Goal:** Confirm that an unrecognized boolean value is rejected.
**Setup:** Active credentials exist at `~/.claude/.credentials.json`.
**Command:** `clp .account.save name::test dry::abc`
**Expected Output:** Error message indicating `abc` is not a valid boolean value with exit 1.
**Verification:**
- Exit code is 1
- Stderr indicates the value is not a valid boolean
- No file created under `~/.claude/accounts/`
**Pass Criteria:** Exit 1; non-boolean `dry::` value rejected with descriptive error.
**Source:** [params.md -- dry::](../../params.md#parameter--5-dry)

---

### EC-6: No Side Effects

**Goal:** Confirm that `dry::1` does not create, modify, or delete any files on disk.
**Setup:** Active credentials exist at `~/.claude/.credentials.json`. Record file listing and checksums of `~/.claude/` and `~/.claude/accounts/` before execution.
**Command:** `clp .account.save name::sideeffect dry::1`
**Expected Output:** `[dry-run] would save current credentials as 'sideeffect'` with exit 0.
**Verification:**
- Exit code is 0
- File `~/.claude/accounts/sideeffect.credentials.json` does not exist
- File listing of `~/.claude/` is identical before and after execution
- File listing of `~/.claude/accounts/` is identical before and after execution
- No new files, no modified timestamps, no deleted files
**Pass Criteria:** Exit 0; filesystem state unchanged after dry-run execution.
**Source:** [params.md -- dry::](../../params.md#parameter--5-dry)

---

### EC-7: Fidelity

**Goal:** Confirm that a successful `dry::1` run guarantees the subsequent `dry::0` run will succeed with the same action.
**Setup:** Active credentials exist at `~/.claude/.credentials.json`. No account named `fidelity` exists.
**Command:**
1. `clp .account.save name::fidelity dry::1`
2. `clp .account.save name::fidelity dry::0`
**Expected Output:**
1. `[dry-run] would save current credentials as 'fidelity'` with exit 0.
2. `saved current credentials as 'fidelity'` with exit 0.
**Verification:**
- Both commands exit 0
- Dry-run output describes the same action that execute performs
- File `~/.claude/accounts/fidelity.credentials.json` does not exist after step 1
- File `~/.claude/accounts/fidelity.credentials.json` exists after step 2
**Pass Criteria:** Exit 0 for both; dry-run success implies execute success with identical semantics.
**Source:** [params.md -- dry::](../../params.md#parameter--5-dry)

---

### EC-8: Default

**Goal:** Confirm that omitting `dry::` defaults to `dry::0` (normal execution).
**Setup:** Active credentials exist at `~/.claude/.credentials.json`. No account named `default_test` exists.
**Command:** `clp .account.save name::default_test`
**Expected Output:** `saved current credentials as 'default_test'` with exit 0.
**Verification:**
- Exit code is 0
- Output contains `saved current credentials as 'default_test'`
- Output does not contain `[dry-run]`
- File `~/.claude/accounts/default_test.credentials.json` exists after execution
**Pass Criteria:** Exit 0; default behavior is normal execution without dry-run prefix.
**Source:** [params.md -- dry::](../../params.md#parameter--5-dry)
