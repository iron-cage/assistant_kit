# Parameter :: `full::`

Edge case tests for the `full::` parameter. Tests validate that `.tail`'s per-turn body cap folds long turns by default and that `full::1` lifts it, plus the precedence rule against `compact::`.

**Source:** [param/42_full.md](../../../../docs/cli/param/42_full.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Long turn folds by default; `full::1` unfolds it | Happy Path |
| EC-2 | `compact::1 full::1` → compact wins, `full::` has no effect | Precedence |

## Test Coverage Summary

- Happy Path: 1 test (EC-1)
- Precedence: 1 test (EC-2)

**Total:** 2 edge cases

**Behavioral Divergence Pair:** EC-1 (default, folded) ↔ EC-1's `full::1` half (uncapped)

## Test Cases

---

### EC-1: Long turn folds by default; `full::1` unfolds it

- **Commands:** `.tail`
- **Given:** a session whose single assistant turn has a 20-line body
- **When:** `clg .tail` then `clg .tail full::1` over the same fixture
- **Then:** the default run prints body lines 1-8, omits line 9 onward, and emits `⋯ 12 more lines` followed by a working `.show session_id::… index::…` invocation; the `full::1` run prints all 20 lines and contains no `⋯` hint
- **Exit:** 0 for both
- **Source:** [param/42_full.md](../../../../docs/cli/param/42_full.md)

---

### EC-2: `compact::1 full::1` → compact wins

- **Commands:** `.tail`
- **Given:** the same multi-turn fixture used by `compact::`'s own EC-1
- **When:** `clg .tail compact::1 full::1`
- **Then:** output is one line per turn — identical to `compact::1` alone; `full::1` cannot re-expand bodies that compact mode never prints
- **Exit:** 0
- **Source:** [param/43_compact.md](../../../../docs/cli/param/43_compact.md)
