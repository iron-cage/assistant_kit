# Parameter :: `count::`

Edge case tests for the `count::` parameter. Tests validate boolean enforcement, count-only output, and empty-state behavior.

**Source:** [param/21_count.md](../../../../docs/cli/param/21_count.md)

> **Note:** `.list` is deprecated (see [`command/02_list.md`](../command/02_list.md)), superseded by [`.projects`](../command/07_projects.md). `.list`'s `count::1` (EC-1–EC-6) counted all listed projects standalone, as a bare integer — that general project-counting role is now served by the `.count` command (`target::projects`) instead. `.projects` does reuse the `count::` name, but with narrower scope: only meaningful paired with `ids::1` (see [`31_ids.md`](../../../../docs/cli/param/31_ids.md)), where it outputs the bare-integer count of one `project::`-selected project's conversation IDs rather than a total project count — EC-1–EC-6 below do not carry over as-is.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `count::1` → integer count only, no list output | Count Mode |
| EC-2 | `count::0` → full list output (count mode off) | Default |
| EC-3 | `count::2` → rejected (must be 0 or 1) | Boundary Values |
| EC-4 | `count::yes` → rejected (type validation) | Type Validation |
| EC-5 | `count::1` with empty storage → outputs `0` | Empty State |
| EC-6 | `count::1` exits 0 even with no results | Exit Code |

## Test Coverage Summary

- Count Mode: 1 test (EC-1)
- Default: 1 test (EC-2)
- Boundary Values: 1 test (EC-3)
- Type Validation: 1 test (EC-4)
- Empty State: 1 test (EC-5)
- Exit Code: 1 test (EC-6)

**Total:** 6 edge cases

**Behavioral Divergence Pair:** EC-1 (count::1, integer output only) ↔ EC-2 (count::0, full list)

## Test Cases

---

### EC-1: `count::1` → integer count only

- **Commands:** `.list`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list count::1`
- **Then:** stdout is a single integer (the project count); no list items shown
- **Exit:** 0
- **Source:** [param/21_count.md](../../../../docs/cli/param/21_count.md)

---

### EC-2: `count::0` → full list (default behavior)

- **Commands:** `.list`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list count::0`
- **Then:** Full list of projects shown (same as without `count::1`)
- **Exit:** 0
- **Source:** [param/21_count.md](../../../../docs/cli/param/21_count.md)

---

### EC-3: `count::2` → rejected

- **Commands:** `.list`
- **Given:** clean environment
- **When:** `clg .list count::2`
- **Then:** `count must be 0 or 1`; exit 1
- **Exit:** 1
- **Source:** [param/21_count.md](../../../../docs/cli/param/21_count.md)

---

### EC-4: `count::yes` → rejected

- **Commands:** `.list`
- **Given:** clean environment
- **When:** `clg .list count::yes`
- **Then:** `count must be 0 or 1`; exit 1
- **Exit:** 1
- **Source:** [param/21_count.md](../../../../docs/cli/param/21_count.md)

---

### EC-5: `count::1` with empty storage → outputs `0`

- **Commands:** `.list`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/empty-fixture`
- **When:** `clg .list count::1`
- **Then:** stdout is `0` (no projects); exit 0
- **Exit:** 0
- **Source:** [param/21_count.md](../../../../docs/cli/param/21_count.md)

---

### EC-6: `count::1` exit code is 0 regardless of result

- **Commands:** `.list`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list count::1`
- **Then:** Exit code is 0 whether result is 0 or positive
- **Exit:** 0
- **Source:** [param/21_count.md](../../../../docs/cli/param/21_count.md)
