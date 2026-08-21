# Parameter :: `show_tree::`

Edge case tests for the `show_tree::` parameter. Tests validate boolean acceptance, default behavior, and display mode switching.

**Source:** [param/24_show_tree.md](../../../../docs/cli/param/24_show_tree.md)

> **Note:** `show_tree::` nests whatever the active `detail::` level lists — agent sessions under their root under `detail::sessions`, projects under their shared directory under `detail::projects`. EC-1 through EC-6 cover the session level and therefore pass `detail::sessions` explicitly, since it is no longer the default. The project level is covered by [`30_detail.md`](30_detail.md) EC-6 and [command/07_projects.md INT-65b](../command/07_projects.md).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Value 0 accepted (compact format) | Valid Input |
| EC-2 | Value 1 accepted (tree-indented format) | Valid Input |
| EC-3 | Non-boolean value rejected | Type Validation |
| EC-4 | Omitted uses default of 0 | Default |
| EC-5 | Tree format shows agent connectors | Display Format |
| EC-6 | Single root without agents in tree mode | Boundary |

## Test Coverage Summary

- Valid Input: 2 tests (EC-1, EC-2)
- Type Validation: 1 test (EC-3)
- Default: 1 test (EC-4)
- Display Format: 1 test (EC-5)
- Boundary: 1 test (EC-6)

**Total:** 6 edge cases

**Behavioral Divergence Pair:** EC-1 (show_tree::0, compact family summary) ↔ EC-2 (show_tree::1, tree-indented agents)

## Test Cases

---

### EC-1: Value 0 accepted (compact format)

- **Commands:** `.projects`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .projects detail::sessions show_tree::0`
- **Then:** Compact family summary format — root session with short UUID, mtime, entry count, inline agent summary
- **Exit:** 0
- **Source:** [param/24_show_tree.md](../../../../docs/cli/param/24_show_tree.md)

---

### EC-2: Value 1 accepted (tree-indented format)

- **Commands:** `.projects`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .projects detail::sessions show_tree::1`
- **Then:** Tree-indented format — each agent session indented under root with connectors, full UUID, per-session entry count
- **Exit:** 0
- **Source:** [param/24_show_tree.md](../../../../docs/cli/param/24_show_tree.md)

---

### EC-3: Non-boolean value rejected

- **Commands:** `.projects`
- **Given:** clean environment
- **When:** `clg .projects detail::sessions show_tree::yes`
- **Then:** Error message indicating boolean expected (0 or 1)
- **Exit:** 1
- **Source:** [param/24_show_tree.md](../../../../docs/cli/param/24_show_tree.md)

---

### EC-4: Omitted uses default of 0

- **Commands:** `.projects`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .projects detail::sessions`
- **Then:** Compact family summary format (same as show_tree::0)
- **Exit:** 0
- **Source:** [param/24_show_tree.md](../../../../docs/cli/param/24_show_tree.md)

---

### EC-5: Tree format shows agent connectors

- **Commands:** `.projects`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with project having root + agent sessions
- **When:** `clg .projects detail::sessions show_tree::1`
- **Then:** Output contains tree connectors (`├─` / `└─`) with agent sessions indented under root
- **Exit:** 0
- **Source:** [param/24_show_tree.md](../../../../docs/cli/param/24_show_tree.md)

---

### EC-6: Single root without agents in tree mode

- **Commands:** `.projects`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with project having only a root session (no agents)
- **When:** `clg .projects detail::sessions show_tree::1`
- **Then:** Tree format shows root session without connector lines (no connectors)
- **Exit:** 0
- **Source:** [param/24_show_tree.md](../../../../docs/cli/param/24_show_tree.md)
