# Feature Test: Process Lifecycle

### Scope

- **Purpose**: FT- test cases for /proc scanning, SIGTERM/SIGKILL sequence, and post-kill verification.
- **Responsibility**: Acceptance criteria verifying no-process behavior, dry-run output, force mode, and dry+force precedence.
- **In Scope**: `.processes`, `.processes.kill`, no-process output, `dry::1`, `force::1`, `dry::1 force::1` precedence.
- **Out of Scope**: Signal delivery internals (non-observable in subprocess tests), `/proc` scanner unit behavior (-> source tests).

Feature test surface for process lifecycle. See [feature/002_process_lifecycle.md](../../../docs/feature/002_process_lifecycle.md) for specification.

## Behavioral Divergence Pair

Two valid `.processes.kill` invocations produce distinct output when no processes are present:

- **Input A:** `clv .processes.kill` → stdout contains `"no active processes"` (normal mode, no dry prefix)
- **Input B:** `clv .processes.kill dry::1` → stdout contains `"[dry-run]"` prefix (dry mode preview)

Both are valid invocations; the output prefix differs.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FT-1 | No processes → "no active processes", exit 0 | No-Process State |
| FT-2 | `dry::1` no processes → `[dry-run]` prefix in output | Dry-Run Output |
| FT-3 | `dry::1 force::1` → dry wins, `[dry-run]` prefix present | Dry+Force Precedence |
| FT-4 | `force::1` no processes → "no active processes", exit 0 | Force Mode |

## Test Coverage Summary

- No-Process State: 1 test (FT-1)
- Dry-Run Output: 1 test (FT-2)
- Dry+Force Precedence: 1 test (FT-3)
- Force Mode: 1 test (FT-4)

**Total:** 4 tests

---

### FT-1: No processes → "no active processes", exit 0

- **Given:** environment with `PATH=""` (no `claude` binary visible to subprocess scan)
- **When:** `clv .processes.kill`
- **Then:** stdout contains `"no active processes"`; exit 0
- **Exit:** 0
- **Source:** [feature/002_process_lifecycle.md — Process detection](../../../docs/feature/002_process_lifecycle.md)

---

### FT-2: `dry::1` no processes → `[dry-run]` prefix in output

- **Given:** environment with no detectable claude processes
- **When:** `clv .processes.kill dry::1`
- **Then:** stdout contains `"[dry-run]"`; exit 0
- **Exit:** 0
- **Source:** [feature/002_process_lifecycle.md — Dry-run](../../../docs/feature/002_process_lifecycle.md)

---

### FT-3: `dry::1 force::1` → dry wins, `[dry-run]` prefix present

- **Given:** environment with no detectable claude processes
- **When:** `clv .processes.kill dry::1 force::1`
- **Then:** stdout contains `"[dry-run]"`; dry-run takes precedence over force; exit 0
- **Exit:** 0
- **Source:** [feature/002_process_lifecycle.md](../../../docs/feature/002_process_lifecycle.md), [feature/004_dry_run.md — Precedence](../../../docs/feature/004_dry_run.md)

---

### FT-4: `force::1` no processes → "no active processes", exit 0

- **Given:** environment with no detectable claude processes
- **When:** `clv .processes.kill force::1`
- **Then:** stdout contains `"no active processes"`; exit 0
- **Exit:** 0
- **Source:** [feature/002_process_lifecycle.md — Kill sequence force mode](../../../docs/feature/002_process_lifecycle.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc310_processes_kill_dry_exits_0` | `tests/cli/mutation_processes_kill_test.rs` |
| `tc311_processes_kill_dry_mentions_sigterm` | `tests/cli/mutation_processes_kill_test.rs` |
| `tc312_processes_kill_dry_force_mentions_sigkill` | `tests/cli/mutation_processes_kill_test.rs` |
| `ft004_processes_kill_force_no_procs` | `tests/cli/feature_surface_test.rs` |
