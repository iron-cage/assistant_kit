# Parameter :: `--inspect`

Edge case coverage for the `--inspect` flag. See [069_inspect.md](../../../../docs/cli/param/069_inspect.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `clr ps --inspect` shows key:value blocks, not table | Behavioral |
| EC-2 | Inspect block contains all 12 attribute keys | Behavioral |
| EC-3 | `clr ps --inspect --pid <pid>` shows only that session's block | Interaction |
| EC-4 | `clr ps --inspect --mode print` shows only print-mode blocks | Interaction |
| EC-5 | `clr ps --inspect --columns pid` — `--columns` ignored in inspect mode | Precedence |
| EC-6 | `clr ps --inspect --wide` — `--wide` ignored in inspect mode | Precedence |
| EC-7 | Inspect mode suppresses Queued CLR Processes table | Behavioral |
| EC-8 | `clr ps --inspect` with no active sessions shows empty-state message | Behavioral |
| EC-9 | `clr ps --help` output contains `--inspect` | Documentation |
| EC-10 | `clr tools --inspect` shows key:value blocks with `idx`/`name`/`category`/`desc` | Behavioral |
| EC-11 | `clr tools --category Web --inspect` — inspect mode respects `--category` filter | Interaction |
| EC-12 | `clr tools --columns name --inspect` — `--columns` ignored in tools inspect mode | Precedence |
| EC-13 | `clr tools --value name --inspect` exits 1 (mutually exclusive) | Validation |

## Test Coverage Summary

- Behavioral: 5 tests (EC-1, EC-2, EC-7, EC-8, EC-10)
- Interaction: 3 tests (EC-3, EC-4, EC-11)
- Precedence: 3 tests (EC-5, EC-6, EC-12)
- Documentation: 1 test (EC-9)
- Validation: 1 test (EC-13)

**Total:** 13 edge cases

---

### EC-1: Inspect mode produces key:value blocks, not a table

- **Setup:** ≥1 fake `claude` process running
- **Command:** `clr ps --inspect`
- **Expected behavior:** Exit 0; stdout contains `pid:` and `mode:` and `path:` as key:value lines; stdout does NOT contain `Elapsed` or `CPU%` table header text
- **Exit:** 0
- **Source:** [069_inspect.md](../../../../docs/cli/param/069_inspect.md)

---

### EC-2: Inspect block contains all 12 attribute keys

- **Setup:** ≥1 fake `claude` process running
- **Command:** `clr ps --inspect`
- **Expected behavior:** Exit 0; stdout contains all of: `pid:`, `mode:`, `elapsed:`, `cpu:`, `ram:`, `state:`, `path:`, `task:`, `binary:`, `cmd:`, `cmdline:`, `started:`
- **Exit:** 0
- **Source:** [069_inspect.md](../../../../docs/cli/param/069_inspect.md)

---

### EC-3: `--inspect --pid` shows only the specified session block

- **Setup:** ≥2 fake `claude` processes running (PID A, PID B)
- **Command:** `clr ps --inspect --pid <PID-A>`
- **Expected behavior:** Exit 0; stdout contains PID A value; stdout does NOT contain PID B value; output is key:value format (not table)
- **Exit:** 0
- **Source:** [069_inspect.md](../../../../docs/cli/param/069_inspect.md)

---

### EC-4: `--inspect --mode` filters to matching execution mode

- **Setup:** 2 fake processes: PID A (interactive, no `-p` arg), PID B (print-mode, has `-p` arg)
- **Command:** `clr ps --inspect --mode print`
- **Expected behavior:** Exit 0; stdout shows inspect block containing PID B value; stdout does NOT contain PID A value
- **Exit:** 0
- **Source:** [069_inspect.md](../../../../docs/cli/param/069_inspect.md)

---

### EC-5: `--columns` is ignored in inspect mode

- **Setup:** ≥1 fake `claude` process running
- **Command:** `clr ps --inspect --columns pid`
- **Expected behavior:** Exit 0; stdout contains ALL 12 attribute keys (`pid:`, `mode:`, `elapsed:`, `cpu:`, `ram:`, `state:`, `path:`, `task:`, `binary:`, `cmd:`, `cmdline:`, `started:`); not only `pid:`
- **Exit:** 0
- **Source:** [069_inspect.md](../../../../docs/cli/param/069_inspect.md)

---

### EC-6: `--wide` is ignored in inspect mode

- **Setup:** ≥1 fake `claude` process running
- **Command:** `clr ps --inspect --wide`
- **Expected behavior:** Exit 0; output is key:value format with all 12 attributes; identical to `clr ps --inspect` without `--wide`
- **Exit:** 0
- **Source:** [069_inspect.md](../../../../docs/cli/param/069_inspect.md)

---

### EC-7: Inspect mode suppresses Queued CLR Processes table

- **Setup:** ≥1 fake `claude` process running; `CLR_GATE_DIR` set to a temp dir containing ≥1 valid `.json` gate file for a live PID
- **Command:** `clr ps --inspect`
- **Expected behavior:** Exit 0; stdout does NOT contain `Queued` caption text
- **Exit:** 0
- **Source:** [069_inspect.md](../../../../docs/cli/param/069_inspect.md)

---

### EC-8: No active sessions with `--inspect` shows empty-state message

- **Setup:** No active `claude` processes (CLR_PROC_DIR set to empty dir)
- **Command:** `clr ps --inspect`
- **Expected behavior:** Exit 0; stdout contains `No active Claude Code sessions.`
- **Exit:** 0
- **Source:** [069_inspect.md](../../../../docs/cli/param/069_inspect.md)

---

### EC-9: Help output contains `--inspect`

- **Command:** `clr ps --help`
- **Expected behavior:** Exit 0; stdout contains `--inspect`
- **Exit:** 0
- **Source:** [069_inspect.md](../../../../docs/cli/param/069_inspect.md)

---

### EC-10: `tools` inspect mode produces key:value blocks

- **Command:** `clr tools --inspect`
- **Expected behavior:** Exit 0; stdout contains `idx:`, `name:`, `category:`, `desc:` key:value lines; stdout does NOT contain a table header row
- **Exit:** 0
- **Source:** [069_inspect.md](../../../../docs/cli/param/069_inspect.md)

---

### EC-11: `tools` inspect mode respects `--category` filter

- **Command:** `clr tools --category Web --inspect`
- **Expected behavior:** Exit 0; stdout shows inspect blocks only for `WebFetch`/`WebSearch`; stdout does NOT contain a `Bash` block
- **Exit:** 0
- **Source:** [069_inspect.md](../../../../docs/cli/param/069_inspect.md)

---

### EC-12: `tools` `--columns` is ignored in inspect mode

- **Command:** `clr tools --columns name --inspect`
- **Expected behavior:** Exit 0; stdout contains all 4 attribute keys (`idx:`, `name:`, `category:`, `desc:`); not only `name:`
- **Exit:** 0
- **Source:** [069_inspect.md](../../../../docs/cli/param/069_inspect.md)

---

### EC-13: `tools` `--value` and `--inspect` are mutually exclusive

- **Command:** `clr tools --value name --inspect`
- **Expected behavior:** Exit 1; stderr states the two flags cannot be combined
- **Exit:** 1
- **Source:** [069_inspect.md](../../../../docs/cli/param/069_inspect.md)
