# Parameter :: `--expect`

Edge case coverage for the `--expect` parameter. See [030_expect.md](../../../../docs/cli/param/030_expect.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Output matches → exit 0 | Behavioral Divergence |
| EC-2 | Output does not match, strategy=fail → exit 3 | Behavioral Divergence |
| EC-3 | Case-insensitive matching ("YES" matches "yes") | Edge Case |
| EC-4 | Whitespace trimmed ("yes\n" matches "yes") | Edge Case |
| EC-5 | `--expect` in interactive mode (no message) → silently ignored | Edge Case |
| EC-6 | `--help` output contains `--expect` | Documentation |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-2)
- Edge Case: 3 tests (EC-3, EC-4, EC-5)
- Documentation: 1 test (EC-6)

**Total:** 6 edge cases

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `t01_expect_match_exits_0` | `expect_validation_test.rs` |
| EC-2 | `t02_expect_mismatch_default_fail_exits_3` | `expect_validation_test.rs` |
| EC-3 | `t03_expect_case_insensitive_match` | `expect_validation_test.rs` |
| EC-4 | `t04_expect_whitespace_trimmed` | `expect_validation_test.rs` |
| EC-5 | `t05_expect_dry_run_exits_0` | `expect_validation_test.rs` |
| EC-6 | `t06_help_lists_all_expect_params` | `expect_validation_test.rs` |

---

### EC-1: Output matches → exit 0

- **Given:** claude returns exactly "yes" (trimmed); `--expect "yes|no"` set
- **When:** `clr -p --expect "yes|no" "answer yes or no"` (mocked to return "yes")
- **Then:** Exit 0; no mismatch diagnostic emitted
- **Exit:** 0
- **Source:** [030_expect.md](../../../../docs/cli/param/030_expect.md)
- **Commands:** run, ask

---

### EC-2: Output does not match, strategy=fail → exit 3

- **Given:** claude returns "maybe"; `--expect "yes|no"` with default strategy (fail)
- **When:** `clr -p --expect "yes|no" "answer yes or no"` (mocked to return "maybe")
- **Then:** Exit 3
- **Exit:** 3
- **Source:** [030_expect.md](../../../../docs/cli/param/030_expect.md)
- **Commands:** run, ask

---

### EC-3: Case-insensitive matching

- **Given:** claude returns "YES"; `--expect "yes|no"` set
- **When:** mocked to return "YES"
- **Then:** Exit 0 (matches "yes" case-insensitively)
- **Exit:** 0
- **Source:** [030_expect.md](../../../../docs/cli/param/030_expect.md)
- **Commands:** run, ask

---

### EC-4: Whitespace trimmed

- **Given:** claude returns "yes\n"; `--expect "yes|no"` set
- **When:** mocked to return "yes\n"
- **Then:** Exit 0 (trailing newline trimmed before comparison)
- **Exit:** 0
- **Source:** [030_expect.md](../../../../docs/cli/param/030_expect.md)
- **Commands:** run, ask

---

### EC-5: Interactive mode → silently ignored

- **Given:** `--expect "yes|no"` set but no message and no `--print` (interactive mode)
- **When:** dry-run (cannot invoke live claude in tests)
- **Then:** `--expect` has no effect; assembled command does not forward `--expect` to subprocess
- **Exit:** 0
- **Source:** [030_expect.md](../../../../docs/cli/param/030_expect.md)
- **Commands:** run, ask

---

### EC-6: `--help` lists `--expect`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--expect`
- **Exit:** 0
- **Source:** [command/02_help.md](../../../../docs/cli/command/02_help.md)
- **Commands:** run, ask
