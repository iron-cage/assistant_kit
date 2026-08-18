# Parameter :: `--category`

Edge case coverage for the `--category` parameter. See [079_category.md](../../../../docs/cli/param/079_category.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `clr tools --category Web` shows only category-matching tools (substring) | Behavioral |
| EC-2 | `clr tools --category web` (lowercase) matches same tools as `Web` | Behavioral |
| EC-3 | `clr tools --category "File Operations"` — quoted multi-word category matches | Behavioral |
| EC-4 | `clr tools --category doesnotexist` — zero matches, exit 0, no rows | Boundary |
| EC-5 | `clr tools --name cron --category Scheduling` — combines with `--name` (AND) | Interaction |
| EC-6 | `clr tools` with no `--category` flag shows all tools (default = no filter) | Default |
| EC-7 | `clr tools --help` output contains `--category` | Documentation |

## Test Coverage Summary

- Behavioral: 3 tests (EC-1, EC-2, EC-3)
- Boundary: 1 test (EC-4)
- Interaction: 1 test (EC-5)
- Default: 1 test (EC-6)
- Documentation: 1 test (EC-7)

**Total:** 7 edge cases

## Implementation Map

All cases live in `tests/tools_command_test.rs`, whose own numbering is `IT-N` (from
`command/08_tools.md`). EC identifiers are cross-references and are always written with
their `079_category.md` prefix at the call site, because `078_name.md` and `080_value.md`
number their own cases from `EC-1` too.

| EC | Test function |
|----|---------------|
| EC-1 | `it11_tools_category_filter` |
| EC-2 | `it23_tools_category_filter_case_insensitive` |
| EC-3 | `it24_tools_category_multi_word_value` |
| EC-4 | `it25_tools_category_zero_match_not_error` |
| EC-5 | `it12_tools_name_and_category_and_logic` |
| EC-6 | `it3_tools_lists_categories` |
| EC-7 | `it29_tools_help_documents_filter_flags` |

---

### EC-1: Substring match on tool category

- **Command:** `clr tools --category Web`
- **Expected behavior:** Exit 0; stdout contains `WebFetch` and `WebSearch`; stdout does NOT contain `Bash`
- **Exit:** 0
- **Source:** [079_category.md](../../../../docs/cli/param/079_category.md)

---

### EC-2: Case-insensitive match

- **Command:** `clr tools --category web`
- **Expected behavior:** Exit 0; stdout is identical to `clr tools --category Web` — the same tools are shown
- **Exit:** 0
- **Source:** [079_category.md](../../../../docs/cli/param/079_category.md)

---

### EC-3: Quoted multi-word category matches

- **Command:** `clr tools --category "File Operations"`
- **Expected behavior:** Exit 0; stdout contains the tools carrying the "File Operations" category (e.g. `Read`, `Write`, `Edit`)
- **Exit:** 0
- **Source:** [079_category.md](../../../../docs/cli/param/079_category.md)

---

### EC-4: Zero matches is not an error

- **Command:** `clr tools --category doesnotexist`
- **Expected behavior:** Exit 0; stdout contains the table heading/caption but no tool rows
- **Exit:** 0
- **Source:** [079_category.md](../../../../docs/cli/param/079_category.md)

---

### EC-5: Combines with `--name` using AND logic

- **Command:** `clr tools --name cron --category Scheduling`
- **Expected behavior:** Exit 0; stdout contains `CronCreate`, `CronDelete`, `CronList`; stdout does NOT contain `RemoteTrigger` or `ScheduleWakeup` (same category, name does not match "cron")
- **Exit:** 0
- **Source:** [079_category.md](../../../../docs/cli/param/079_category.md)

---

### EC-6: Default shows all tools

- **Command:** `clr tools` (no `--category` flag)
- **Expected behavior:** Exit 0; stdout contains tools from multiple distinct categories — no filtering applied
- **Exit:** 0
- **Source:** [079_category.md](../../../../docs/cli/param/079_category.md)

---

### EC-7: Help output contains `--category`

- **Command:** `clr tools --help`
- **Expected behavior:** Exit 0; stdout contains `--category`
- **Exit:** 0
- **Source:** [079_category.md](../../../../docs/cli/param/079_category.md)
