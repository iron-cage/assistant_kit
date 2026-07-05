# Parameter :: `--strip-fences`

Edge case coverage for the `--strip-fences` flag. See [026_strip_fences.md](../../../../docs/cli/param/026_strip_fences.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Stdout with fence pair → fences stripped, content preserved | Behavioral Divergence |
| EC-2 | Default (no `--strip-fences`) → stdout unchanged including fences | Behavioral Divergence |
| EC-3 | Stdout with no fences → passthrough unchanged | Edge Case |
| EC-4 | `--help` output contains `--strip-fences` | Documentation |
| EC-5 | Stdout with language-tagged fence (e.g. ` ```rust `) → stripped | Edge Case |
| EC-6 | Stdout with multiple fence pairs → only outermost removed | Edge Case |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-2)
- Edge Case: 3 tests (EC-3, EC-5, EC-6)
- Documentation: 1 test (EC-4)

**Total:** 6 edge cases

---

### EC-1: Fence pair stripped

- **Given:** fake claude binary that outputs ` ``` `, `content line`, ` ``` `
- **When:** `clr -p --strip-fences "task"` (via fake claude with `--dry-run` not applicable here — use exec test)
- **Then:** Captured stdout is `content line` (fences removed)
- **Exit:** 0
- **Source:** [--strip-fences](../../../../docs/cli/param/026_strip_fences.md)
- **Commands:** run, ask

---

### EC-2: Default → fences preserved

- **Given:** fake claude binary that outputs ` ``` `, `content line`, ` ``` `
- **When:** `clr -p "task"` (no `--strip-fences`)
- **Then:** Captured stdout includes the fence lines unchanged
- **Exit:** 0
- **Source:** [--strip-fences](../../../../docs/cli/param/026_strip_fences.md)
- **Commands:** run, ask

---

### EC-3: No fences → passthrough

- **Given:** fake claude binary that outputs `plain output` (no fences)
- **When:** `clr -p --strip-fences "task"`
- **Then:** Captured stdout is `plain output` unchanged
- **Exit:** 0
- **Source:** [--strip-fences](../../../../docs/cli/param/026_strip_fences.md)
- **Commands:** run, ask

---

### EC-4: `--help` lists `--strip-fences`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--strip-fences`
- **Exit:** 0
- **Source:** [command/02_help.md](../../../../docs/cli/command/02_help.md)
- **Commands:** run, ask

---

### EC-5: Language-tagged fence stripped

- **Given:** fake claude binary that outputs ` ```rust `, `let x = 1;`, ` ``` `
- **When:** `clr -p --strip-fences "task"`
- **Then:** Captured stdout is `let x = 1;` (opening fence with language tag removed)
- **Exit:** 0
- **Source:** [--strip-fences](../../../../docs/cli/param/026_strip_fences.md)
- **Commands:** run, ask

---

### EC-6: Multiple fence pairs → outermost removed

- **Given:** fake claude binary outputs: ` ``` `, `outer`, ` ``` `, `inner`, ` ``` `
- **When:** `clr -p --strip-fences "task"`
- **Then:** Captured stdout contains the content between first ` ``` ` and last ` ``` ` line
- **Exit:** 0
- **Source:** [--strip-fences](../../../../docs/cli/param/026_strip_fences.md)
- **Commands:** run, ask
