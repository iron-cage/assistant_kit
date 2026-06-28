# Parameter :: `strategy::`

Edge case tests for the `strategy::` parameter. Tests validate enum values, case insensitivity, forcing behavior, and default (auto-detect) behavior.

**Source:** [param/20_strategy.md](../../../../docs/cli/param/20_strategy.md) | [type/13_strategy_type.md](../../../../docs/cli/type/13_strategy_type.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | resume accepted | Type Validation |
| EC-2 | fresh accepted | Type Validation |
| EC-3 | Invalid value rejected | Boundary Values |
| EC-4 | Case-insensitive: Resume accepted | Type Validation |
| EC-5 | Case-insensitive: FRESH accepted | Type Validation |
| EC-6 | Absent defaults to auto-detect (fresh when no history) | Default |
| EC-7 | Absent defaults to auto-detect (resume when history exists) | Default |
| EC-8 | resume forced overrides auto-detect fresh | Override |
| EC-9 | fresh forced overrides auto-detect resume | Override |

## Test Coverage Summary

- Type Validation: 4 tests (EC-1, EC-2, EC-4, EC-5)
- Boundary Values: 1 test (EC-3)
- Default: 2 tests (EC-6, EC-7)
- Override: 2 tests (EC-8, EC-9)

**Total:** 9 edge cases

**Behavioral Divergence Pair:** EC-1 (strategy::resume) ↔ EC-2 (strategy::fresh)

## Test Cases

---

### EC-1: resume accepted

- **Commands:** `.session.ensure`
- **Given:** TempDir as HOME; base directory accessible.
- **When:** `clg .session.ensure path::{base} strategy::resume`
- **Then:** Two lines; line 2 is `resume`; exit code 0.; `resume` accepted
- **Exit:** 0
- **Source:** [param/20_strategy.md](../../../../docs/cli/param/20_strategy.md)

---

### EC-2: fresh accepted

- **Commands:** `.session.ensure`
- **Given:** TempDir as HOME; base directory accessible.
- **When:** `clg .session.ensure path::{base} strategy::fresh`
- **Then:** Two lines; line 2 is `fresh`; exit code 0.; + `fresh` accepted
- **Exit:** 0
- **Source:** [param/20_strategy.md](../../../../docs/cli/param/20_strategy.md)

---

### EC-3: Invalid value rejected

- **Commands:** `.session.ensure`
- **Given:** clean environment
- **When:** `clg .session.ensure path::{base} strategy::auto`
- **Then:** Error message containing `"strategy must be resume|fresh"`; exit code 1.; + correct error message
- **Exit:** 1
- **Source:** [param/20_strategy.md](../../../../docs/cli/param/20_strategy.md)

---

### EC-4: Case-insensitive: Resume accepted

- **Commands:** `.session.ensure`
- **Given:** TempDir as HOME.
- **When:** `clg .session.ensure path::{base} strategy::Resume`
- **Then:** Two lines; line 2 is `resume` (normalized to lowercase); exit code 0.; + mixed-case strategy accepted
- **Exit:** 0
- **Source:** [param/20_strategy.md](../../../../docs/cli/param/20_strategy.md)

---

### EC-5: Case-insensitive: FRESH accepted

- **Commands:** `.session.ensure`
- **Given:** TempDir as HOME.
- **When:** `clg .session.ensure path::{base} strategy::FRESH`
- **Then:** Two lines; line 2 is `fresh`; exit code 0.; + uppercase strategy accepted
- **Exit:** 0
- **Source:** [param/20_strategy.md](../../../../docs/cli/param/20_strategy.md)

---

### EC-6: Absent defaults to auto-detect (fresh when no history)

- **Commands:** `.session.ensure`
- **Given:** TempDir as HOME with NO matching storage for the session directory.
- **When:** `clg .session.ensure path::{base}`
- **Then:** Line 2 is `fresh`.; line 2 is "fresh" when no history exists and strategy not forced
- **Exit:** 0
- **Source:** [param/20_strategy.md](../../../../docs/cli/param/20_strategy.md)

---

### EC-7: Absent defaults to auto-detect (resume when history exists)

- **Commands:** `.session.ensure`
- **Given:** TempDir as HOME; create `~/.claude/projects/{encoded_session_dir}/` with a non-empty `.jsonl` file.
- **When:** `clg .session.ensure path::{base} topic::{topic}`
- **Then:** Line 2 is `resume`.; line 2 is "resume" when history exists and strategy not forced
- **Exit:** 0
- **Source:** [param/20_strategy.md](../../../../docs/cli/param/20_strategy.md)

---

### EC-8: resume forced overrides auto-detect fresh

- **Commands:** `.session.ensure`
- **Given:** TempDir as HOME with NO matching storage (auto-detect would be `fresh`).
- **When:** `clg .session.ensure path::{base} strategy::resume`
- **Then:** Line 2 is `resume` (not `fresh`).; line 2 is "resume" despite auto-detect being "fresh"
- **Exit:** 0
- **Source:** [param/20_strategy.md](../../../../docs/cli/param/20_strategy.md)

---

### EC-9: fresh forced overrides auto-detect resume

- **Commands:** `.session.ensure`
- **Given:** TempDir as HOME; create storage history (auto-detect would be `resume`).
- **When:** `clg .session.ensure path::{base} topic::{topic} strategy::fresh`
- **Then:** Line 2 is `fresh` (not `resume`).; line 2 is "fresh" despite existing history
- **Exit:** 0
- **Source:** [param/20_strategy.md](../../../../docs/cli/param/20_strategy.md)
