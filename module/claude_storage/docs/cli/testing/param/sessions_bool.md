# Parameter :: `sessions::` (bool)

Edge case tests for the `sessions::` boolean override parameter in `.list`. Tests validate override behavior against auto-enable logic.

**Source:** [params.md#parameter--15-sessions-bool](../../params.md#parameter--15-sessions-bool)

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

## Test Cases

### EC-1: sessions::1 forces session display with no filters

**Goal:** Verify that `sessions::1` forces session listing in `.list` output even when no session filter parameters are provided.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .list sessions::1`
**Expected Output:** stdout includes session entries under each project; sessions are shown despite no `session::`, `agent::`, or `min_entries::` being set.
**Verification:**
- Exit code is 0
- Output includes session-level entries (not just project summaries)
- Sessions appear under at least one project in the output
**Pass Criteria:** exit 0 + sessions displayed (override active with no session filters present)
**Source:** [params.md](../../params.md)

### EC-2: sessions::0 suppresses session display even with session::

**Goal:** Verify that `sessions::0` suppresses session listing even when `session::` is also specified (explicit override beats auto-enable).
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .list sessions::0 session::default`
**Expected Output:** stdout lists matching projects but does not expand sessions under them; `session::default` acts as a project filter but sessions are not shown.
**Verification:**
- Exit code is 0
- Output contains project entries matching the `session::default` filter
- No session-level entries appear in output
**Pass Criteria:** exit 0 + no sessions displayed despite session:: filter (suppression override applied)
**Source:** [params.md](../../params.md)

### EC-3: sessions::0 suppresses session display even with agent::

**Goal:** Verify that `sessions::0` suppresses session listing even when `agent::` is also specified.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .list sessions::0 agent::1`
**Expected Output:** stdout lists projects (filtered to those with agent sessions) but does not display the session entries themselves.
**Verification:**
- Exit code is 0
- Output may list projects that have agent sessions
- No session-level detail entries appear in output
**Pass Criteria:** exit 0 + no sessions displayed despite agent:: filter (suppression override applied)
**Source:** [params.md](../../params.md)

### EC-4: sessions::0 suppresses session display even with min_entries::

**Goal:** Verify that `sessions::0` suppresses session listing even when `min_entries::` is also specified.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .list sessions::0 min_entries::2`
**Expected Output:** stdout lists projects (filtered to those meeting the min_entries threshold) but does not display session entries.
**Verification:**
- Exit code is 0
- Output may list qualifying projects
- No session-level detail entries appear in output
**Pass Criteria:** exit 0 + no sessions displayed despite min_entries:: filter (suppression override applied)
**Source:** [params.md](../../params.md)

### EC-5: Omitted + no session filters = no sessions shown

**Goal:** Verify the default behavior: when `sessions::` is omitted and no session filter params are set, session entries are not shown.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .list`
**Expected Output:** stdout lists projects as summaries only; no session-level entries are expanded under projects.
**Verification:**
- Exit code is 0
- Output contains project-level entries only
- No session-level detail entries appear in output
**Pass Criteria:** exit 0 + only project summaries shown (auto-detect: no filters → no sessions)
**Source:** [params.md](../../params.md)

### EC-6: Omitted + session:: present = sessions auto-shown

**Goal:** Verify the auto-enable behavior: when `sessions::` is omitted but `session::` is present, session display is automatically enabled.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .list session::default`
**Expected Output:** stdout includes session entries matching the `session::default` filter; sessions are shown automatically without explicit `sessions::1`.
**Verification:**
- Exit code is 0
- Output includes session-level entries matching `default` in the session ID
- Session display was activated automatically by the presence of `session::`
**Pass Criteria:** exit 0 + sessions displayed automatically (auto-enable triggered by session:: filter)
**Source:** [params.md](../../params.md)

### EC-7: Value "yes" rejected (not a boolean)

**Goal:** Verify that `sessions::yes` is rejected because only `0` and `1` are valid boolean values.
**Setup:** None
**Command:** `clg .list sessions::yes`
**Expected Output:** stderr contains an error indicating `sessions` must be 0 or 1.
**Verification:**
- Exit code is 1
- stderr contains a message like `sessions must be 0 or 1` or similar boolean rejection error
**Pass Criteria:** exit 1 + error indicating non-boolean value rejected
**Source:** [params.md](../../params.md)
