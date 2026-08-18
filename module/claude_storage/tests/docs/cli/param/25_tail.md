# Parameter :: `tail::`

Edge case tests for the `tail::` parameter. Tests validate integer enforcement, entry-count capping, and per-command default behavior (`4` on `.tail`, `10` on `.show`'s project overview).

**Source:** [param/25_tail.md](../../../../docs/cli/param/25_tail.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | No `tail::` given → last 4 entries shown | Default |
| EC-2 | `tail::2` → exactly 2 entries shown | Happy Path |
| EC-3 | `tail::0` → all entries shown | Boundary Values |
| EC-4 | Negative tail (e.g., `tail::-1`) → rejected | Boundary Values |
| EC-5 | `tail::` empty value → rejected | Boundary Values |
| EC-6 | `tail::100` when session has fewer entries → all shown | Boundary Values |
| EC-7 | `tail::` non-integer value → rejected | Type Validation |
| EC-8 | No `tail::` given on `.show` → last 10 entries shown | Cross-Command Default |
| EC-9 | `tail::0` on `.show` → all entries from most-recently-active session shown | Cross-Command Default |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Happy Path: 1 test (EC-2)
- Boundary Values: 4 tests (EC-3, EC-4, EC-5, EC-6)
- Type Validation: 1 test (EC-7)
- Cross-Command Default: 2 tests (EC-8, EC-9)

**Total:** 9 edge cases

Validation edge cases (EC-4, EC-5, EC-7) are parameter-level and shared across consuming commands — not re-tested per command. Only the default-value divergence (EC-1 vs EC-8) and its uncapped boundary (EC-3 vs EC-9) are command-specific and re-verified for `.show`.

**Behavioral Divergence Pair:** EC-2 (tail::2, capped) ↔ EC-3 (tail::0, uncapped)

**Behavioral Divergence Pair:** EC-1 (`.tail` default 4) ↔ EC-8 (`.show` default 10)

## Test Cases

---

### EC-1: No `tail::` given → last 4 entries shown

- **Commands:** `.tail`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session with 6 entries)
- **When:** `clg .tail`
- **Then:** Exactly the last 4 entries shown, oldest-first
- **Exit:** 0
- **Source:** [param/25_tail.md](../../../../docs/cli/param/25_tail.md)

---

### EC-2: `tail::2` → exactly 2 entries shown

- **Commands:** `.tail`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session with 6 entries)
- **When:** `clg .tail tail::2`
- **Then:** Exactly the last 2 entries shown; earlier entries omitted
- **Exit:** 0
- **Source:** [param/25_tail.md](../../../../docs/cli/param/25_tail.md)

---

### EC-3: `tail::0` → all entries shown (no cap)

- **Commands:** `.tail`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session with 6 entries)
- **When:** `clg .tail tail::0`
- **Then:** All 6 entries shown; no capping applied
- **Exit:** 0
- **Source:** [param/25_tail.md](../../../../docs/cli/param/25_tail.md)

---

### EC-4: Negative tail rejected

- **Commands:** `.tail`
- **Given:** clean environment
- **When:** `clg .tail tail::-1`
- **Then:** Exit 1; error indicating `tail` must be a non-negative integer
- **Exit:** 1
- **Source:** [param/25_tail.md](../../../../docs/cli/param/25_tail.md)

---

### EC-5: Empty value rejected

- **Commands:** `.tail`
- **Given:** clean environment
- **When:** `clg .tail tail::`
- **Then:** Exit 1; error indicating `tail` requires a value
- **Exit:** 1
- **Source:** [param/25_tail.md](../../../../docs/cli/param/25_tail.md)

---

### EC-6: `tail::100` when session has fewer entries → all shown

- **Commands:** `.tail`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session with 3 entries)
- **When:** `clg .tail tail::100`
- **Then:** All 3 entries shown (limit not reached); no error
- **Exit:** 0
- **Source:** [param/25_tail.md](../../../../docs/cli/param/25_tail.md)

---

### EC-7: Non-integer value rejected

- **Commands:** `.tail`
- **Given:** clean environment
- **When:** `clg .tail tail::four`
- **Then:** Exit 1; error indicating `tail` requires a non-negative integer
- **Exit:** 1
- **Source:** [param/25_tail.md](../../../../docs/cli/param/25_tail.md)

---

### EC-8: No `tail::` given on `.show` → last 10 entries shown

- **Commands:** `.show`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project whose most-recently-active session has 15 entries)
- **When:** `clg .show`
- **Then:** Exactly the last 10 messages shown from the most-recently-active session — `.show`'s own default, distinct from `.tail`'s default of 4 (see EC-1)
- **Exit:** 0
- **Source:** [param/25_tail.md](../../../../docs/cli/param/25_tail.md)

---

### EC-9: `tail::0` on `.show` → all entries from most-recently-active session shown

- **Commands:** `.show`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project whose most-recently-active session has 15 entries)
- **When:** `clg .show tail::0`
- **Then:** All 15 messages shown; no capping applied
- **Exit:** 0
- **Source:** [param/25_tail.md](../../../../docs/cli/param/25_tail.md)
