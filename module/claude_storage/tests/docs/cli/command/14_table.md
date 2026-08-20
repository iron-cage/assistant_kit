# Command :: `.table`

Integration tests for the `.table` command, implemented in `tests/cli_cmd_table_test.rs`. Tests verify grouping (session/project/model/day), `sort::`/`order::` wiring, column projection (`columns::`), the `model::` filter's percent-recompute-against-filtered-total behavior, `limit::`'s post-aggregation cap semantics, a byte-exact worked-example render, and exit/validation codes. `scope::`/`depth::` reuse `.usage`'s own machinery byte-for-byte and are only smoke-tested here (INT-11/INT-12) — exhaustive per-value coverage lives in `cli_cmd_usage_test.rs`'s own INT-1 through INT-8.

**Source:** [command/14_table.md](../../../../docs/cli/command/14_table.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| INT-1 | Default group::session shows one row per session | Grouping |
| INT-2 | group::project sums multiple sessions into one row | Grouping |
| INT-3 | group::model separates rows by model name | Grouping |
| INT-4 | group::day separates rows by calendar day | Grouping |
| INT-5 | sort::calls order::desc orders by call count, not total | Sorting & Order |
| INT-6 | order::asc reverses the sort::calls result from INT-5 | Sorting & Order |
| INT-7 | columns:: custom subset projects only the chosen columns | Column Projection |
| INT-8 | Default columns:: excludes First/Last | Column Projection |
| INT-9 | model:: filters before grouping; Pct recomputes against filtered total | Filtering |
| INT-10 | limit:: caps the grouped row count, not the raw session count | Limit Semantics |
| INT-11 | scope::global reaches .table (representative smoke test) | Reused Scope Machinery |
| INT-12 | depth:: caps candidates beyond the component distance (smoke test) | Reused Scope Machinery |
| INT-13 | Full table render matches the worked example byte-for-byte | Worked Example |
| INT-14 | No matching sessions in non-local scope exits 0 with header-only output | Exit Codes |
| INT-15 | scope::local with no project at cwd exits 2 | Exit Codes |
| INT-16 | Invalid group:: value rejected | Input Validation |
| INT-17 | Invalid sort:: value rejected | Input Validation |
| INT-18 | Invalid order:: value rejected | Input Validation |
| INT-19 | Invalid columns:: entry rejected | Input Validation |
| INT-20 | Negative depth:: is rejected | Input Validation |
| INT-21 | Negative limit:: is rejected | Input Validation |

## Test Coverage Summary

- Grouping: 4 tests (INT-1 through INT-4)
- Sorting & Order: 2 tests (INT-5, INT-6)
- Column Projection: 2 tests (INT-7, INT-8)
- Filtering: 1 test (INT-9)
- Limit Semantics: 1 test (INT-10)
- Reused Scope Machinery: 2 tests (INT-11, INT-12)
- Worked Example: 1 test (INT-13)
- Exit Codes: 2 tests (INT-14, INT-15)
- Input Validation: 6 tests (INT-16 through INT-21)

## Test Cases

---

### INT-1: Default group::session shows one row per session

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .table
```

**Expected behavior:**
- Fixture: two sessions in the same project, each a known 8-char short id prefix
- Output has a header row and exactly 2 data rows — one per session, matching `.usage`'s own per-session granularity at the default
- Both short ids appear in stdout
- Exit code: 0
- **Source:** [command/14_table.md](../../../../docs/cli/command/14_table.md)

---

### INT-2: group::project sums multiple sessions into one row

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .table group::project
```

**Expected behavior:**
- Fixture: two sessions in the same project with input totals 600 and 400 (neither session shows the summed value alone)
- Output has exactly 1 data row; its `Total` column reads `1.0k` (600+400)
- Exit code: 0
- **Source:** [command/14_table.md](../../../../docs/cli/command/14_table.md)

---

### INT-3: group::model separates rows by model name

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .table group::model
```

**Expected behavior:**
- Fixture: two sessions in the same project with distinct models (`claude-opus-5`, `claude-haiku-5`)
- Output has exactly 2 data rows, each labeled by its model name — the two sessions do not merge despite sharing a project
- Exit code: 0
- **Source:** [command/14_table.md](../../../../docs/cli/command/14_table.md)

---

### INT-4: group::day separates rows by calendar day

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .table group::day
```

**Expected behavior:**
- Fixture: two sessions in the same project with `first_timestamp` on distinct calendar days (`2025-06-01`, `2025-06-05`)
- Output has exactly 2 data rows, each labeled `YYYY-MM-DD`
- Exit code: 0
- **Source:** [command/14_table.md](../../../../docs/cli/command/14_table.md)

---

### INT-5: sort::calls order::desc orders by call count, not total

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .table sort::calls order::desc
```

**Expected behavior:**
- Fixture: three sessions with calls/total deliberately inversely correlated — S1 (1 call, total 300), S2 (3 calls, total 200), S3 (5 calls, total 100)
- Row order in stdout is S3, then S2, then S1 (most calls first) — proves `sort::` actually reorders away from the default `total`-descending order, since a total-based sort would produce the opposite order
- Exit code: 0
- **Source:** [command/14_table.md](../../../../docs/cli/command/14_table.md), [param/33_sort.md](../../../../docs/cli/param/33_sort.md)

---

### INT-6: order::asc reverses the sort::calls result from INT-5

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .table sort::calls order::asc
```

**Expected behavior:**
- Fixture: identical to INT-5 (S1/S2/S3 with 1/3/5 calls)
- Row order in stdout is S1, then S2, then S3 (fewest calls first) — the exact reverse of INT-5's `order::desc` result under the same `sort::calls` key
- Exit code: 0
- **Source:** [command/14_table.md](../../../../docs/cli/command/14_table.md), [param/34_order.md](../../../../docs/cli/param/34_order.md)

---

### INT-7: columns:: custom subset projects only the chosen columns

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .table columns::group,total
```

**Expected behavior:**
- Fixture: one session
- Header row contains exactly `Group` and `Total` labels; every other column label (`Sessions`, `Calls`, `Input`, `Output`, `Cache`, `MaxCtx`, `Pct`, `First`, `Last`) is absent
- Exit code: 0
- **Source:** [command/14_table.md](../../../../docs/cli/command/14_table.md), [param/36_columns.md](../../../../docs/cli/param/36_columns.md)

---

### INT-8: Default columns:: excludes First/Last

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .table
```

**Expected behavior:**
- Fixture: one session
- Header row contains all 9 default labels (`Group`, `Sessions`, `Calls`, `Input`, `Output`, `Cache`, `MaxCtx`, `Total`, `Pct`); `First` and `Last` are both absent
- Exit code: 0
- **Source:** [command/14_table.md](../../../../docs/cli/command/14_table.md), [param/36_columns.md](../../../../docs/cli/param/36_columns.md)

---

### INT-9: model:: filters before grouping; Pct recomputes against filtered total

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .table model::opus
```

**Expected behavior:**
- Fixture: two `claude-opus-5` sessions (100 tokens each) and one `claude-haiku-5` session (800 tokens) in the same project
- Output has exactly 2 data rows (haiku session entirely absent, not merely hidden); each surviving row shows `50.0%`, computed against the filtered 200-token total — never `10.0%`, which is what the same row would show against the unfiltered 1000-token grand total
- Exit code: 0
- **Source:** [command/14_table.md](../../../../docs/cli/command/14_table.md), [param/35_model.md](../../../../docs/cli/param/35_model.md)

---

### INT-10: limit:: caps the grouped row count, not the raw session count

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .table scope::global group::project limit::2
```

**Expected behavior:**
- Fixture: three distinct single-session projects with totals 900, 600, and 300
- Output has exactly 2 data rows — the 900 and 600 rows survive, the 300 row is cut entirely, not merely reordered — proving the cap applies AFTER `group::project` aggregation and `sort::`/`order::` ranking, distinct from `.usage`'s own flat per-session `limit::`
- Exit code: 0
- **Source:** [command/14_table.md](../../../../docs/cli/command/14_table.md), [param/22_limit.md](../../../../docs/cli/param/22_limit.md)

---

### INT-11: scope::global reaches .table (representative smoke test)

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .table scope::global
```

**Expected behavior:**
- Fixture: two sessions in two unrelated projects
- Both sessions appear in stdout — confirms `.table` genuinely wires `scope::` into `resolve_scoped_projects`, the same function `.usage` uses; NOT an exhaustive re-derivation of `.usage`'s own 5-scope-value coverage (`cli_cmd_usage_test.rs` INT-1 through INT-5), which already covers every `scope::` value exhaustively against the identical underlying function
- Exit code: 0
- **Source:** [command/14_table.md](../../../../docs/cli/command/14_table.md), [param/12_scope.md](../../../../docs/cli/param/12_scope.md)

---

### INT-12: depth:: caps candidates beyond the component distance (smoke test)

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .table scope::under path::/a depth::1
```

**Expected behavior:**
- Fixture: projects at `/a`, `/a/b`, `/a/b/c`, each with one session
- Output contains the `/a` (distance 0) and `/a/b` (distance 1) sessions; the `/a/b/c` (distance 2) session is absent — confirms `.table` wires `depth::` into the same `beyond_depth`/`component_distance` boundary check `.usage` already exhaustively tests (`cli_cmd_usage_test.rs` INT-7/INT-8); NOT a re-derivation
- Exit code: 0
- **Source:** [command/14_table.md](../../../../docs/cli/command/14_table.md), [param/26_depth.md](../../../../docs/cli/param/26_depth.md)

---

### INT-13: Full table render matches the worked example byte-for-byte

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .table
```

**Expected behavior:**
- Fixture: project at cwd with two sessions built to match the doc's worked example exactly: session 1 (4 calls, In=500, Out=300, Cache=200), session 2 (2 calls, In=100, Out=50, Cache=50)
- stdout equals, byte-for-byte:
```
Group                     Sessions   Calls     Input    Output     Cache    MaxCtx     Total     Pct
aaaaaaaa                         1       4       500       300       200       700      1.0k   83.3%
bbbbbbbb                         1       2       100        50        50       150       200   16.7%
```
- Exit code: 0
- **Source:** [command/14_table.md](../../../../docs/cli/command/14_table.md)

---

### INT-14: No matching sessions in non-local scope exits 0 with header-only output

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .table scope::global
```

**Expected behavior:**
- Fixture: empty storage — no projects
- stdout is exactly the header row (`Group  Sessions  Calls  Input  Output  Cache  MaxCtx  Total  Pct`, correctly widthed); no data rows; stderr is empty
- Exit code: 0
- **Source:** [command/14_table.md](../../../../docs/cli/command/14_table.md)

---

### INT-15: scope::local with no project at cwd exits 2

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .table
```

**Expected behavior:**
- Fixture: run from a directory with no matching storage project; default `scope::local` applies
- stderr contains exactly `"No project found for current directory"`
- Exit code: 2
- **Source:** [command/14_table.md](../../../../docs/cli/command/14_table.md)

---

### INT-16: Invalid group:: value rejected

**Command:**
```
clg .table group::bogus
```

**Expected behavior:**
- `bogus` is not a valid `group::` value (accepted: `session`, `project`, `model`, `day`)
- stderr names the invalid value; no table output on stdout
- Exit code: 1
- **Source:** [command/14_table.md](../../../../docs/cli/command/14_table.md), [param/32_group.md](../../../../docs/cli/param/32_group.md)

---

### INT-17: Invalid sort:: value rejected

**Command:**
```
clg .table sort::bogus
```

**Expected behavior:**
- `bogus` is not a valid `sort::` value (accepted: `total`, `input`, `output`, `cache`, `max_context`, `calls`, `sessions`, `group`)
- stderr names the invalid value; no table output on stdout
- Exit code: 1
- **Source:** [command/14_table.md](../../../../docs/cli/command/14_table.md), [param/33_sort.md](../../../../docs/cli/param/33_sort.md)

---

### INT-18: Invalid order:: value rejected

**Command:**
```
clg .table order::bogus
```

**Expected behavior:**
- `bogus` is not a valid `order::` value (accepted: `asc`, `desc`)
- stderr names the invalid value; no table output on stdout
- Exit code: 1
- **Source:** [command/14_table.md](../../../../docs/cli/command/14_table.md), [param/34_order.md](../../../../docs/cli/param/34_order.md)

---

### INT-19: Invalid columns:: entry rejected

**Command:**
```
clg .table columns::group,bogus
```

**Expected behavior:**
- `bogus` is not a valid column key, even alongside the valid `group` entry in the same list
- stderr names the invalid value; no table output on stdout
- Exit code: 1
- **Source:** [command/14_table.md](../../../../docs/cli/command/14_table.md), [param/36_columns.md](../../../../docs/cli/param/36_columns.md)

---

### INT-20: Negative depth:: is rejected

**Command:**
```
clg .table depth::-1
```

**Expected behavior:**
- stderr is exactly `"depth must be non-negative"` — identical validation code path to `.usage`'s own INT-20
- No table output on stdout
- Exit code: 1
- **Source:** [command/14_table.md](../../../../docs/cli/command/14_table.md), [param/26_depth.md](../../../../docs/cli/param/26_depth.md)

---

### INT-21: Negative limit:: is rejected

**Command:**
```
clg .table limit::-1
```

**Expected behavior:**
- stderr is exactly `"limit must be non-negative"` — identical validation code path to `.usage`'s own INT-21
- No table output on stdout
- Exit code: 1
- **Source:** [command/14_table.md](../../../../docs/cli/command/14_table.md), [param/22_limit.md](../../../../docs/cli/param/22_limit.md)
