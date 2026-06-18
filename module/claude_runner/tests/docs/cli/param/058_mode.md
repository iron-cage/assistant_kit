# Parameter :: `--mode`

Edge case coverage for the `--mode` parameter. See [058_mode.md](../../../../docs/cli/param/058_mode.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `clr ps --mode interactive` shows only interactive sessions | Behavioral |
| EC-2 | `clr ps --mode print` shows only print-mode sessions | Behavioral |
| EC-3 | `clr ps --mode all` shows both interactive and print sessions | Behavioral |
| EC-4 | `clr ps --mode bogus` exits 1 with error listing valid values | Validation |
| EC-5 | `CLR_PS_MODE=print clr ps` filters to print-mode sessions (env fallback) | Env Var |
| EC-6 | `clr ps --mode interactive` with `CLR_PS_MODE=print` → CLI wins | CLI-wins |
| EC-7 | `clr ps` with no `--mode` flag shows all sessions (default = `all`) | Default |
| EC-8 | `clr ps --help` output contains `--mode` | Documentation |

## Test Coverage Summary

- Behavioral: 3 tests (EC-1, EC-2, EC-3)
- Validation: 1 test (EC-4)
- Env Var: 1 test (EC-5)
- CLI-wins: 1 test (EC-6)
- Default: 1 test (EC-7)
- Documentation: 1 test (EC-8)

**Total:** 8 edge cases

---

### EC-1: `--mode interactive` shows only interactive sessions

- **Setup:** Spawn 2 fake `claude` processes: one with `--print` arg (print-mode), one without (interactive)
- **Command:** `clr ps --mode interactive`
- **Expected behavior:** Exit 0; output contains the interactive session PID; output does NOT contain the print-mode session PID
- **Exit:** 0
- **Source:** [058_mode.md](../../../../docs/cli/param/058_mode.md)

---

### EC-2: `--mode print` shows only print-mode sessions

- **Setup:** Spawn 2 fake `claude` processes: one with `--print` arg (print-mode), one without (interactive)
- **Command:** `clr ps --mode print`
- **Expected behavior:** Exit 0; output contains the print-mode session PID; output does NOT contain the interactive session PID
- **Exit:** 0
- **Source:** [058_mode.md](../../../../docs/cli/param/058_mode.md)

---

### EC-3: `--mode all` shows both session types

- **Setup:** Spawn 2 fake `claude` processes: one print-mode, one interactive
- **Command:** `clr ps --mode all`
- **Expected behavior:** Exit 0; output contains both PIDs
- **Exit:** 0
- **Source:** [058_mode.md](../../../../docs/cli/param/058_mode.md)

---

### EC-4: `--mode bogus` → exit 1 with error

- **Command:** `clr ps --mode bogus`
- **Expected behavior:** Exit 1; stderr contains error message listing valid values (`all`, `interactive`, `print`)
- **Exit:** 1
- **Source:** [058_mode.md](../../../../docs/cli/param/058_mode.md)

---

### EC-5: `CLR_PS_MODE=print` env var fallback

- **Setup:** Spawn 2 fake `claude` processes: one print-mode, one interactive
- **Command:** `clr ps` with `CLR_PS_MODE=print` in env
- **Expected behavior:** Exit 0; output contains only the print-mode session PID
- **Exit:** 0
- **Source:** [058_mode.md](../../../../docs/cli/param/058_mode.md)

---

### EC-6: CLI `--mode` wins over `CLR_PS_MODE`

- **Setup:** Spawn 2 fake `claude` processes: one print-mode, one interactive
- **Command:** `clr ps --mode interactive` with `CLR_PS_MODE=print` in env
- **Expected behavior:** Exit 0; output contains the interactive session PID; output does NOT contain the print-mode session PID (CLI wins)
- **Exit:** 0
- **Source:** [058_mode.md](../../../../docs/cli/param/058_mode.md)

---

### EC-7: Default mode shows all sessions

- **Setup:** Spawn 2 fake `claude` processes: one print-mode, one interactive
- **Command:** `clr ps` (no `--mode` flag, no `CLR_PS_MODE` env var)
- **Expected behavior:** Exit 0; output contains both session PIDs
- **Exit:** 0
- **Source:** [058_mode.md](../../../../docs/cli/param/058_mode.md)

---

### EC-8: Help output contains `--mode`

- **Command:** `clr ps --help`
- **Expected behavior:** Exit 0; stdout contains `--mode`
- **Exit:** 0
- **Source:** [058_mode.md](../../../../docs/cli/param/058_mode.md)
