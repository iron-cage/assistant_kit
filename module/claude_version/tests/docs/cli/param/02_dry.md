# Test: `dry::`

Edge case coverage for the `dry::` parameter. See [params.md](../../../../docs/cli/params.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `dry::1` → `[dry-run]` prefix on `.version.install` | Explicit True |
| TC-311 | `dry::1` on `.processes.kill` → no kill | Explicit True |
| TC-330 | `dry::1` on `.settings.set` → no file change | Explicit True |
| EC-2 | `dry::1` wins over `force::1` | Interaction |
| TC-312 | `dry::1 force::1` on `.processes.kill` → dry wins | Interaction |
| EC-3 | `dry::1` does NOT write preference keys | Side-Effect Guard |
| EC-1 | Default (absent) resolves to `dry::0` (real action) | Default Behavior |
| EC-2 | `dry::0` explicit → same as absent | Explicit False |
| EC-3 | `dry::2` → exit 1, out of range | Invalid Value |
| EC-1 | `dry::-1` → exit 1, out of range | Invalid Value |
| EC-3 | `dry::abc` → exit 1, non-integer | Format Violation |
| EC-1 | `dry::` (empty) → exit 1 | Empty Value |
| EC-3 | `dry::` only accepted by mutation commands | Command Scope |

## Test Coverage Summary

- Explicit True: 3 tests
- Interaction (dry wins over force): 2 tests
- Side-Effect Guard: 1 test
- Default Behavior: 1 test
- Explicit False: 1 test
- Invalid Value: 2 tests
- Format Violation: 1 test
- Empty Value: 1 test
- Command Scope: 1 test

**Total:** 13 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

---

### EC-1: `dry::1` → `[dry-run]` prefix

- **Given:** clean environment
- **When:** `cm .version.install dry::1`
- **Then:** output contains `[dry-run]`; exit code 0.; dry-run marker present
- **Exit:** 0
- **Source:** [feature/004_dry_run.md](../../../../docs/feature/004_dry_run.md)

---

### EC-2: `dry::1` wins over `force::1`

- **Given:** clean environment
- **When:** `cm .version.install dry::1 force::1`
- **Then:** output contains `[dry-run]`; no install.; preview mode only
- **Exit:** 0
- **Source:** [parameter_interactions.md — dry+force precedence](../../../../docs/cli/parameter_interactions.md)

---

### EC-3: `dry::1` does NOT write preference keys

- **Given:** `HOME=<tmp>`; settings file empty.
- **When:** `cm .version.install dry::1 version::stable`
- **Then:** `settings.json` has no `preferredVersionSpec` key after command.; no settings written
- **Exit:** 0
- **Source:** [feature/004_dry_run.md](../../../../docs/feature/004_dry_run.md)

---

### EC-1: Default (absent) → `dry::0`

- **Given:** clean environment
- **When:** `cm .version.install dry::1` (compare to absent).
- **Then:** Behavior identical to explicit `dry::0`.; Default and explicit 0 produce identical behavior
- **Exit:** 0
- **Source:** [params.md — dry:: default: 0](../../../../docs/cli/params.md)

---

### EC-3: `dry::2` → exit 1

- **Given:** clean environment
- **When:** `cm .version.install dry::2`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [params.md — dry:: type: Boolean (0/1)](../../../../docs/cli/params.md)

---

### EC-1: `dry::-1` → exit 1

- **Given:** clean environment
- **When:** `cm .version.install dry::-1`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [params.md — dry:: type: Boolean (0/1)](../../../../docs/cli/params.md)

---

### EC-3: `dry::abc` → exit 1

- **Given:** clean environment
- **When:** `cm .version.install dry::abc`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [params.md — dry:: type: Boolean (0/1)](../../../../docs/cli/params.md)

---

### EC-1: `dry::` (empty) → exit 1

- **Given:** clean environment
- **When:** `cm .version.install dry::`
- **Then:** exit code 1; error about dry:: requiring a value.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-3: `dry::` only for mutation commands

- **Given:** clean environment
- **When:** `cm .version.list dry::1`
- **Then:** exit code 1; "unknown parameter" or similar.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)
