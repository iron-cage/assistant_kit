# Parameter :: `--columns`

Edge case coverage for the `--columns` parameter. See [059_columns.md](../../../../docs/cli/param/059_columns.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `clr ps --columns pid,path,task` shows exactly 3 columns in specified order | Behavioral |
| EC-2 | `clr ps --columns bogus` exits 1 with error listing valid keys | Validation |
| EC-3 | `CLR_PS_COLUMNS=pid,elapsed clr ps` shows PID and Elapsed only (env fallback) | Env Var |
| EC-4 | `clr ps --columns pid,path` with `CLR_PS_COLUMNS=pid,elapsed` → CLI wins | CLI-wins |
| EC-5 | `clr ps --columns` with `--wide` → `--columns` wins | Precedence |
| EC-6 | `clr ps --columns idx,pid,mode,cmd,binary` shows optional columns | Behavioral |
| EC-7 | `clr ps` with no `--columns` shows 9 default columns (including Mode) | Default |
| EC-8 | `clr ps --help` output contains `--columns` | Documentation |
| EC-9 | `idx` column counter reflects visible row number (1-based) after `--mode` filtering | Interaction |
| EC-10 | `clr ps --help` lists `idx` and `cmd` as column keys, not `num` or `command` (BUG-303 regression) | Documentation |
| EC-11 | `clr tools --columns name,category` shows exactly 2 tools columns | Behavioral |
| EC-12 | `clr tools --columns badkey` exits 1 with error listing tools' valid keys | Validation |
| EC-13 | `clr tools` with no `--columns` shows all 4 default columns (`idx`, `name`, `category`, `desc`) | Default |

## Test Coverage Summary

- Behavioral: 3 tests (EC-1, EC-6, EC-11)
- Validation: 2 tests (EC-2, EC-12)
- Env Var: 1 test (EC-3)
- CLI-wins: 1 test (EC-4)
- Precedence: 1 test (EC-5)
- Default: 2 tests (EC-7, EC-13)
- Documentation: 2 tests (EC-8, EC-10)
- Interaction: 1 test (EC-9)

**Total:** 13 edge cases

---

### EC-1: Custom column subset in specified order

- **Setup:** ≥1 fake `claude` process running
- **Command:** `clr ps --columns pid,path,task`
- **Expected behavior:** Exit 0; stdout contains `PID`, `Absolute Path`, `Task` headers; stdout does NOT contain `CPU%`, `RAM`, `State`, `Elapsed` headers
- **Exit:** 0
- **Source:** [059_columns.md](../../../../docs/cli/param/059_columns.md)

---

### EC-2: Unknown column key → exit 1

- **Command:** `clr ps --columns bogus`
- **Expected behavior:** Exit 1; stderr contains error message listing valid column keys
- **Exit:** 1
- **Source:** [059_columns.md](../../../../docs/cli/param/059_columns.md)

---

### EC-3: `CLR_PS_COLUMNS` env var fallback

- **Setup:** ≥1 fake `claude` process running
- **Command:** `clr ps` with `CLR_PS_COLUMNS=pid,elapsed` in env
- **Expected behavior:** Exit 0; stdout contains `PID` and `Elapsed`; stdout does NOT contain `CPU%`, `RAM`, `Task`
- **Exit:** 0
- **Source:** [059_columns.md](../../../../docs/cli/param/059_columns.md)

---

### EC-4: CLI `--columns` wins over `CLR_PS_COLUMNS`

- **Setup:** ≥1 fake `claude` process running
- **Command:** `clr ps --columns pid,path` with `CLR_PS_COLUMNS=pid,elapsed` in env
- **Expected behavior:** Exit 0; stdout contains `PID` and `Absolute Path`; stdout does NOT contain `Elapsed`
- **Exit:** 0
- **Source:** [059_columns.md](../../../../docs/cli/param/059_columns.md)

---

### EC-5: `--columns` overrides `--wide`

- **Setup:** ≥1 fake `claude` process running
- **Command:** `clr ps --columns pid,task --wide`
- **Expected behavior:** Exit 0; stdout contains `PID` and `Task`; stdout does NOT contain `Mode`, `Command`, `Binary` (columns wins over wide)
- **Exit:** 0
- **Source:** [059_columns.md](../../../../docs/cli/param/059_columns.md)

---

### EC-6: Optional columns displayed when requested

- **Setup:** ≥1 fake `claude` process running
- **Command:** `clr ps --columns idx,pid,mode,cmd,binary`
- **Expected behavior:** Exit 0; stdout contains `#`, `PID`, `Mode`, `Command`, `Binary` headers
- **Exit:** 0
- **Source:** [059_columns.md](../../../../docs/cli/param/059_columns.md)

---

### EC-7: Default columns shown without `--columns`

- **Setup:** ≥1 fake `claude` process running
- **Command:** `clr ps` (no `--columns` flag)
- **Expected behavior:** Exit 0; stdout contains `PID`, `Elapsed`, `CPU%`, `RAM`, `State`, `Mode`, `Absolute Path`, `Task`; stdout does NOT contain `Command`, `Binary` headers
- **Exit:** 0
- **Source:** [059_columns.md](../../../../docs/cli/param/059_columns.md)

---

### EC-8: Help output contains `--columns`

- **Command:** `clr ps --help`
- **Expected behavior:** Exit 0; stdout contains `--columns`
- **Exit:** 0
- **Source:** [059_columns.md](../../../../docs/cli/param/059_columns.md)

---

### EC-9: `idx` counter reflects visible rows after filtering

- **Setup:** Spawn 2 fake `claude` processes: one print-mode, one interactive
- **Command:** `clr ps --mode print --columns idx,pid,task`
- **Expected behavior:** Exit 0; only print-mode row shown; `#` column value is `1` (not original row index)
- **Exit:** 0
- **Source:** [059_columns.md](../../../../docs/cli/param/059_columns.md)

---

### EC-10: Help output uses correct column key names (BUG-303 regression)

- **Command:** `clr ps --help`
- **Expected behavior:** Exit 0; stdout contains `  idx ` and `  cmd `; stdout does NOT contain `  num ` or `  command ` as column key names; DEFAULT COLUMNS line starts with `idx`
- **Exit:** 0
- **Regression for:** BUG-303 (`print_ps_help()` originally used `num`/`command` key names diverging from `COLUMN_KEYS`)
- **Source:** [059_columns.md](../../../../docs/cli/param/059_columns.md)

---

### EC-11: `tools` custom column subset

- **Command:** `clr tools --columns name,category`
- **Expected behavior:** Exit 0; stdout header row contains `Tool` and `Category`; stdout does NOT contain `Description`
- **Exit:** 0
- **Source:** [059_columns.md](../../../../docs/cli/param/059_columns.md)

---

### EC-12: `tools` unknown column key → exit 1

- **Command:** `clr tools --columns badkey`
- **Expected behavior:** Exit 1; stderr contains error message listing valid column keys (`idx`, `name`, `category`, `desc`)
- **Exit:** 1
- **Source:** [059_columns.md](../../../../docs/cli/param/059_columns.md)

---

### EC-13: `tools` default columns shown without `--columns`

- **Command:** `clr tools` (no `--columns` flag)
- **Expected behavior:** Exit 0; stdout contains `#`, `Tool`, `Category`, `Description` headers (all 4 available columns — `tools` has no narrower default to expand from)
- **Exit:** 0
- **Source:** [059_columns.md](../../../../docs/cli/param/059_columns.md)
