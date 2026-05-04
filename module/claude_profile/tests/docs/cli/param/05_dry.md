# Test: `dry::`

Edge case coverage for the `dry::` parameter. See [params.md](../../../../docs/cli/params.md#parameter--5-dry) for specification.

### Test Case Index

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

### Test Coverage Summary

- Dry Output: 1 test
- Normal Execution: 1 test
- Boolean Alias: 2 tests
- Invalid Value: 1 test
- No Side Effects: 1 test
- Fidelity: 1 test
- Default: 1 test

**Total:** 8 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

---

### EC-1: Dry Output

- **Given:** Active credentials exist at `~/.claude/.credentials.json`. No account named `preview` exists.
- **When:** `clp .account.save name::preview dry::1`
- **Then:** `[dry-run] would save current credentials as 'preview'` with exit 0.; output has `[dry-run]` prefix and no file is created
- **Exit:** 0
- **Source:** [params.md -- dry::](../../../../docs/cli/params.md#parameter--5-dry)

---

### EC-2: Normal Execution

- **Given:** Active credentials exist at `~/.claude/.credentials.json`. No account named `real` exists.
- **When:** `clp .account.save name::real dry::0`
- **Then:** `saved current credentials as 'real'` with exit 0.; command executes normally with file created
- **Exit:** 0
- **Source:** [params.md -- dry::](../../../../docs/cli/params.md#parameter--5-dry)

---

### EC-3: Boolean Alias — True

- **Given:** Active credentials exist at `~/.claude/.credentials.json`. No account named `alias_test` exists.
- **When:** `clp .account.save name::alias_test dry::true`
- **Then:** `[dry-run] would save current credentials as 'alias_test'` with exit 0.; `dry::true` behaves identically to `dry::1`
- **Exit:** 0
- **Source:** [params.md -- dry::](../../../../docs/cli/params.md#parameter--5-dry)

---

### EC-4: Boolean Alias — False

- **Given:** Active credentials exist at `~/.claude/.credentials.json`. No account named `alias_test2` exists.
- **When:** `clp .account.save name::alias_test2 dry::false`
- **Then:** `saved current credentials as 'alias_test2'` with exit 0.; `dry::false` behaves identically to `dry::0`
- **Exit:** 0
- **Source:** [params.md -- dry::](../../../../docs/cli/params.md#parameter--5-dry)

---

### EC-5: Invalid Value

- **Given:** Active credentials exist at `~/.claude/.credentials.json`.
- **When:** `clp .account.save name::test dry::abc`
- **Then:** Error message indicating `abc` is not a valid boolean value with exit 1.; non-boolean `dry::` value rejected with descriptive error
- **Exit:** 1
- **Source:** [params.md -- dry::](../../../../docs/cli/params.md#parameter--5-dry)

---

### EC-6: No Side Effects

- **Given:** Active credentials exist at `~/.claude/.credentials.json`. Record file listing and checksums of `~/.claude/` and `~/.persistent/claude/credential/` before execution.
- **When:** `clp .account.save name::sideeffect dry::1`
- **Then:** `[dry-run] would save current credentials as 'sideeffect'` with exit 0.; filesystem state unchanged after dry-run execution
- **Exit:** 0
- **Source:** [params.md -- dry::](../../../../docs/cli/params.md#parameter--5-dry)

---

### EC-7: Fidelity

- **Given:** Active credentials exist at `~/.claude/.credentials.json`. No account named `fidelity` exists.
- **When:**
  1. `clp .account.save name::fidelity dry::1`
  2. `clp .account.save name::fidelity dry::0`
- **Then:** 1. `[dry-run] would save current credentials as 'fidelity'` with exit 0.
2. `saved current credentials as 'fidelity'` with exit 0.; for both; dry-run success implies execute success with identical semantics
- **Exit:** 0
- **Source:** [params.md -- dry::](../../../../docs/cli/params.md#parameter--5-dry)

---

### EC-8: Default

- **Given:** Active credentials exist at `~/.claude/.credentials.json`. No account named `default_test` exists.
- **When:** `clp .account.save name::default_test`
- **Then:** `saved current credentials as 'default_test'` with exit 0.; default behavior is normal execution without dry-run prefix
- **Exit:** 0
- **Source:** [params.md -- dry::](../../../../docs/cli/params.md#parameter--5-dry)
