# Test: Output Control Group

Integration and edge case coverage for the Output Control parameter group (`v::`, `format::`). See [parameter_groups.md](../../../../../docs/cli/parameter_groups.md#group--1-output-control) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `v::0` suppresses labels across all 3 supported commands | Quiet Mode |
| IT-2 | `v::1` (default) produces labeled output across all 3 commands | Standard Mode |
| IT-3 | `v::2` includes full metadata across all 3 commands | Verbose Mode |
| IT-4 | `verbosity::` long-form alias is cross-command consistent with `v::` | Alias Consistency |
| EC-1 | `v::0` with `format::json` — quiet flag does not strip JSON keys | Interaction |
| EC-2 | `v::2` with `format::json` — verbose flag does not add extra JSON keys | Interaction |
| EC-3 | `v::0` exit code unchanged — quiet mode does not suppress errors | Exit Code Preservation |
| EC-4 | Output Control params ignored by mutation commands (save, switch, delete) | Non-Applicability |

### Test Coverage Summary

- Quiet Mode: 1 test
- Standard Mode: 1 test
- Verbose Mode: 1 test
- Alias Consistency: 1 test
- Interaction: 2 tests
- Exit Code Preservation: 1 test
- Non-Applicability: 1 test

**Total:** 8 tests (4 integration, 4 edge cases)

---

### IT-1: Quiet Mode

**Goal:** Confirm that `v::0` suppresses labels and produces bare values across all 3 commands that accept Output Control.
**Setup:** At least one saved account exists under `~/.persistent/claude/credential/`. Active credentials exist at `~/.claude/.credentials.json`.
**Command:**
1. `clp .account.list v::0`
2. `clp .token.status v::0`
3. `clp .paths v::0`
**Expected Output:**
1. Bare account names, one per line, no labels or metadata.
2. Bare status word (e.g., `valid`), no label prefix or remaining time detail.
3. Bare base path (e.g., `/home/user/.claude`), no path labels.
All exit 0.
**Verification:**
- All 3 commands exit 0
- `.account.list v::0` output does not contain `<- active`, subscription type, or tier
- `.token.status v::0` output is a single status word without labels
- `.paths v::0` output is a single path without field labels like `credentials:` or `accounts:`
**Pass Criteria:** Exit 0 for all 3; `v::0` consistently suppresses labels across all Output Control commands.
**Source:** [parameter_groups.md -- Output Control](../../../../../docs/cli/parameter_groups.md#group--1-output-control)

---

### IT-2: Standard Mode

**Goal:** Confirm that `v::1` produces labeled output with human context across all 3 commands.
**Setup:** At least one saved account exists under `~/.persistent/claude/credential/`, with one marked active. Active credentials exist at `~/.claude/.credentials.json`.
**Command:**
1. `clp .account.list v::1`
2. `clp .token.status v::1`
3. `clp .paths v::1`
**Expected Output:**
1. Account names with `<- active` marker and subscription type (e.g., `work <- active (max, standard, expires in 47m)`).
2. Status with label and remaining time (e.g., `valid -- 47m remaining`).
3. Labeled paths (e.g., `credentials: /home/user/.claude/.credentials.json`).
All exit 0.
**Verification:**
- All 3 commands exit 0
- `.account.list v::1` output contains `<- active` on the active account and subscription type
- `.token.status v::1` output contains a remaining time indicator
- `.paths v::1` output contains labeled path entries (e.g., `credentials:`, `accounts:`)
**Pass Criteria:** Exit 0 for all 3; `v::1` consistently produces labeled output across all Output Control commands.
**Source:** [parameter_groups.md -- Output Control](../../../../../docs/cli/parameter_groups.md#group--1-output-control)

---

### IT-3: Verbose Mode

**Goal:** Confirm that `v::2` includes full metadata across all 3 commands.
**Setup:** At least one saved account exists under `~/.persistent/claude/credential/`. Active credentials exist at `~/.claude/.credentials.json`.
**Command:**
1. `clp .account.list v::2`
2. `clp .token.status v::2`
3. `clp .paths v::2`
**Expected Output:**
1. Account names with subscription type, rate-limit tier, and full expiry timestamp.
2. Status with detailed expiry information including raw seconds or full timestamp.
3. All canonical paths with labels and expanded detail.
All exit 0.
**Verification:**
- All 3 commands exit 0
- `.account.list v::2` output contains rate-limit tier (e.g., `standard`) and expiry details beyond `v::1`
- `.token.status v::2` output contains more detail than `v::1` (e.g., raw seconds, full timestamp)
- `.paths v::2` output includes more paths or metadata than `v::1`
**Pass Criteria:** Exit 0 for all 3; `v::2` consistently provides maximum metadata across all Output Control commands.
**Source:** [parameter_groups.md -- Output Control](../../../../../docs/cli/parameter_groups.md#group--1-output-control)

---

### IT-4: Alias Consistency

**Goal:** Confirm that the long-form `verbosity::` alias produces identical output to `v::` across all 3 commands.
**Setup:** At least one saved account exists under `~/.persistent/claude/credential/`. Active credentials exist at `~/.claude/.credentials.json`.
**Command:**
1. `clp .account.list verbosity::0` vs `clp .account.list v::0`
2. `clp .token.status verbosity::1` vs `clp .token.status v::1`
3. `clp .paths verbosity::2` vs `clp .paths v::2`
**Expected Output:** Each `verbosity::N` invocation produces output identical to the corresponding `v::N` invocation. All exit 0.
**Verification:**
- All 6 commands exit 0
- `.account.list verbosity::0` output matches `.account.list v::0` output exactly
- `.token.status verbosity::1` output matches `.token.status v::1` output exactly
- `.paths verbosity::2` output matches `.paths v::2` output exactly
**Pass Criteria:** Exit 0 for all; `verbosity::` is a perfect alias for `v::` across all Output Control commands.
**Source:** [parameter_groups.md -- Output Control](../../../../../docs/cli/parameter_groups.md#group--1-output-control)

---

### EC-1: Interaction — Quiet with JSON

**Goal:** Confirm that `v::0` with `format::json` still produces complete JSON with all keys (format overrides verbosity for JSON).
**Setup:** At least one saved account exists under `~/.persistent/claude/credential/`.
**Command:** `clp .account.list v::0 format::json`
**Expected Output:** Valid JSON array with all fields (`name`, `is_active`, `subscription_type`, etc.) present in each object. Exit 0.
**Verification:**
- Exit code is 0
- Output is valid JSON (parseable without error)
- JSON objects contain all standard keys (not stripped by quiet mode)
- Output matches `clp .account.list format::json` (without `v::0`)
**Pass Criteria:** Exit 0; `v::0` does not strip keys from JSON output — format overrides verbosity.
**Source:** [parameter_groups.md -- Output Control](../../../../../docs/cli/parameter_groups.md#group--1-output-control)

---

### EC-2: Interaction — Verbose with JSON

**Goal:** Confirm that `v::2` with `format::json` does not inject extra keys or diagnostic metadata into the JSON structure.
**Setup:** At least one saved account exists under `~/.persistent/claude/credential/`.
**Command:** `clp .account.list v::2 format::json`
**Expected Output:** Valid JSON array with the same keys as `format::json` alone. Exit 0.
**Verification:**
- Exit code is 0
- Output is valid JSON (parseable without error)
- JSON key set matches `clp .account.list format::json` exactly
- No extra keys (e.g., `debug`, `metadata`, `raw_timestamp`) added by verbose mode
**Pass Criteria:** Exit 0; `v::2` does not add extra JSON keys — format overrides verbosity.
**Source:** [parameter_groups.md -- Output Control](../../../../../docs/cli/parameter_groups.md#group--1-output-control)

---

### EC-3: Exit Code Preservation

**Goal:** Confirm that `v::0` (quiet mode) does not suppress error exit codes.
**Setup:** Remove or rename `~/.claude/.credentials.json` so that `.token.status` triggers a runtime error (exit 2).
**Command:** `clp .token.status v::0`
**Expected Output:** Error exit code (exit 2) for unreadable credentials, even though output is quiet.
**Verification:**
- Exit code is 2 (runtime error), not 0
- Stderr contains an error message about missing or unreadable credentials
- Quiet mode suppresses normal output labels but does not hide errors
**Pass Criteria:** Exit 2; quiet mode does not mask error conditions or change exit codes.
**Source:** [parameter_groups.md -- Output Control](../../../../../docs/cli/parameter_groups.md#group--1-output-control)

---

### EC-4: Non-Applicability

**Goal:** Confirm that Output Control parameters (`v::`, `format::`) are ignored or rejected by mutation commands that do not support them.
**Setup:** Active credentials exist at `~/.claude/.credentials.json`. An account named `test_na` exists.
**Command:**
1. `clp .account.save name::test_na v::0`
2. `clp .account.switch name::test_na format::json`
3. `clp .account.delete name::test_na v::2`
**Expected Output:** Each mutation command either ignores the Output Control parameter (producing its standard single-line confirmation) or rejects it with an error. The parameter does not alter the mutation command's output format.
**Verification:**
- Mutation commands produce their standard fixed-format confirmation messages
- `.account.save` output is `saved current credentials as 'test_na'` (not affected by `v::0`)
- `.account.switch` output is `switched to 'test_na'` (not JSON despite `format::json`)
- `.account.delete` output is `deleted account 'test_na'` (not verbose despite `v::2`)
- If parameters are rejected: exit 1 with an error about unsupported parameter
**Pass Criteria:** Output Control params have no effect on mutation commands; output remains fixed format.
**Source:** [parameter_groups.md -- Output Control](../../../../../docs/cli/parameter_groups.md#group--1-output-control)
