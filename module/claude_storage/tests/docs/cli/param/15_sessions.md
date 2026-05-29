# Parameter :: `show_sessions::` (bool)

Edge case tests for the `show_sessions::` boolean override parameter in `.list`. Tests validate override behavior against auto-enable logic.

**Source:** [param/15_sessions.md](../../../../docs/cli/param/15_sessions.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | show_sessions::1 forces session display with no filters | Override |
| EC-2 | show_sessions::0 suppresses session display even with session:: | Override |
| EC-3 | show_sessions::0 suppresses session display even with agent:: | Override |
| EC-4 | show_sessions::0 suppresses session display even with min_entries:: | Override |
| EC-5 | Omitted + no session filters = no sessions shown | Default |
| EC-6 | Omitted + session:: present = sessions auto-shown | Default |
| EC-7 | Value "yes" rejected (not a boolean) | Type Validation |

## Test Coverage Summary

- Override: 4 tests (EC-1, EC-2, EC-3, EC-4)
- Default: 2 tests (EC-5, EC-6)
- Type Validation: 1 test (EC-7)

**Total:** 7 edge cases

**Behavioral Divergence Pair:** EC-1 (show_sessions::1, force display) ↔ EC-2 (show_sessions::0, suppress display)

## Test Cases

---

### EC-1: show_sessions::1 forces session display with no filters

- **Commands:** `.list`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list show_sessions::1`
- **Then:** stdout includes session entries under each project; sessions are shown despite no `session::`, `agent::`, or `min_entries::` being set.; sessions displayed (override active with no session filters present)
- **Exit:** 0
- **Source:** [param/15_sessions.md](../../../../docs/cli/param/15_sessions.md)

---

### EC-2: show_sessions::0 suppresses session display even with session::

- **Commands:** `.list`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list show_sessions::0 session::default`
- **Then:** stdout lists matching projects but does not expand sessions under them; `session::default` acts as a project filter but sessions are not shown.; no sessions displayed despite session:: filter (suppression override applied)
- **Exit:** 0
- **Source:** [param/15_sessions.md](../../../../docs/cli/param/15_sessions.md)

---

### EC-3: show_sessions::0 suppresses session display even with agent::

- **Commands:** `.list`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list show_sessions::0 agent::1`
- **Then:** stdout lists projects (filtered to those with agent sessions) but does not display the session entries themselves.; no sessions displayed despite agent:: filter (suppression override applied)
- **Exit:** 0
- **Source:** [param/15_sessions.md](../../../../docs/cli/param/15_sessions.md)

---

### EC-4: show_sessions::0 suppresses session display even with min_entries::

- **Commands:** `.list`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list show_sessions::0 min_entries::2`
- **Then:** stdout lists projects (filtered to those meeting the min_entries threshold) but does not display session entries.; no sessions displayed despite min_entries:: filter (suppression override applied)
- **Exit:** 0
- **Source:** [param/15_sessions.md](../../../../docs/cli/param/15_sessions.md)

---

### EC-5: Omitted + no session filters = no sessions shown

- **Commands:** `.list`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list`
- **Then:** stdout lists projects as summaries only; no session-level entries are expanded under projects.; only project summaries shown (auto-detect: no filters → no sessions)
- **Exit:** 0
- **Source:** [param/15_sessions.md](../../../../docs/cli/param/15_sessions.md)

---

### EC-6: Omitted + session:: present = sessions auto-shown

- **Commands:** `.list`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list session::default`
- **Then:** stdout includes session entries matching the `session::default` filter; sessions are shown automatically without explicit `show_sessions::1`.; sessions displayed automatically (auto-enable triggered by session:: filter)
- **Exit:** 0
- **Source:** [param/15_sessions.md](../../../../docs/cli/param/15_sessions.md)

---

### EC-7: Value "yes" rejected (not a boolean)

- **Commands:** `.list`
- **Given:** clean environment
- **When:** `clg .list show_sessions::yes`
- **Then:** stderr contains an error indicating `sessions` must be 0 or 1.; error indicating non-boolean value rejected
- **Exit:** 1
- **Source:** [param/15_sessions.md](../../../../docs/cli/param/15_sessions.md)
