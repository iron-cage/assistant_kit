# Parameter :: `show_tokens::`

Edge case tests for the `show_tokens::` parameter. Tests validate boolean acceptance, default behavior, and performance impact awareness.

**Source:** [param/23_show_tokens.md](../../../../docs/cli/param/23_show_tokens.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Value 0 accepted (default, no token section) | Valid Input |
| EC-2 | Value 1 accepted (token usage shown) | Valid Input |
| EC-3 | Non-boolean value rejected | Type Validation |
| EC-4 | Omitted uses default of 0 | Default |
| EC-5 | In .status triggers full JSONL parse | Performance |
| EC-6 | In .show appends token usage to output | Command Interaction |

## Test Coverage Summary

- Valid Input: 2 tests (EC-1, EC-2)
- Type Validation: 1 test (EC-3)
- Default: 1 test (EC-4)
- Performance: 1 test (EC-5)
- Command Interaction: 1 test (EC-6)

**Total:** 6 edge cases

**Behavioral Divergence Pair:** EC-1 (show_tokens::0, fast path) ↔ EC-5 (show_tokens::1 in .status, slow JSONL parse)

## Test Cases

---

### EC-1: Value 0 accepted (default, no token section)

- **Commands:** `.show`, `.status`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .status show_tokens::0`
- **Then:** Output without token usage section; fast filesystem-only stats
- **Exit:** 0
- **Source:** [param/23_show_tokens.md](../../../../docs/cli/param/23_show_tokens.md)

---

### EC-2: Value 1 accepted (token usage shown)

- **Commands:** `.show`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .show show_tokens::1`
- **Then:** Output includes token usage breakdown (input, output, cache read, cache creation tokens)
- **Exit:** 0
- **Source:** [param/23_show_tokens.md](../../../../docs/cli/param/23_show_tokens.md)

---

### EC-3: Non-boolean value rejected

- **Commands:** `.status`
- **Given:** clean environment
- **When:** `clg .status show_tokens::yes`
- **Then:** Error message indicating boolean expected (0 or 1)
- **Exit:** 1
- **Source:** [param/23_show_tokens.md](../../../../docs/cli/param/23_show_tokens.md)

---

### EC-4: Omitted uses default of 0

- **Commands:** `.status`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .status`
- **Then:** Output without token usage section (same as show_tokens::0)
- **Exit:** 0
- **Source:** [param/23_show_tokens.md](../../../../docs/cli/param/23_show_tokens.md)

---

### EC-5: In .status triggers full JSONL parse

- **Commands:** `.status`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .status show_tokens::1`
- **Then:** Output includes token totals; command completes (may be slower than default)
- **Exit:** 0
- **Source:** [param/23_show_tokens.md](../../../../docs/cli/param/23_show_tokens.md)

---

### EC-6: In .show appends token usage to output

- **Commands:** `.show`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .show show_tokens::1`
- **Then:** Session content or metadata output includes token usage section
- **Exit:** 0
- **Source:** [param/23_show_tokens.md](../../../../docs/cli/param/23_show_tokens.md)
