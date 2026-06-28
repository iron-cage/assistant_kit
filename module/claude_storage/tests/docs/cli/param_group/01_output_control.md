# Parameter Group :: Output Control

Interaction tests for the Output Control group (`show_stat::`, `show_tokens::`, `show_tree::`). Tests verify independent boolean toggle semantics and cross-command consistency.

**Source:** [param_group/01_output_control.md](../../../../docs/cli/param_group/01_output_control.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | All toggles off (defaults) — no optional sections | Baseline |
| CC-2 | show_stat::1 only — statistics footer without token section | Independent Toggle |
| CC-3 | show_tokens::1 only — token section without statistics footer | Independent Toggle |
| CC-4 | show_stat::1 + show_tokens::1 — both sections appear | Toggle Combination |
| CC-5 | show_tokens::1 in .status — same section in different command | Cross-Command |
| CC-6 | Toggles do not affect core content returned | Non-Interference |

## Test Coverage Summary

- Baseline: 1 test (CC-1)
- Independent Toggle: 2 tests (CC-2, CC-3)
- Toggle Combination: 1 test (CC-4)
- Cross-Command: 1 test (CC-5)
- Non-Interference: 1 test (CC-6)

**Total:** 6 cross-command cases

## Test Cases

---

### CC-1: All toggles off (defaults) — no optional sections

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .show` (no toggle params — all default to 0)
- **Then:** Standard session content without statistics footer, token usage section, or tree format
- **Exit:** 0
- **Source:** [param_group/01_output_control.md](../../../../docs/cli/param_group/01_output_control.md)

---

### CC-2: show_stat::1 only — statistics footer without token section

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .show show_stat::1`
- **Then:** Output includes statistics footer (entry count, user/assistant breakdown, timestamp range) but no token usage section
- **Exit:** 0
- **Source:** [param_group/01_output_control.md](../../../../docs/cli/param_group/01_output_control.md)

---

### CC-3: show_tokens::1 only — token section without statistics footer

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .show show_tokens::1`
- **Then:** Output includes token usage breakdown (input, output, cache read, cache creation) but no statistics footer
- **Exit:** 0
- **Source:** [param_group/01_output_control.md](../../../../docs/cli/param_group/01_output_control.md)

---

### CC-4: show_stat::1 + show_tokens::1 — both sections appear

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .show show_stat::1 show_tokens::1`
- **Then:** Output includes both statistics footer and token usage section
- **Exit:** 0
- **Source:** [param_group/01_output_control.md](../../../../docs/cli/param_group/01_output_control.md)

---

### CC-5: show_tokens::1 in .status — same section in different command

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .status show_tokens::1`
- **Then:** Token usage section present in .status output (triggers full JSONL parse)
- **Exit:** 0
- **Source:** [param_group/01_output_control.md](../../../../docs/cli/param_group/01_output_control.md)

---

### CC-6: Toggles do not affect core content returned

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .show show_stat::1 show_tokens::1` vs `clg .show`
- **Then:** Core session content identical in both; only the appended optional sections differ
- **Exit:** 0
- **Source:** [param_group/01_output_control.md](../../../../docs/cli/param_group/01_output_control.md)
