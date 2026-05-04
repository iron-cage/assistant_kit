# Parameter :: `sessions::` (bool)

Edge case tests for the `sessions::` boolean override parameter in `.list`. Tests validate override behavior against auto-enable logic.

**Source:** [params.md#parameter--15-sessions-bool](../../../../docs/cli/params.md#parameter--15-sessions-bool)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | sessions::1 forces session display with no filters | Override |
| EC-2 | sessions::0 suppresses session display even with session:: | Override |
| EC-3 | sessions::0 suppresses session display even with agent:: | Override |
| EC-4 | sessions::0 suppresses session display even with min_entries:: | Override |
| EC-5 | Omitted + no session filters = no sessions shown | Default |
| EC-6 | Omitted + session:: present = sessions auto-shown | Default |
| EC-7 | Value "yes" rejected (not a boolean) | Type Validation |

## Test Coverage Summary

- Override: 4 tests (EC-1, EC-2, EC-3, EC-4)
- Default: 2 tests (EC-5, EC-6)
- Type Validation: 1 test (EC-7)

**Total:** 7 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

## Test Cases

---

### EC-1: sessions::1 forces session display with no filters

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list sessions::1`
- **Then:** stdout includes session entries under each project; sessions are shown despite no `session::`, `agent::`, or `min_entries::` being set.; sessions displayed (override active with no session filters present)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-2: sessions::0 suppresses session display even with session::

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list sessions::0 session::default`
- **Then:** stdout lists matching projects but does not expand sessions under them; `session::default` acts as a project filter but sessions are not shown.; no sessions displayed despite session:: filter (suppression override applied)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-3: sessions::0 suppresses session display even with agent::

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list sessions::0 agent::1`
- **Then:** stdout lists projects (filtered to those with agent sessions) but does not display the session entries themselves.; no sessions displayed despite agent:: filter (suppression override applied)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-4: sessions::0 suppresses session display even with min_entries::

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list sessions::0 min_entries::2`
- **Then:** stdout lists projects (filtered to those meeting the min_entries threshold) but does not display session entries.; no sessions displayed despite min_entries:: filter (suppression override applied)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-5: Omitted + no session filters = no sessions shown

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list`
- **Then:** stdout lists projects as summaries only; no session-level entries are expanded under projects.; only project summaries shown (auto-detect: no filters → no sessions)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-6: Omitted + session:: present = sessions auto-shown

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list session::default`
- **Then:** stdout includes session entries matching the `session::default` filter; sessions are shown automatically without explicit `sessions::1`.; sessions displayed automatically (auto-enable triggered by session:: filter)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-7: Value "yes" rejected (not a boolean)

- **Given:** clean environment
- **When:** `clg .list sessions::yes`
- **Then:** stderr contains an error indicating `sessions` must be 0 or 1.; error indicating non-boolean value rejected
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md)
