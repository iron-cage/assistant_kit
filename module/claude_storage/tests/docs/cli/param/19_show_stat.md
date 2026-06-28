# Parameter :: `show_stat::`

Edge case tests for the `show_stat::` parameter. Tests validate boolean acceptance, default behavior, and mode interaction.

**Source:** [param/19_show_stat.md](../../../../docs/cli/param/19_show_stat.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Value 0 accepted (default, no footer) | Valid Input |
| EC-2 | Value 1 accepted (statistics footer shown) | Valid Input |
| EC-3 | Non-boolean value rejected | Type Validation |
| EC-4 | Omitted uses default of 0 | Default |
| EC-5 | No effect in metadata mode (metadata::1) | Mode Interaction |
| EC-6 | Combined with show_tokens::1 | Parameter Interaction |

## Test Coverage Summary

- Valid Input: 2 tests (EC-1, EC-2)
- Type Validation: 1 test (EC-3)
- Default: 1 test (EC-4)
- Mode Interaction: 1 test (EC-5)
- Parameter Interaction: 1 test (EC-6)

**Total:** 6 edge cases

**Behavioral Divergence Pair:** EC-1 (show_stat::0, no footer) ↔ EC-2 (show_stat::1, footer shown)

## Test Cases

---

### EC-1: Value 0 accepted (default, no footer)

- **Commands:** `.show`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .show show_stat::0`
- **Then:** Session content displayed without statistics footer
- **Exit:** 0
- **Source:** [param/19_show_stat.md](../../../../docs/cli/param/19_show_stat.md)

---

### EC-2: Value 1 accepted (statistics footer shown)

- **Commands:** `.show`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .show show_stat::1`
- **Then:** Session content displayed with statistics footer (entry count, user/assistant breakdown, timestamp range)
- **Exit:** 0
- **Source:** [param/19_show_stat.md](../../../../docs/cli/param/19_show_stat.md)

---

### EC-3: Non-boolean value rejected

- **Commands:** `.show`
- **Given:** clean environment
- **When:** `clg .show show_stat::yes`
- **Then:** Error message indicating boolean expected (0 or 1)
- **Exit:** 1
- **Source:** [param/19_show_stat.md](../../../../docs/cli/param/19_show_stat.md)

---

### EC-4: Omitted uses default of 0

- **Commands:** `.show`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .show`
- **Then:** Session content displayed without statistics footer (same as show_stat::0)
- **Exit:** 0
- **Source:** [param/19_show_stat.md](../../../../docs/cli/param/19_show_stat.md)

---

### EC-5: No effect in metadata mode (metadata::1)

- **Commands:** `.show`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .show show_stat::1 metadata::1`
- **Then:** Metadata mode output (structured fields); statistics footer not appended
- **Exit:** 0
- **Source:** [param/19_show_stat.md](../../../../docs/cli/param/19_show_stat.md)

---

### EC-6: Combined with show_tokens::1

- **Commands:** `.show`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .show show_stat::1 show_tokens::1`
- **Then:** Output includes both statistics footer and token usage section
- **Exit:** 0
- **Source:** [param/19_show_stat.md](../../../../docs/cli/param/19_show_stat.md)
