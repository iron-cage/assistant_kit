# Parameter :: `--keep-claudecode`

Edge case coverage for the `--keep-claudecode` flag. See [027_keep_claudecode.md](../../../../docs/cli/param/027_keep_claudecode.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Default (no `--keep-claudecode`) → CLAUDECODE absent from subprocess env | Behavioral Divergence |
| EC-2 | `--keep-claudecode` → CLAUDECODE preserved in subprocess env | Behavioral Divergence |
| EC-3 | `--dry-run` shows unset_claudecode state in describe output | Edge Case |
| EC-4 | `--help` output contains `--keep-claudecode` | Documentation |
| EC-5 | `--keep-claudecode` + `--model` → both applied, no conflict | Interaction |
| EC-6 | Default when `CLAUDECODE` not set in parent → no-op (subprocess env unchanged) | Edge Case |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-2)
- Edge Case: 2 tests (EC-3, EC-6)
- Interaction: 1 test (EC-5)
- Documentation: 1 test (EC-4)

**Total:** 6 edge cases

---

### EC-1: Default → CLAUDECODE removed from subprocess env

- **Given:** parent env has `CLAUDECODE=1` set; no `--keep-claudecode` flag
- **When:** `clr --dry-run "task"` (env inspection via fake claude or env-dump test)
- **Then:** The assembled command or subprocess environment does not contain `CLAUDECODE`
- **Exit:** 0
- **Source:** [--keep-claudecode](../../../../docs/cli/param/027_keep_claudecode.md)
- **Commands:** run, ask

---

### EC-2: `--keep-claudecode` → CLAUDECODE preserved

- **Given:** parent env has `CLAUDECODE=1` set
- **When:** `clr --keep-claudecode --dry-run "task"` (env inspection via fake claude)
- **Then:** The subprocess environment contains `CLAUDECODE=1`
- **Exit:** 0
- **Source:** [--keep-claudecode](../../../../docs/cli/param/027_keep_claudecode.md)
- **Commands:** run, ask

---

### EC-3: Dry-run shows unset_claudecode state in describe output

- **Given:** clean environment
- **When:** `clr --dry-run "task"` vs `clr --dry-run --keep-claudecode "task"`
- **Then:** Default (`--dry-run`): last line of stdout starts with `"env -u CLAUDECODE claude ..."` (WYSIWYG: CLAUDECODE removal visible). With `--keep-claudecode`: last line starts with `"claude ..."` (no `env -u CLAUDECODE` prefix — removal suppressed)
- **Exit:** 0
- **Source:** [--keep-claudecode](../../../../docs/cli/param/027_keep_claudecode.md)
- **Commands:** run, ask

---

### EC-4: `--help` lists `--keep-claudecode`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--keep-claudecode`
- **Exit:** 0
- **Source:** [command/04_help.md](../../../../docs/cli/command/04_help.md)
- **Commands:** run, ask

---

### EC-5: `--keep-claudecode` + `--model` → both applied

- **Given:** parent env has `CLAUDECODE=1`; clean environment otherwise
- **When:** `clr --dry-run --keep-claudecode --model sonnet "task"`
- **Then:** Assembled command contains `--model sonnet`; subprocess env retains `CLAUDECODE`
- **Exit:** 0
- **Source:** [--keep-claudecode](../../../../docs/cli/param/027_keep_claudecode.md)
- **Commands:** run, ask

---

### EC-6: No CLAUDECODE in parent → flag is no-op

- **Given:** parent env does NOT have `CLAUDECODE` set
- **When:** `clr --dry-run "task"`
- **Then:** Subprocess environment does not contain `CLAUDECODE` regardless of `--keep-claudecode`
- **Exit:** 0
- **Source:** [--keep-claudecode](../../../../docs/cli/param/027_keep_claudecode.md)
- **Commands:** run, ask
