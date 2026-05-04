# Test: `verbosity::` / `v::`

Edge case coverage for the `verbosity::` parameter. See [params.md](../../../../docs/cli/params.md#parameter--2-verbosity--v) and [types.md](../../../../docs/cli/types.md#type--2-verbositylevel) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `v::0` produces quiet output (bare status word) | Valid Level |
| EC-2 | `v::1` produces default labeled output | Valid Level |
| EC-3 | `v::2` produces verbose output with raw timestamp | Valid Level |
| EC-4 | `v::3` exits 1 (out of range) | Out of Range |
| EC-5 | `v::abc` exits 1 (non-integer) | Invalid Type |
| EC-6 | `verbosity::1` long-form alias works same as `v::1` | Alias |
| EC-7 | Duplicate `v::0 v::2` — last wins (produces verbose) | Last Wins |
| EC-8 | Omitted `v::` defaults to `v::1` | Default |

### Test Coverage Summary

- Valid Level: 3 tests
- Out of Range: 1 test
- Invalid Type: 1 test
- Alias: 1 test
- Last Wins: 1 test
- Default: 1 test

**Total:** 8 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

---

### EC-1: Valid Level — Quiet

- **Given:** `~/.claude/.credentials.json` exists with a far-future `expiresAt` value so the token is valid.
- **When:** `clp .token.status v::0`
- **Then:** Bare status word only (e.g., `valid`). No dash, no remaining duration, no labels. Exit 0.; output is bare status word without any labels or metadata
- **Exit:** 0
- **Source:** [params.md -- verbosity::](../../../../docs/cli/params.md#parameter--2-verbosity--v)

---

### EC-2: Valid Level — Normal

- **Given:** `~/.claude/.credentials.json` exists with a far-future `expiresAt` value so the token is valid.
- **When:** `clp .token.status v::1`
- **Then:** Status word with remaining duration (e.g., `valid — 47h30m remaining`). Exit 0.; labeled output with status and remaining duration
- **Exit:** 0
- **Source:** [params.md -- verbosity::](../../../../docs/cli/params.md#parameter--2-verbosity--v)

---

### EC-3: Valid Level — Verbose

- **Given:** `~/.claude/.credentials.json` exists with a far-future `expiresAt` value.
- **When:** `clp .token.status v::2`
- **Then:** Status word with remaining duration plus raw `expiresAt` value and effective threshold. Exit 0.; verbose output includes all available metadata fields
- **Exit:** 0
- **Source:** [params.md -- verbosity::](../../../../docs/cli/params.md#parameter--2-verbosity--v)

---

### EC-4: Out of Range

- **Given:** No special credential state needed (adapter rejects before command runs).
- **When:** `clp .token.status v::3`
- **Then:** Error message containing `verbosity must be 0, 1, or 2` or similar with exit 1.; out-of-range verbosity level rejected with descriptive error
- **Exit:** 1
- **Source:** [types.md -- VerbosityLevel](../../../../docs/cli/types.md#type--2-verbositylevel)

---

### EC-5: Invalid Type

- **Given:** No special credential state needed (adapter rejects before command runs).
- **When:** `clp .token.status v::abc`
- **Then:** Error message indicating non-integer value with exit 1.; non-integer verbosity value rejected
- **Exit:** 1
- **Source:** [types.md -- VerbosityLevel](../../../../docs/cli/types.md#type--2-verbositylevel)

---

### EC-6: Alias

- **Given:** `~/.claude/.credentials.json` exists with a far-future `expiresAt` value.
- **When:** `clp .token.status verbosity::1`
- **Then:** Same labeled output as `clp .token.status v::1`. Exit 0.; `verbosity::1` output identical to `v::1` output
- **Exit:** 0
- **Source:** [params.md -- verbosity::](../../../../docs/cli/params.md#parameter--2-verbosity--v)

---

### EC-7: Last Wins

- **Given:** `~/.claude/.credentials.json` exists with a far-future `expiresAt` value.
- **When:** `clp .token.status v::0 v::2`
- **Then:** Verbose output (matching `v::2` behavior), not quiet bare output. Exit 0.; last `v::` value (2) takes effect, producing verbose output
- **Exit:** 0
- **Source:** [params.md -- verbosity::](../../../../docs/cli/params.md#parameter--2-verbosity--v)

---

### EC-8: Default

- **Given:** `~/.claude/.credentials.json` exists with a far-future `expiresAt` value.
- **When:** `clp .token.status`
- **Then:** Same labeled output as `clp .token.status v::1`. Exit 0.; default output matches `v::1` labeled format
- **Exit:** 0
- **Source:** [params.md -- verbosity::](../../../../docs/cli/params.md#parameter--2-verbosity--v)
