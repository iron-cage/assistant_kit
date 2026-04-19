# Test: `.processes.kill`

Integration test planning for the `.processes.kill` command. See [commands.md](../../../../../docs/cli/commands.md) for specification.

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
| TC-310 | No processes → "no active processes", exit 0 | P | 0 | F1=absent, F4=none | [mutation_commands_test.rs] |
| TC-311 | `dry::1` no processes → "no active processes" | P | 0 | F1=1, F4=none | [mutation_commands_test.rs] |
| TC-312 | `dry::1 force::1` no processes → "no active processes" | P | 0 | F1=1, F2=1, F3, F4=none | [mutation_commands_test.rs] |
| TC-313 | `v::0` → accepted, exit 0 | P | 0 | F6=0 | [mutation_commands_test.rs] |
| TC-315 | Source-level AF: `let _ = send_sig` absent from commands.rs | P | 0 | — | [mutation_commands_test.rs] |
| TC-316 | `dry::1 format::json` → JSON object output, exit 0 | P | 0 | F1=1, F7=json | [mutation_commands_test.rs] |

### Negative Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| TC-314 | `format::JSON` (uppercase) → exit 1 | N | 1 | F7=JSON | [mutation_commands_test.rs] |
| TC-465 | `bogus::x` → exit 1 | N | 1 | F5=present | [mutation_commands_test.rs] |
| TC-466 | `dry::2` → exit 1, out-of-range boolean | N | 1 | F1=2 | [mutation_commands_test.rs] |
| TC-467 | `force::2` → exit 1, out-of-range boolean | N | 1 | F2=2 | [mutation_commands_test.rs] |

### Summary

- **Total:** 10 tests (6 positive, 4 negative)
- **Negative ratio:** 40.0% ✅ (≥40%)
- **TC range:** TC-310 to TC-316, TC-465 to TC-467

---

## Coverage Verification

### Exit Status Coverage

| Exit Code | Meaning | Tests |
|-----------|---------|-------|
| 0 | Success (kill or no-op) | TC-310, TC-311, TC-312, TC-313, TC-315, TC-316 |
| 1 | Invalid arguments | TC-314, TC-465 through TC-467 |
| 2 | Kill verification failure (post-kill survivors) | Manual only (FR-09) |

### Kill Sequence Coverage (FR-09)

| Scenario | Coverage |
|----------|---------|
| No processes (no-op) | TC-310 |
| `dry::1` no processes | TC-311 |
| `dry::1 force::1` (dry wins) | TC-312 |
| Real SIGTERM sequence (with processes) | Manual (requires live processes) |
| `force::1` SIGKILL (with processes) | Manual (requires live processes) |

TC-310 through TC-312 cover the "no processes" path. Real kill sequences require
live claude processes and are manual-only tests.

---

## Test Case Details

### TC-310: No processes → "no active processes"

**Goal:** When no claude processes exist, command reports no-op and exits 0.
**Setup:** No claude processes in /proc (may not be guaranteed).
**Command:** `cm .processes.kill`
**Expected:** Exit 0; stdout contains "no active processes" or similar.
**Verification:** exit code 0.
**Pass Criteria:** Exit 0.
**Note:** TC must accept both "no processes" and "processes killed" outcomes due to global /proc.

---

### TC-311: `dry::1` no processes

**Goal:** Dry-run with no processes shows no-op message without side effects.
**Setup:** None.
**Command:** `cm .processes.kill dry::1`
**Expected:** Exit 0; appropriate message.
**Verification:** exit code 0.
**Pass Criteria:** Exit 0.

---

### TC-312: `dry::1 force::1` → dry wins

**Goal:** `dry::` takes precedence over `force::`.
**Setup:** None.
**Command:** `cm .processes.kill dry::1 force::1`
**Expected:** Exit 0; no kill executed.
**Verification:** exit code 0.
**Pass Criteria:** Exit 0.

---

### TC-313: `v::0` → accepted, exit 0

**Goal:** Verbosity 0 is accepted by `.processes.kill` after TSK-099 output-control wiring.
**Setup:** None.
**Command:** `cm .processes.kill v::0`
**Expected:** Exit 0; output produced (either "no active processes" or kill summary).
**Verification:** exit code 0.
**Pass Criteria:** Exit 0 (not exit 1 "unknown parameter").

---

### TC-314: `format::JSON` (uppercase) → exit 1

**Goal:** Format parameter is case-sensitive; uppercase variant is rejected.
**Setup:** None.
**Command:** `cm .processes.kill format::JSON`
**Expected:** Exit 1.
**Verification:** exit code 1.
**Pass Criteria:** Exit 1.

---

### TC-315: Source-level AF — `let _ = send_sig` absent

**Goal:** Verify signal errors are no longer silently swallowed via `let _` pattern.
**Setup:** None (reads source file at compile-time via `CARGO_MANIFEST_DIR`).
**Command:** Code inspection via `std::fs::read_to_string`.
**Expected:** `let _ = send_sigterm` and `let _ = send_sigkill` absent from `commands.rs`.
**Verification:** string absence assertions.
**Pass Criteria:** Both `let _` patterns absent.
**Note:** This is an anti-faking check for TSK-101. The signal-error path cannot be triggered through the binary without process injection; source inspection is the only reliable verification.

---

### TC-316: `dry::1 format::json` → JSON object output

**Goal:** `format::json` produces machine-readable JSON from `.processes.kill`.
**Setup:** None.
**Command:** `cm .processes.kill dry::1 format::json`
**Expected:** Exit 0; stdout starts with `{`.
**Verification:** `text.trim_start().starts_with('{')`.
**Pass Criteria:** Exit 0; JSON object output.

---

### TC-465: `bogus::x` → exit 1

**Goal:** Unknown parameter rejected before scan.
**Setup:** None.
**Command:** `cm .processes.kill bogus::x`
**Expected:** Exit 1.
**Verification:** exit code 1.
**Pass Criteria:** Exit 1.

---

### TC-466: `dry::2` → exit 1

**Goal:** Boolean parameter only accepts 0 or 1.
**Setup:** None.
**Command:** `cm .processes.kill dry::2`
**Expected:** Exit 1.
**Verification:** exit code 1.
**Pass Criteria:** Exit 1.

---

### TC-467: `force::2` → exit 1

**Goal:** Boolean parameter only accepts 0 or 1.
**Setup:** None.
**Command:** `cm .processes.kill force::2`
**Expected:** Exit 1.
**Verification:** exit code 1.
**Pass Criteria:** Exit 1.
