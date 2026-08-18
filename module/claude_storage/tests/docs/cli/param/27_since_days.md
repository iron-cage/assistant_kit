# Parameter :: `since_days::`

Edge case tests for the `since_days::` parameter. Tests validate recency-window filtering, the zero-day boundary, negative and non-integer rejection, and the omitted-default regression.

**Source:** [param/27_since_days.md](../../../../docs/cli/param/27_since_days.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Window includes recent session, excludes old session | Filter Behavior |
| EC-2 | `since_days::0` shows a session touched today | Boundary |
| EC-3 | Negative value rejected | Validation |
| EC-4 | Omitted means no window filtering | Default |
| EC-5 | Non-integer value rejected | Type Validation |

## Test Coverage Summary

- Filter Behavior: 1 test (EC-1)
- Boundary: 1 test (EC-2)
- Validation: 1 test (EC-3)
- Default: 1 test (EC-4)
- Type Validation: 1 test (EC-5)

**Total:** 5 edge cases

**Behavioral Divergence Pair:** EC-1 (`since_days::20`, old session dropped) ↔ EC-4 (omitted, old session listed)

## Test Cases

---

### EC-1: Window includes recent session, excludes old session

- **Commands:** `.projects`
- **Given:** Two sessions in one project with mtimes `now - 5d` and `now - 25d` (set via `FileTimes::set_modified`)
- **When:** `clg .projects scope::global since_days::20`
- **Then:** The 5-day-old session is listed; the 25-day-old session is absent; the header counts only the windowed session
- **Exit:** 0
- **Source:** [param/27_since_days.md](../../../../docs/cli/param/27_since_days.md)

---

### EC-2: `since_days::0` shows a session touched today

- **Commands:** `.projects`
- **Given:** A freshly written session (mtime = now)
- **When:** `clg .projects scope::global since_days::0`
- **Then:** The session is listed — `0` means the most recent 24 hours, never an empty window
- **Exit:** 0
- **Source:** [param/27_since_days.md](../../../../docs/cli/param/27_since_days.md)

---

### EC-3: Negative value rejected

- **Commands:** `.projects`
- **Given:** clean environment
- **When:** `clg .projects since_days::-1`
- **Then:** Error output mentions `since_days` (`Invalid since_days: -1. Must be non-negative`)
- **Exit:** 1
- **Source:** [param/27_since_days.md](../../../../docs/cli/param/27_since_days.md)

---

### EC-4: Omitted means no window filtering

- **Commands:** `.projects`
- **Given:** Same two-session fixture as EC-1 (mtimes `now - 5d` and `now - 25d`)
- **When:** `clg .projects scope::global`
- **Then:** Both sessions are listed; the header counts both — the parameter is purely additive
- **Exit:** 0
- **Source:** [param/27_since_days.md](../../../../docs/cli/param/27_since_days.md)

---

### EC-5: Non-integer value rejected

- **Commands:** `.projects`
- **Given:** clean environment
- **When:** `clg .projects since_days::abc`
- **Then:** Coercion error on the `since_days` argument (cannot coerce to Integer)
- **Exit:** non-zero
- **Source:** [param/27_since_days.md](../../../../docs/cli/param/27_since_days.md)
