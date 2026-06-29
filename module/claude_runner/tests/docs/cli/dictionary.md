# CLI Dictionary: Vocabulary Consistency Tests

Term accuracy and completeness checks for the CLI dictionary at
[`docs/cli/dictionary.md`](../../../docs/cli/dictionary.md).

**Source:** [dictionary.md](../../../docs/cli/dictionary.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| DT-1 | Commands section lists all 8 subcommands | Completeness |
| DT-2 | Modes section lists all 10 behavioral modes | Completeness |
| DT-3 | `help` canonical form → `clr help` exits 0 with usage | Accuracy |
| DT-4 | `dry-run` mode definition → no subprocess spawned | Accuracy |
| DT-5 | `last-wins` architecture term → final flag value wins | Accuracy |
| DT-6 | `ultrathink suffix` definition → appended to message text | Accuracy |

## Test Coverage Summary

- Completeness: 2 tests (DT-1, DT-2)
- Accuracy: 4 tests (DT-3, DT-4, DT-5, DT-6)

**Total:** 6 dictionary checks

## Test Cases

---

### DT-1: Commands section lists all 8 subcommands

- **Given:** `docs/cli/dictionary.md` Commands section
- **When:** inspect entries in the Commands table
- **Then:** exactly 8 entries present: `run`, `ask`, `isolated`, `refresh`, `ps`, `kill`, `tools`, `help`; no entry missing; no extra entries; each name matches the corresponding file under `docs/cli/command/`
- **Exit:** n/a (static doc check)
- **Source:** [dictionary.md](../../../docs/cli/dictionary.md), [command/](../../../docs/cli/command/readme.md)

---

### DT-2: Modes section lists all 10 behavioral modes

- **Given:** `docs/cli/dictionary.md` Modes section
- **When:** inspect entries in the Modes table
- **Then:** exactly 10 entries present: `interactive mode`, `print mode`, `dry-run`, `new session`, `ultrathink suffix`, `credential-isolated mode`, `fence stripping`, `standalone mode`, `nested-agent mode`, `credential refresh mode`
- **Exit:** n/a (static doc check)
- **Source:** [dictionary.md](../../../docs/cli/dictionary.md)

---

### DT-3: `help` canonical form → `clr help` exits 0 with usage

- **Given:** clean environment; dictionary `help` entry states canonical form `clr help`; aliases `--help` / `-h` stated
- **When:** `clr help`
- **Then:** exit 0; stdout contains usage information; output identical to `clr --help` and `clr -h`
- **Exit:** 0
- **Source:** [dictionary.md](../../../docs/cli/dictionary.md), [command/04_help.md](../../../docs/cli/command/04_help.md)

---

### DT-4: `dry-run` mode definition → no subprocess spawned

- **Given:** clean environment; dictionary `dry-run` entry states "prints assembled command without executing it"
- **When:** `clr --dry-run "test message"`
- **Then:** exit 0; stdout contains assembled command line; no claude subprocess is spawned; output does not contain subprocess execution artifacts
- **Exit:** 0
- **Source:** [dictionary.md](../../../docs/cli/dictionary.md), [param/011_dry_run.md](../../../docs/cli/param/011_dry_run.md)

---

### DT-5: `last-wins` architecture term → final flag value wins

- **Given:** clean environment; dictionary `last-wins` entry states "when a flag appears multiple times, the last occurrence takes effect"
- **When:** `clr --model haiku --model sonnet --dry-run "x"`
- **Then:** assembled command reflects `sonnet` (last value), not `haiku` (first value); no error for duplicate flag
- **Exit:** 0
- **Source:** [dictionary.md](../../../docs/cli/dictionary.md), [param/03_model.md](../../../docs/cli/param/03_model.md)

---

### DT-6: `ultrathink suffix` definition → appended to message text

- **Given:** clean environment; dictionary `ultrathink suffix` entry states `"\n\nultrathink"` is appended after every message before it is sent to the subprocess; default-on, suppressed with `--no-ultrathink`
- **When:** `clr --dry-run "fix the bug"`
- **Then:** assembled command argument list ends with `"fix the bug\n\nultrathink"`; confirm suffix absent when `--no-ultrathink` is added to same invocation
- **Exit:** 0
- **Source:** [dictionary.md](../../../docs/cli/dictionary.md), [param/014_no_ultrathink.md](../../../docs/cli/param/014_no_ultrathink.md)
