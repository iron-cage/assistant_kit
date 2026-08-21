# Parameter :: `last::`

Edge case tests for the `last::` parameter. Tests validate integer enforcement and window capping on `.tail`.

**Unit note:** on `.tail`, `last::` caps **turns**, not raw entries (`docs/cli/command/12_tail.md § Turn Grouping`). These fixtures are built with one entry per turn, so the two coincide here and "entries" below reads correctly; the turn-vs-entry distinction itself is covered by `command/12_tail.md` INT-12 and INT-15.

**Source:** [param/25_last.md](../../../../docs/cli/param/25_last.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | No `last::` given → last 4 entries shown | Default |
| EC-2 | `last::2` → exactly 2 entries shown | Happy Path |
| EC-3 | `last::0` → all entries shown | Boundary Values |
| EC-4 | Negative `last::` (e.g., `last::-1`) → rejected | Boundary Values |
| EC-5 | `last::` empty value → rejected | Boundary Values |
| EC-6 | `last::100` when session has fewer entries → all shown | Boundary Values |
| EC-7 | `last::` non-integer value → rejected | Type Validation |
| EC-8 | `l::N` alias behaves identically to `last::N` | Happy Path |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Happy Path: 2 tests (EC-2, EC-8)
- Boundary Values: 4 tests (EC-3, EC-4, EC-5, EC-6)
- Type Validation: 1 test (EC-7)

**Total:** 8 edge cases

**Behavioral Divergence Pair:** EC-2 (last::2, capped) ↔ EC-3 (last::0, uncapped)

## Test Cases

---

### EC-1: No `last::` given → last 4 entries shown

- **Commands:** `.tail`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session with 6 entries)
- **When:** `clg .tail`
- **Then:** Exactly the last 4 entries shown, oldest-first
- **Exit:** 0
- **Source:** [param/25_last.md](../../../../docs/cli/param/25_last.md)

---

### EC-2: `last::2` → exactly 2 entries shown

- **Commands:** `.tail`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session with 6 entries)
- **When:** `clg .tail last::2`
- **Then:** Exactly the last 2 entries shown; earlier entries omitted
- **Exit:** 0
- **Source:** [param/25_last.md](../../../../docs/cli/param/25_last.md)

---

### EC-3: `last::0` → all entries shown (no cap)

- **Commands:** `.tail`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session with 6 entries)
- **When:** `clg .tail last::0`
- **Then:** All 6 entries shown; no capping applied
- **Exit:** 0
- **Source:** [param/25_last.md](../../../../docs/cli/param/25_last.md)

---

### EC-4: Negative `last::` rejected

- **Commands:** `.tail`
- **Given:** clean environment
- **When:** `clg .tail last::-1`
- **Then:** Exit 1; error indicating `last` must be a non-negative integer
- **Exit:** 1
- **Source:** [param/25_last.md](../../../../docs/cli/param/25_last.md)

---

### EC-5: Empty value rejected

- **Commands:** `.tail`
- **Given:** clean environment
- **When:** `clg .tail last::`
- **Then:** Exit 1; error indicating `last` requires a value
- **Exit:** 1
- **Source:** [param/25_last.md](../../../../docs/cli/param/25_last.md)

---

### EC-6: `last::100` when session has fewer entries → all shown

- **Commands:** `.tail`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session with 3 entries)
- **When:** `clg .tail last::100`
- **Then:** All 3 entries shown (limit not reached); no error
- **Exit:** 0
- **Source:** [param/25_last.md](../../../../docs/cli/param/25_last.md)

---

### EC-7: Non-integer value rejected

- **Commands:** `.tail`
- **Given:** clean environment
- **When:** `clg .tail last::four`
- **Then:** Exit 1; error indicating `last` requires a non-negative integer
- **Exit:** 1
- **Source:** [param/25_last.md](../../../../docs/cli/param/25_last.md)

---

### EC-8: `l::N` alias behaves identically to `last::N`

- **Commands:** `.tail`, `.show`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session with 6 entries)
- **When:** `clg .tail l::2` and `clg .tail last::2` are both run against the same fixture
- **Then:** Byte-identical stdout from both invocations — the alias binds to the same canonical `last` argument, so the routine cannot distinguish which spelling was used
- **Exit:** 0
- **Source:** [param/25_last.md](../../../../docs/cli/param/25_last.md)
