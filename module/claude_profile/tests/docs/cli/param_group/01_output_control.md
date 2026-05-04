# Test: Output Control Group

Integration and edge case coverage for the Output Control parameter group (`v::`, `format::`). See [parameter_groups.md](../../../../docs/cli/parameter_groups.md#group--1-output-control) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `v::0` suppresses labels across supported commands | Quiet Mode |
| EC-2 | `v::1` (default) produces labeled output across supported commands | Standard Mode |
| EC-3 | `v::2` includes full metadata across supported commands | Verbose Mode |
| EC-4 | `verbosity::` long-form alias is cross-command consistent with `v::` | Alias Consistency |
| EC-1 | `v::0` with `format::json` — quiet flag does not strip JSON keys | Interaction |
| EC-2 | `v::2` with `format::json` — verbose flag does not add extra JSON keys | Interaction |
| EC-3 | `v::0` exit code unchanged — quiet mode does not suppress errors | Exit Code Preservation |
| EC-4 | Output Control params ignored by mutation commands (save, switch, delete) | Non-Applicability |
| EC-1 | `.accounts` accepts `format::` but not `v::` — partial implementor | Partial Implementor |

### Test Coverage Summary

- Quiet Mode: 1 test
- Standard Mode: 1 test
- Verbose Mode: 1 test
- Alias Consistency: 1 test
- Interaction: 2 tests
- Exit Code Preservation: 1 test
- Non-Applicability: 1 test
- Partial Implementor: 1 test

**Total:** 9 tests (4 integration, 5 edge cases)

---

### EC-1: Quiet Mode

- **Given:** Active credentials exist at `~/.claude/.credentials.json`.
- **When:**
  1. `clp .token.status v::0`
  2. `clp .paths v::0`
  3. `clp .usage v::0`
  4. `clp .account.limits v::0`
- **Then:** 1. Bare status word (e.g., `valid`), no label prefix or remaining time detail.
2. Bare base path (e.g., `/home/user/.claude`), no path labels.
3. Bare total token count (e.g., `163.3K`), no period or model breakdown.
4. Bare utilization values, no labels or reset times.
All exit 0.; for all 4; `v::0` consistently suppresses labels across all verbosity-supporting commands
- **Exit:** 0
- **Source:** [parameter_groups.md -- Output Control](../../../../docs/cli/parameter_groups.md#group--1-output-control)

---

### EC-2: Standard Mode

- **Given:** Active credentials exist at `~/.claude/.credentials.json`.
- **When:**
  1. `clp .token.status v::1`
  2. `clp .paths v::1`
  3. `clp .usage v::1`
- **Then:** 1. Status with label and remaining time (e.g., `valid — 47m remaining`).
2. Labeled paths (e.g., `credentials: /home/user/.claude/.credentials.json`).
3. Labeled per-model usage with total line.
All exit 0.; for all; `v::1` consistently produces labeled output across supported commands
- **Exit:** 0
- **Source:** [parameter_groups.md -- Output Control](../../../../docs/cli/parameter_groups.md#group--1-output-control)

---

### EC-3: Verbose Mode

- **Given:** Active credentials exist at `~/.claude/.credentials.json`.
- **When:**
  1. `clp .token.status v::2`
  2. `clp .paths v::2`
  3. `clp .usage v::2`
- **Then:** 1. Status with detailed expiry information including raw seconds or full timestamp.
2. All canonical paths with labels and expanded detail.
3. Per-model summary with daily breakdown.
All exit 0.; for all; `v::2` consistently provides maximum metadata across supported commands
- **Exit:** 0
- **Source:** [parameter_groups.md -- Output Control](../../../../docs/cli/parameter_groups.md#group--1-output-control)

---

### EC-4: Alias Consistency

- **Given:** Active credentials exist at `~/.claude/.credentials.json`.
- **When:**
  1. `clp .token.status verbosity::0` vs `clp .token.status v::0`
  2. `clp .paths verbosity::1` vs `clp .paths v::1`
  3. `clp .usage verbosity::2` vs `clp .usage v::2`
- **Then:** Each `verbosity::N` invocation produces output identical to the corresponding `v::N` invocation. All exit 0.; for all; `verbosity::` is a perfect alias for `v::` across all Output Control commands
- **Exit:** 0
- **Source:** [parameter_groups.md -- Output Control](../../../../docs/cli/parameter_groups.md#group--1-output-control)

---

### EC-1: Interaction — Quiet with JSON

- **Given:** Active credentials exist at `~/.claude/.credentials.json`.
- **When:** `clp .token.status v::0 format::json`
- **Then:** Valid JSON object with all fields (`status`, `expires_in_secs`) present. Exit 0.; `v::0` does not strip keys from JSON output — format overrides verbosity
- **Exit:** 0
- **Source:** [parameter_groups.md -- Output Control](../../../../docs/cli/parameter_groups.md#group--1-output-control)

---

### EC-2: Interaction — Verbose with JSON

- **Given:** Active credentials exist at `~/.claude/.credentials.json`.
- **When:** `clp .token.status v::2 format::json`
- **Then:** Valid JSON object with the same keys as `format::json` alone. Exit 0.; `v::2` does not add extra JSON keys — format overrides verbosity
- **Exit:** 0
- **Source:** [parameter_groups.md -- Output Control](../../../../docs/cli/parameter_groups.md#group--1-output-control)

---

### EC-3: Exit Code Preservation

- **Given:** Remove or rename `~/.claude/.credentials.json` so that `.token.status` triggers a runtime error (exit 2).
- **When:** `clp .token.status v::0`
- **Then:** Error exit code (exit 2) for unreadable credentials, even though output is quiet.; quiet mode does not mask error conditions or change exit codes
- **Exit:** 2
- **Source:** [parameter_groups.md -- Output Control](../../../../docs/cli/parameter_groups.md#group--1-output-control)

---

### EC-4: Non-Applicability

- **Given:** Active credentials exist at `~/.claude/.credentials.json`. An account named `test_na@x.com` exists.
- **When:**
  1. `clp .account.save name::test_na@x.com v::0`
  2. `clp .account.switch name::test_na@x.com format::json`
  3. `clp .account.delete name::test_na@x.com v::2`
- **Then:** Each mutation command either ignores the Output Control parameter (producing its standard single-line confirmation) or rejects it with an error. The parameter does not alter the mutation command's output format.; Output Control params have no effect on mutation commands; output remains fixed format
- **Exit:** 0
- **Source:** [parameter_groups.md -- Output Control](../../../../docs/cli/parameter_groups.md#group--1-output-control)

---

### EC-1: Partial Implementor — `.accounts` accepts `format::` but not `v::`

- **Given:** At least one saved account exists.
- **When:**
  1. `clp .accounts format::json` — should succeed
  2. `clp .accounts v::0` — should fail (v:: not supported by .accounts)
- **Then:** 1. Valid JSON array with account objects. Exit 0.
2. Error indicating `v::` is an unknown parameter. Exit 1.; `.accounts` is a partial implementor — `format::` works; `v::` does not
- **Exit:** 0
- **Source:** [parameter_groups.md -- Output Control](../../../../docs/cli/parameter_groups.md#group--1-output-control) (Partial Implementors)
