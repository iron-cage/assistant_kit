# Command :: `.usage`

Integration tests for the `.usage` command, implemented in `tests/cli_cmd_usage_test.rs`. Tests verify scope resolution (reusing the canonical `ScopeValue` semantics), `depth::` boundary behavior, agent-session exclusion, `limit::`/ordering, and the column-formatting rules (short id, command truncation, k/M token suffixes, s/m/h duration).

**Source:** [command/13_usage.md](../../../../docs/cli/command/13_usage.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| INT-1 | No args defaults to scope::local, single session in cwd's project | Scope Behavior |
| INT-2 | scope::relevant includes ancestor project sessions | Scope Behavior |
| INT-3 | scope::under includes descendant project sessions | Scope Behavior |
| INT-4 | scope::around includes both ancestor and descendant sessions | Scope Behavior |
| INT-5 | scope::global returns all sessions regardless of path::/depth:: | Scope Behavior |
| INT-6 | path:: overrides cwd as scope anchor | Path Anchoring |
| INT-7 | depth:: caps candidates beyond the component distance | Depth Boundary |
| INT-8 | depth::0 is unbounded | Depth Boundary |
| INT-9 | Agent sessions excluded from every scope | Agent Exclusion |
| INT-10 | limit::N caps the flat result set | Limit & Ordering |
| INT-11 | Sessions ordered most-recent-first by mtime | Limit & Ordering |
| INT-12 | Session column shows 8-character short id | Output Formatting |
| INT-13 | Command column truncates at 35 chars with trailing … | Output Formatting |
| INT-14 | In/Out/Cache columns use k/M-suffix formatting | Output Formatting |
| INT-15 | Dur column formats seconds/minutes/hours boundaries | Output Formatting |
| INT-16 | Column values match Session::stats() aggregation exactly | Column Values |
| INT-17 | No matching sessions in non-local scope exits 0 with empty table | Exit Codes |
| INT-18 | scope::local with no project at cwd exits 2 | Exit Codes |
| INT-19 | Invalid scope:: value rejected | Input Validation |
| INT-20 | Negative depth:: is rejected | Input Validation |
| INT-21 | Negative limit:: is rejected | Input Validation |

## Test Coverage Summary

- Scope Behavior: 5 tests (INT-1 through INT-5)
- Path Anchoring: 1 test (INT-6)
- Depth Boundary: 2 tests (INT-7, INT-8)
- Agent Exclusion: 1 test (INT-9)
- Limit & Ordering: 2 tests (INT-10, INT-11)
- Output Formatting: 4 tests (INT-12 through INT-15)
- Column Values: 1 test (INT-16)
- Exit Codes: 2 tests (INT-17, INT-18)
- Input Validation: 3 tests (INT-19 through INT-21)

## Test Cases

---

### INT-1: No args defaults to scope::local, single session in cwd's project

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .usage
```

**Expected behavior:**
- Fixture: project at cwd containing one non-agent session with known Turns/In/Out/Cache/Dur values
- Output is a table with a header row (`Session Command Turns In Out Cache Dur Dir`) and exactly one data row, for the cwd project's session only
- Exit code: 0
- **Source:** [command/13_usage.md](../../../../docs/cli/command/13_usage.md)

---

### INT-2: scope::relevant includes ancestor project sessions

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .usage scope::relevant
```

**Expected behavior:**
- Fixture: projects at `/a/b/c`, `/a/b`, and `/a`, each with one non-agent session; run from `/a/b/c`
- Table contains rows for all three projects' sessions
- Exit code: 0
- **Source:** [command/13_usage.md](../../../../docs/cli/command/13_usage.md)

---

### INT-3: scope::under includes descendant project sessions

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .usage scope::under
```

**Expected behavior:**
- Fixture: projects at `/a/b`, `/a/b/c`, `/a/b/c/d`, and `/z` (unrelated), each with one non-agent session; run from `/a/b`
- Table contains rows for `/a/b`, `/a/b/c`, and `/a/b/c/d`; no row for `/z`
- Exit code: 0
- **Source:** [command/13_usage.md](../../../../docs/cli/command/13_usage.md)

---

### INT-4: scope::around includes both ancestor and descendant sessions

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .usage scope::around
```

**Expected behavior:**
- Fixture: projects at `/a` (ancestor), `/a/b` (current), and `/a/b/c` (descendant), each with one non-agent session; run from `/a/b`
- Table contains rows for all three projects (union of `relevant` and `under`, deduplicated — `/a/b` itself appears exactly once)
- Exit code: 0
- **Source:** [command/13_usage.md](../../../../docs/cli/command/13_usage.md)

---

### INT-5: scope::global returns all sessions regardless of path::/depth::

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .usage scope::global path::/a/b depth::1
```

**Expected behavior:**
- Fixture: projects at `/a/b`, `/c/d`, and `/e/f`, each with one non-agent session; run from anywhere
- Table contains rows for all three projects — `path::`/`depth::` have no filtering effect under `scope::global`
- Exit code: 0
- **Source:** [command/13_usage.md](../../../../docs/cli/command/13_usage.md)

---

### INT-6: path:: overrides cwd as scope anchor

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .usage scope::local path::/a/b/c
```

**Expected behavior:**
- Fixture: projects at `/a/b/c`, `/a/b`, and `/a`, each with one non-agent session; run from `/tmp` (no project there)
- Table contains the row for `/a/b/c` only; cwd (`/tmp`) has no effect
- Exit code: 0
- **Source:** [command/13_usage.md](../../../../docs/cli/command/13_usage.md)

---

### INT-7: depth:: caps candidates beyond the component distance

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .usage scope::under path::/a depth::1
```

**Expected behavior:**
- Fixture: projects at `/a`, `/a/b` (1 component away), and `/a/b/c` (2 components away), each with one non-agent session; run with `path::/a`
- Table contains rows for `/a` and `/a/b`; no row for `/a/b/c` (beyond `depth::1`)
- Exit code: 0
- **Source:** [command/13_usage.md](../../../../docs/cli/command/13_usage.md)

---

### INT-8: depth::0 is unbounded

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .usage scope::under path::/a depth::0
```

**Expected behavior:**
- Fixture: same as INT-7 — `/a`, `/a/b`, `/a/b/c`, each with one non-agent session
- Table contains rows for all three projects — `depth::0` removes the component-distance cap
- Exit code: 0
- **Source:** [command/13_usage.md](../../../../docs/cli/command/13_usage.md)

---

### INT-9: Agent sessions excluded from every scope

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .usage scope::global
```

**Expected behavior:**
- Fixture: one project with one main (UUID-named) session and one `agent-*`-named sidecar session
- Table contains exactly one data row (the main session); the agent session never appears as its own row
- Exit code: 0
- **Source:** [command/13_usage.md](../../../../docs/cli/command/13_usage.md)

---

### INT-10: limit::N caps the flat result set

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .usage scope::global limit::2
```

**Expected behavior:**
- Fixture: three projects (`/a`, `/b`, `/c`), each with one non-agent session at distinct mtimes
- Table contains exactly 2 data rows — the cap is flat across the whole result set, not per-project (contrast `.projects`' per-project `limit::`)
- Exit code: 0
- **Source:** [command/13_usage.md](../../../../docs/cli/command/13_usage.md), [param/22_limit.md](../../../../docs/cli/param/22_limit.md)

---

### INT-11: Sessions ordered most-recent-first by mtime

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .usage scope::global
```

**Expected behavior:**
- Fixture: two sessions in different projects, written with distinguishable mtimes (older written first, then a delay, then the newer)
- The newer session's row appears before the older session's row in stdout
- Exit code: 0
- **Source:** [command/13_usage.md](../../../../docs/cli/command/13_usage.md)

---

### INT-12: Session column shows 8-character short id

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .usage
```

**Expected behavior:**
- Fixture: project at cwd with one session whose full ID is a known UUID (e.g. `bf61b676-1234-4abc-9def-0123456789ab`)
- The `Session` column shows exactly the first 8 characters (`bf61b676`), matching the `short_id()` helper `.projects` already uses — never the full UUID
- Exit code: 0
- **Source:** [command/13_usage.md](../../../../docs/cli/command/13_usage.md)

---

### INT-13: Command column truncates at 35 chars with trailing …

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .usage
```

**Expected behavior:**
- Fixture: project at cwd with one session whose first non-sidechain user entry is a known 50-character string
- The `Command` column shows exactly the first 35 characters followed by `…`; the full 50-character string does not appear verbatim
- Exit code: 0
- **Source:** [command/13_usage.md](../../../../docs/cli/command/13_usage.md)

---

### INT-14: In/Out/Cache columns use k/M-suffix formatting

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .usage
```

**Expected behavior:**
- Fixture: project at cwd with one session having known token totals: In=500 (< 1000), Out=44800 (mid-range), Cache=4800000 (≥ 1000000)
- Row shows `In` as bare `500`, `Out` as `44.8k`, `Cache` as `4.8M` — never raw unformatted integers (contrast [`.status`](../../../../docs/cli/command/01_status.md)'s `show_tokens::1`, which does print raw integers)
- Exit code: 0
- **Source:** [command/13_usage.md](../../../../docs/cli/command/13_usage.md)

---

### INT-15: Dur column formats seconds/minutes/hours boundaries

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .usage scope::global
```

**Expected behavior:**
- Fixture: three sessions with known `first_timestamp`/`last_timestamp` spans: 45s, 324s (5m24s), and 3661s (1h01m)
- Rows show `Dur` as `45s`, `5m24s`, and `1h01m` respectively
- Exit code: 0
- **Source:** [command/13_usage.md](../../../../docs/cli/command/13_usage.md)

---

### INT-16: Column values match Session::stats() aggregation exactly

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .usage
```

**Expected behavior:**
- Fixture: project at cwd with one session built to match the doc's worked example: 31 assistant entries, In=44800, Out=105800, Cache=4800000, Dur=324s, first user entry `/role`, cwd `/data/repos/yrd_review/2026_troy_venue_pipeline_dev/pr_101`
- Row reads: `Turns=31`, `In=44.8k`, `Out=105.8k`, `Cache=4.8M`, `Dur=5m24s`, `Command=/role`, `Dir=/data/repos/yrd_review/2026_troy_venue_pipeline_dev/pr_101` — matching [command/13_usage.md](../../../../docs/cli/command/13_usage.md)'s Output example row exactly
- Exit code: 0
- **Source:** [command/13_usage.md](../../../../docs/cli/command/13_usage.md)

---

### INT-17: No matching sessions in non-local scope exits 0 with empty table

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .usage scope::global
```

**Expected behavior:**
- Fixture: empty storage — no projects
- stdout is empty or contains only the header row (no data rows); no error on stderr
- Exit code: 0
- **Source:** [command/13_usage.md](../../../../docs/cli/command/13_usage.md)

---

### INT-18: scope::local with no project at cwd exits 2

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .usage
```

**Expected behavior:**
- Fixture: run from a directory (e.g., `/tmp`) that has no matching storage project; default `scope::local` applies
- Error message on stderr indicating the current directory has no project in storage
- Exit code: 2
- **Source:** [command/13_usage.md](../../../../docs/cli/command/13_usage.md)

---

### INT-19: Invalid scope:: value rejected

**Command:**
```
clg .usage scope::badvalue
```

**Expected behavior:**
- `badvalue` is not a valid option for `scope::` (accepted: `local`, `relevant`, `under`, `around`, `global`)
- Error message on stderr naming the invalid value
- No table output on stdout
- Exit code: 1
- **Source:** [command/13_usage.md](../../../../docs/cli/command/13_usage.md), [type/07_scope_value.md](../../../../docs/cli/type/07_scope_value.md)

---

### INT-20: Negative depth:: is rejected

**Command:**
```
clg .usage depth::-1
```

**Expected behavior:**
- Error message on stderr: exactly `"depth must be non-negative"`
- No table output on stdout
- Exit code: 1
- **Source:** [command/13_usage.md](../../../../docs/cli/command/13_usage.md), [param/26_depth.md](../../../../docs/cli/param/26_depth.md)

---

### INT-21: Negative limit:: is rejected

**Command:**
```
clg .usage limit::-1
```

**Expected behavior:**
- Error message on stderr: exactly `"limit must be non-negative"`
- No table output on stdout
- Exit code: 1
- **Source:** [command/13_usage.md](../../../../docs/cli/command/13_usage.md), [param/22_limit.md](../../../../docs/cli/param/22_limit.md)
