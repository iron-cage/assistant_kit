# Parameter Group :: Output Control

Interaction tests for the Output Control group (`verbosity::`). Tests verify consistent verbosity semantics across all commands using this group.

**Source:** [parameter_groups.md#output-control](../../../../docs/cli/parameter_groups.md#output-control)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | verbosity::0 gives minimal output in .status | Cross-Command |
| EC-2 | verbosity::0 gives minimal output in .list | Cross-Command |
| EC-3 | verbosity::1 default is consistent across commands | Cross-Command |
| EC-4 | verbosity::2 adds detail in .status | Cross-Command |
| EC-5 | v:: alias works in .list same as verbosity:: | Alias Consistency |
| EC-6 | verbosity level does not affect which results are returned | Non-Interference |

## Test Coverage Summary

- Cross-Command: 4 tests (EC-1, EC-2, EC-3, EC-4)
- Alias Consistency: 1 test (EC-5)
- Non-Interference: 1 test (EC-6)

## Test Cases

---

### EC-1: verbosity::0 gives minimal output in .status

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with at least one project present.
- **When:** `clg .status verbosity::0`
- **Then:** Bare value or minimal fields with no labels, headers, or decorations.; output is minimal/unlabelled
- **Exit:** 0
- **Source:** [parameter_groups.md#output-control](../../../../docs/cli/parameter_groups.md#output-control)

---

### EC-2: verbosity::0 gives minimal output in .list

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with at least one project present.
- **When:** `clg .list verbosity::0`
- **Then:** One path per line, no header line and no count footer.; undecorated one-per-line output
- **Exit:** 0
- **Source:** [parameter_groups.md#output-control](../../../../docs/cli/parameter_groups.md#output-control)

---

### EC-3: verbosity::1 default is consistent across commands

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with at least one project present.
- **When:** `clg .status` and `clg .list` (no verbosity param — uses default verbosity::1)
- **Then:** Both commands produce standard summary output with labels and counts.; consistent styled output across both commands
- **Exit:** 0
- **Source:** [parameter_groups.md#output-control](../../../../docs/cli/parameter_groups.md#output-control)

---

### EC-4: verbosity::2 adds detail in .status

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with at least one project and multiple sessions.
- **When:** `clg .status verbosity::2`
- **Then:** More detailed breakdown than verbosity::1, such as per-project session counts.; output is more detailed than verbosity::1
- **Exit:** 0
- **Source:** [parameter_groups.md#output-control](../../../../docs/cli/parameter_groups.md#output-control)

---

### EC-5: v:: alias works in .list same as verbosity::

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with at least one project present.
- **When:** `clg .list v::0` (compare against `clg .list verbosity::0`)
- **Then:** Identical output to `clg .list verbosity::0`.; output identical to the long-form version
- **Exit:** 0
- **Source:** [parameter_groups.md#output-control](../../../../docs/cli/parameter_groups.md#output-control)

---

### EC-6: verbosity level does not affect which results are returned

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with exactly 3 projects.
- **When:** `clg .list verbosity::0` and `clg .list verbosity::3`
- **Then:** Both commands return all 3 projects; only the format of each entry differs.; same result set at all verbosity levels
- **Exit:** 0
- **Source:** [parameter_groups.md#output-control](../../../../docs/cli/parameter_groups.md#output-control)
