# Test: `.processes.kill`

### Scope

- **Purpose**: Integration test cases for the `.processes.kill` command.
- **Responsibility**: Test factor analysis, case index, and expected behavior for process termination.
- **In Scope**: SIGTERM/SIGKILL sequence, force mode, dry-run, post-kill verification.
- **Out of Scope**: Parameter edge cases (→ `../param/`), group interactions (→ `../param_group/`).

Integration test planning for the `.processes.kill` command. See [command/readme.md](../../../../docs/cli/command/readme.md) for specification.

## Test Factor Analysis

### Factor 1: `dry::` (Boolean, optional, default 0)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default: real kill | Default behavior |
| 0 | Explicit: real kill | Explicit false |
| 1 | Preview only: no kill | Explicit true |
| 2 | Out-of-range boolean | Invalid: exit 1 |

### Factor 2: `force::` (Boolean, optional, default 0)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default: SIGTERM → wait → SIGKILL | Default behavior |
| 0 | Explicit: SIGTERM sequence | Explicit false |
| 1 | SIGKILL directly (no SIGTERM) | Explicit true |
| 2 | Out-of-range boolean | Invalid: exit 1 |

### Factor 3: Interaction: `dry::1` vs `force::1`

| Combination | Behavior |
|-------------|----------|
| `dry::1` alone | Preview: "no active processes" or "[dry-run] would kill N" |
| `force::1` alone | Real SIGKILL |
| `dry::1 force::1` | dry wins: preview only, no kill |

### Factor 4: Active processes (Environmental — /proc global state)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| none | No claude processes | No-op |
| one or more | Processes found | Kill sequence |

**Note:** Tests cannot control /proc state. All automated tests must handle both empty
and non-empty /proc results gracefully.

### Factor 5: Unknown parameters

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| none | No unknown params | Happy path |
| present | e.g. `bogus::x` | Invalid: exit 1 |

### Factor 6: `verbosity::` / `v::` (Integer, optional, default 1)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default: labeled output | Default behavior |
| 0 | Bare count / minimal output | Compact |
| 1 | Labeled message | Labeled |

### Factor 7: `format::` (String, optional, default "text")

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default: text output | Default behavior |
| `text` | Human-readable text | Valid |
| `json` | Machine-readable JSON | Valid |
| `JSON` | Wrong case | Invalid: exit 1 |

---

## Test Matrix

### Positive Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| IT-1 | No processes → "no active processes", exit 0 | P | 0 | F1=absent, F4=none | [mutation_processes_kill_test.rs] |
| IT-2 | `dry::1` no processes → "no active processes" | P | 0 | F1=1, F4=none | [mutation_processes_kill_test.rs] |
| IT-3 | `dry::1 force::1` no processes → "no active processes" | P | 0 | F1=1, F2=1, F3, F4=none | [mutation_processes_kill_test.rs] |
| IT-4 | `v::0` → accepted, exit 0 | P | 0 | F6=0 | [mutation_processes_kill_test.rs] |
| IT-6 | Source-level AF: `let _ = send_sig` absent from commands/process.rs | P | 0 | — | [mutation_processes_kill_test.rs] |
| IT-7 | `dry::1 format::json` → JSON object output, exit 0 | P | 0 | F1=1, F7=json | [mutation_processes_kill_test.rs] |

### Negative Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| IT-5 | `format::JSON` (uppercase) → exit 1 | N | 1 | F7=JSON | [mutation_processes_kill_test.rs] |
| IT-8 | `bogus::x` → exit 1 | N | 1 | F5=present | [mutation_processes_kill_test.rs] |
| IT-9 | `dry::2` → exit 1, out-of-range boolean | N | 1 | F1=2 | [mutation_processes_kill_test.rs] |
| IT-10 | `force::2` → exit 1, out-of-range boolean | N | 1 | F2=2 | [mutation_processes_kill_test.rs] |

### Summary

- **Total:** 10 tests (6 positive, 4 negative)
- **Negative ratio:** 40.0% ✅ (≥40%)
- **TC range:** IT-1 to IT-7, IT-8 to IT-10

---

## Coverage Verification

### Exit Status Coverage

| Exit Code | Meaning | Tests |
|-----------|---------|-------|
| 0 | Success (kill or no-op) | IT-1, IT-2, IT-3, IT-4, IT-6, IT-7 |
| 1 | Invalid arguments | IT-5, IT-8 through IT-10 |
| 2 | Kill verification failure (post-kill survivors) | Manual only (FR-09) |

### Kill Sequence Coverage (FR-09)

| Scenario | Coverage |
|----------|---------|
| No processes (no-op) | IT-1 |
| `dry::1` no processes | IT-2 |
| `dry::1 force::1` (dry wins) | IT-3 |
| Real SIGTERM sequence (with processes) | Manual (requires live processes) |
| `force::1` SIGKILL (with processes) | Manual (requires live processes) |

IT-1 through IT-3 cover the "no processes" path. Real kill sequences require
live claude processes and are manual-only tests.

---

## Test Case Details

---

### IT-1: No processes → "no active processes"

- **Given:** No claude processes in /proc (may not be guaranteed).
- **When:**
  `clv .processes.kill`
  **Expected:** Exit 0; stdout contains "no active processes" or similar.
- **Then:** exit 0; stdout contains "no active processes" message or kill completion summary; either outcome accepted due to /proc global state
- **Exit:** 0
- **Source:** [command/processes.md](../../../../docs/cli/command/processes.md)

---

### IT-2: `dry::1` no processes

- **Given:** clean environment
- **When:**
  `clv .processes.kill dry::1`
  **Expected:** Exit 0; appropriate message.
- **Then:** exit 0; stdout contains "[dry-run]" indicator or "no active processes" message; no kill executed; stderr is empty
- **Exit:** 0
- **Source:** [command/processes.md](../../../../docs/cli/command/processes.md)

---

### IT-3: `dry::1 force::1` → dry wins

- **Given:** clean environment
- **When:**
  `clv .processes.kill dry::1 force::1`
  **Expected:** Exit 0; no kill executed.
- **Then:** exit 0; stdout contains dry-run preview; no kill executed even though force::1 is present (dry wins); stderr is empty
- **Exit:** 0
- **Source:** [command/processes.md](../../../../docs/cli/command/processes.md)

---

### IT-4: `v::0` → accepted, exit 0

- **Given:** clean environment
- **When:**
  `clv .processes.kill v::0`
  **Expected:** Exit 0; output produced (either "no active processes" or kill summary).
- **Then:** exit 0; stdout contains output (either "no active processes" or kill summary); v::0 accepted as valid verbosity level (not rejected as unknown)
- **Exit:** 0
- **Source:** [command/processes.md](../../../../docs/cli/command/processes.md)

---

### IT-5: `format::JSON` (uppercase) → exit 1

- **Given:** clean environment
- **When:**
  `clv .processes.kill format::JSON`
  **Expected:** Exit 1.
- **Then:** exit 1; stderr contains error referencing case-sensitive format value or listing valid options; no kill executed
- **Exit:** 1
- **Source:** [command/processes.md](../../../../docs/cli/command/processes.md)

---

### IT-6: Source-level AF — `let _ = send_sig` absent

- **Given:** clean environment
- **When:**
  Code inspection via `std::fs::read_to_string`.
  **Expected:** `let _ = send_sigterm` and `let _ = send_sigkill` absent from `commands/process.rs`.
- **Then:** Both `let _` patterns absent.
**Note:** This is an anti-faking check for TSK-101. The signal-error path cannot be triggered through the binary without process injection; source inspection is the only reliable verification
- **Exit:** 0
- **Source:** [command/processes.md](../../../../docs/cli/command/processes.md)

---

### IT-7: `dry::1 format::json` → JSON object output

- **Given:** clean environment
- **When:**
  `clv .processes.kill dry::1 format::json`
  **Expected:** Exit 0; stdout starts with `{`.
- **Then:** exit 0; stdout is valid JSON starting with `{`; contains dry-run process information; stderr is empty
- **Exit:** 0
- **Source:** [command/processes.md](../../../../docs/cli/command/processes.md)

---

### IT-8: `bogus::x` → exit 1

- **Given:** clean environment
- **When:**
  `clv .processes.kill bogus::x`
  **Expected:** Exit 1.
- **Then:** exit 1; stderr or stdout contains "bogus" or "unknown parameter" error message; no kill executed
- **Exit:** 1
- **Source:** [command/processes.md](../../../../docs/cli/command/processes.md)

---

### IT-9: `dry::2` → exit 1

- **Given:** clean environment
- **When:**
  `clv .processes.kill dry::2`
  **Expected:** Exit 1.
- **Then:** exit 1; stderr or stdout references out-of-range boolean value "2" for dry::; no kill executed
- **Exit:** 1
- **Source:** [command/processes.md](../../../../docs/cli/command/processes.md)

---

### IT-10: `force::2` → exit 1

- **Given:** clean environment
- **When:**
  `clv .processes.kill force::2`
  **Expected:** Exit 1.
- **Then:** exit 1; stderr or stdout references out-of-range boolean value "2" for force::; no kill executed
- **Exit:** 1
- **Source:** [command/processes.md](../../../../docs/cli/command/processes.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc310_processes_kill_dry_exits_0` | `tests/cli/mutation_processes_kill_test.rs` |
| `tc311_processes_kill_dry_mentions_sigterm` | `tests/cli/mutation_processes_kill_test.rs` |
| `tc312_processes_kill_dry_force_mentions_sigkill` | `tests/cli/mutation_processes_kill_test.rs` |
| `tc313_processes_kill_v0_accepted` | `tests/cli/mutation_processes_kill_test.rs` |
| `tc314_processes_kill_format_uppercase_rejected` | `tests/cli/mutation_processes_kill_test.rs` |
| `tc315_processes_kill_no_let_underscore_on_send_sig` | `tests/cli/mutation_processes_kill_test.rs` |
| `tc316_processes_kill_dry_format_json` | `tests/cli/mutation_processes_kill_test.rs` |
| `tc251_processes_kill_dry_force_dry_wins` | `tests/cli/cross_cutting_test.rs` |
