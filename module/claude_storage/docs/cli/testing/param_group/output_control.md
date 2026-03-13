# Parameter Group :: Output Control

Interaction tests for the Output Control group (`verbosity::`). Tests verify consistent verbosity semantics across all commands using this group.

**Source:** [parameter_groups.md#output-control](../../parameter_groups.md#output-control)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | verbosity::0 gives minimal output in .status | Cross-Command |
| CC-2 | verbosity::0 gives minimal output in .list | Cross-Command |
| CC-3 | verbosity::1 default is consistent across commands | Cross-Command |
| CC-4 | verbosity::2 adds detail in .status | Cross-Command |
| CC-5 | v:: alias works in .list same as verbosity:: | Alias Consistency |
| CC-6 | verbosity level does not affect which results are returned | Non-Interference |

## Test Coverage Summary

- Cross-Command: 4 tests (CC-1, CC-2, CC-3, CC-4)
- Alias Consistency: 1 test (CC-5)
- Non-Interference: 1 test (CC-6)

## Test Cases

### CC-1: verbosity::0 gives minimal output in .status

**Goal:** Verify that verbosity::0 produces machine-readable minimal output in `.status`.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with at least one project present.
**Command:** `clg .status verbosity::0`
**Expected Output:** Bare value or minimal fields with no labels, headers, or decorations.
**Verification:**
- Output contains no section headers or label strings (e.g., no "Storage Root:" label)
- Output contains the storage root path
- Fewer lines than verbosity::1 output for the same fixture
**Pass Criteria:** exit 0 + output is minimal/unlabelled
**Source:** [parameter_groups.md#output-control](../../parameter_groups.md#output-control)

### CC-2: verbosity::0 gives minimal output in .list

**Goal:** Verify that verbosity::0 produces machine-readable minimal output in `.list`.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with at least one project present.
**Command:** `clg .list verbosity::0`
**Expected Output:** One path per line, no header line and no count footer.
**Verification:**
- Each output line is a bare project path (no decorations or counts)
- No header or footer line present
- Line count equals project count in fixture
**Pass Criteria:** exit 0 + undecorated one-per-line output
**Source:** [parameter_groups.md#output-control](../../parameter_groups.md#output-control)

### CC-3: verbosity::1 default is consistent across commands

**Goal:** Verify that default verbosity (1) produces consistently styled output across commands.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with at least one project present.
**Command:** `clg .status` and `clg .list` (no verbosity param — uses default verbosity::1)
**Expected Output:** Both commands produce standard summary output with labels and counts.
**Verification:**
- Both commands produce labelled output (contains key=value or key: value style)
- Output is more detailed than verbosity::0 but less detailed than verbosity::2
- No crash or inconsistency between the two commands
**Pass Criteria:** exit 0 + consistent styled output across both commands
**Source:** [parameter_groups.md#output-control](../../parameter_groups.md#output-control)

### CC-4: verbosity::2 adds detail in .status

**Goal:** Verify that verbosity::2 produces more detailed output than verbosity::1 in `.status`.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with at least one project and multiple sessions.
**Command:** `clg .status verbosity::2`
**Expected Output:** More detailed breakdown than verbosity::1, such as per-project session counts.
**Verification:**
- Output line count is greater than verbosity::1 output for the same fixture
- Additional detail is present (e.g., per-project session or entry counts)
- Output contains all information present at verbosity::1
**Pass Criteria:** exit 0 + output is more detailed than verbosity::1
**Source:** [parameter_groups.md#output-control](../../parameter_groups.md#output-control)

### CC-5: v:: alias works in .list same as verbosity::

**Goal:** Verify that the `v::` shorthand alias produces identical output to the full `verbosity::` parameter.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with at least one project present.
**Command:** `clg .list v::0` (compare against `clg .list verbosity::0`)
**Expected Output:** Identical output to `clg .list verbosity::0`.
**Verification:**
- Output of `clg .list v::0` is byte-for-byte identical to `clg .list verbosity::0`
- No error or warning when using the `v::` alias
- Alias accepted without argument error
**Pass Criteria:** exit 0 + output identical to the long-form version
**Source:** [parameter_groups.md#output-control](../../parameter_groups.md#output-control)

### CC-6: verbosity level does not affect which results are returned

**Goal:** Verify that changing verbosity affects output format only, not the result set.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with exactly 3 projects.
**Command:** `clg .list verbosity::0` and `clg .list verbosity::3`
**Expected Output:** Both commands return all 3 projects; only the format of each entry differs.
**Verification:**
- Project count is identical in both outputs
- All projects present at verbosity::0 are also present at verbosity::3
- No extra or missing projects when verbosity changes
**Pass Criteria:** exit 0 + same result set at all verbosity levels
**Source:** [parameter_groups.md#output-control](../../parameter_groups.md#output-control)
