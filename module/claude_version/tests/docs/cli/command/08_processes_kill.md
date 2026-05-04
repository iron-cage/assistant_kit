# Test: `.processes.kill`

Integration test planning for the `.processes.kill` command. See [commands.md](../../../../docs/cli/commands.md) for specification.

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
| IT-1 | No processes → "no active processes", exit 0 | P | 0 | F1=absent, F4=none | [mutation_commands_test.rs] |
| IT-2 | `dry::1` no processes → "no active processes" | P | 0 | F1=1, F4=none | [mutation_commands_test.rs] |
| IT-3 | `dry::1 force::1` no processes → "no active processes" | P | 0 | F1=1, F2=1, F3, F4=none | [mutation_commands_test.rs] |
| IT-4 | `v::0` → accepted, exit 0 | P | 0 | F6=0 | [mutation_commands_test.rs] |
| IT-6 | Source-level AF: `let _ = send_sig` absent from commands.rs | P | 0 | — | [mutation_commands_test.rs] |
| IT-7 | `dry::1 format::json` → JSON object output, exit 0 | P | 0 | F1=1, F7=json | [mutation_commands_test.rs] |

### Negative Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| IT-5 | `format::JSON` (uppercase) → exit 1 | N | 1 | F7=JSON | [mutation_commands_test.rs] |
| IT-8 | `bogus::x` → exit 1 | N | 1 | F5=present | [mutation_commands_test.rs] |
| IT-9 | `dry::2` → exit 1, out-of-range boolean | N | 1 | F1=2 | [mutation_commands_test.rs] |
| IT-10 | `force::2` → exit 1, out-of-range boolean | N | 1 | F2=2 | [mutation_commands_test.rs] |

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
  `cm .processes.kill`
  **Expected:** Exit 0; stdout contains "no active processes" or similar.
- **Then:** **Note:** TC must accept both "no processes" and "processes killed" outcomes due to global /proc
- **Exit:** 0

---

### IT-2: `dry::1` no processes

- **Given:** clean environment
- **When:**
  `cm .processes.kill dry::1`
  **Expected:** Exit 0; appropriate message.
- **Then:** see spec
- **Exit:** 0

---

### IT-3: `dry::1 force::1` → dry wins

- **Given:** clean environment
- **When:**
  `cm .processes.kill dry::1 force::1`
  **Expected:** Exit 0; no kill executed.
- **Then:** see spec
- **Exit:** 0

---

### IT-4: `v::0` → accepted, exit 0

- **Given:** clean environment
- **When:**
  `cm .processes.kill v::0`
  **Expected:** Exit 0; output produced (either "no active processes" or kill summary).
- **Then:** (not exit 1 "unknown parameter")
- **Exit:** 0

---

### IT-5: `format::JSON` (uppercase) → exit 1

- **Given:** clean environment
- **When:**
  `cm .processes.kill format::JSON`
  **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-6: Source-level AF — `let _ = send_sig` absent

- **Given:** clean environment
- **When:**
  Code inspection via `std::fs::read_to_string`.
  **Expected:** `let _ = send_sigterm` and `let _ = send_sigkill` absent from `commands.rs`.
- **Then:** Both `let _` patterns absent.
**Note:** This is an anti-faking check for TSK-101. The signal-error path cannot be triggered through the binary without process injection; source inspection is the only reliable verification
- **Exit:** 0

---

### IT-7: `dry::1 format::json` → JSON object output

- **Given:** clean environment
- **When:**
  `cm .processes.kill dry::1 format::json`
  **Expected:** Exit 0; stdout starts with `{`.
- **Then:** JSON object output
- **Exit:** 0

---

### IT-8: `bogus::x` → exit 1

- **Given:** clean environment
- **When:**
  `cm .processes.kill bogus::x`
  **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-9: `dry::2` → exit 1

- **Given:** clean environment
- **When:**
  `cm .processes.kill dry::2`
  **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-10: `force::2` → exit 1

- **Given:** clean environment
- **When:**
  `cm .processes.kill force::2`
  **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1
