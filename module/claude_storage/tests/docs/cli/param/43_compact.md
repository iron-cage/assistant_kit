# Parameter :: `compact::`

Edge case tests for the `compact::` parameter. Tests validate `.tail`'s one-line-per-turn scan layout — row count, per-row ordinal and speaker, suppression of rule lines — and its precedence over `full::`.

**Source:** [param/43_compact.md](../../../../docs/cli/param/43_compact.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `compact::1` prints exactly one line per turn | Happy Path |
| EC-2 | `compact::1 full::1` → compact wins | Precedence |

## Test Coverage Summary

- Happy Path: 1 test (EC-1)
- Precedence: 1 test (EC-2)

**Total:** 2 edge cases

**Behavioral Divergence Pair:** EC-1 (`compact::1`, one row per turn, no rules) ↔ default layout (full bodies separated by rule lines)

## Test Cases

---

### EC-1: `compact::1` prints exactly one line per turn

- **Commands:** `.tail`
- **Given:** a session with three turns — user, assistant, user — each carrying a distinct marker
- **When:** `clg .tail compact::1`
- **Then:** exactly three marker-bearing rows are printed and zero rule lines are drawn; the first row carries turn ordinal `1` and speaker `You`; the second names `Claude`
- **Exit:** 0
- **Source:** [param/43_compact.md](../../../../docs/cli/param/43_compact.md)

---

### EC-2: `compact::1 full::1` → compact wins

- **Commands:** `.tail`
- **Given:** a two-turn session whose assistant turn carries a 20-line body — long enough that `full::` is observable in the default layout, which is what gives this case its teeth (EC-1's short fixture would leave `full::` inert either way)
- **When:** `clg .tail compact::1 full::1`
- **Then:** the rendered rows are identical to `compact::1` alone — one row per turn, no rule lines, no unfolded body; `full::` is inert because compact mode never prints the bodies it would unfold
- **Exit:** 0
- **Source:** [param/42_full.md](../../../../docs/cli/param/42_full.md)
