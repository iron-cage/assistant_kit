# Parameter :: `.help`

Edge case tests for the `.help` parameter. Tests validate help trigger behavior, output stream, and stability.

**Source:** [params.md](../../../../docs/cli/params.md#parameter--10-help)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `.help` present → help output shown, exit 0 | Happy Path |
| EC-2 | `.help` absent → normal command processing (not help) | Default |
| EC-3 | `.help` anywhere in argv triggers help | Anywhere-In-Argv |
| EC-4 | `.help` output goes to stdout; stderr is empty | Output Stream |
| EC-5 | `.help` output is stable across repeated invocations | Stability |
| EC-6 | `.help` does not appear in its own command listing | Visibility |

## Test Coverage Summary

- Happy Path: 1 test (EC-1)
- Default: 1 test (EC-2)
- Anywhere-In-Argv: 1 test (EC-3)
- Output Stream: 1 test (EC-4)
- Stability: 1 test (EC-5)
- Visibility: 1 test (EC-6)

**Total:** 6 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

## Test Cases
---

### EC-1: `.help` → help output, exit 0:

- **Given:** clean environment
- **When:** `cm .help`
- **Then:** Help listing printed to stdout; exit 0
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--10-help)
---

### EC-2: Without `.help` → normal command processing:

- **Given:** clean environment
- **When:** `cm .status`
- **Then:** `.status` output shown (not help); exit 0
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--10-help)
---

### EC-3: `.help` anywhere in argv triggers help:

- **Given:** clean environment
- **When:** `cm .status .help`
- **Then:** Help output shown; `.status` command NOT executed
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--10-help)
---

### EC-4: `.help` output to stdout; stderr empty:

- **Given:** clean environment
- **When:** `cm .help`
- **Then:** stdout is non-empty; stderr is empty
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--10-help)
---

### EC-5: `.help` output stable across invocations:

- **Given:** clean environment
- **When:** `cm .help` (run 3 times)
- **Then:** All 3 stdout captures are byte-identical
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--10-help)
---

### EC-6: `.help` not in its own listing:

- **Given:** clean environment
- **When:** `cm .help`
- **Then:** The command listing does NOT contain a `.help` entry
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--10-help)
