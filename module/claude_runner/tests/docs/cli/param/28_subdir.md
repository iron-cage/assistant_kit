# Parameter :: `--subdir`

Edge case coverage for the `--subdir` parameter. See [028_subdir.md](../../../../docs/cli/param/028_subdir.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Default (no `--subdir`) → effective dir equals `--dir` base | Behavioral Divergence |
| EC-2 | `--subdir NAME` → effective dir ends with `/-NAME` | Behavioral Divergence |
| EC-3 | `--subdir .` (explicit identity) → effective dir equals `--dir` base | Edge Case |
| EC-4 | `--help` output contains `--subdir` | Documentation |
| EC-5 | `--subdir NAME` + `--dir PATH` → effective dir is `PATH/-NAME` | Interaction |
| EC-6 | `CLR_SUBDIR=NAME` env var → effective dir ends with `/-NAME` (CLI absent) | Env Var |
| EC-7 | `--subdir NAME` CLI wins over `CLR_SUBDIR=OTHER` env var | CLI-wins |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-2)
- Edge Case: 1 test (EC-3)
- Documentation: 1 test (EC-4)
- Interaction: 1 test (EC-5)
- Env Var: 1 test (EC-6)
- CLI-wins: 1 test (EC-7)

**Total:** 7 edge cases

---

### EC-1: Default (no --subdir) → no /-NAME suffix

- **Given:** No `--subdir` flag; no `CLR_SUBDIR` env var
- **When:** `clr --dry-run "task"`
- **Then:** Dry-run output contains no `/-` path component; no named subdir is appended
- **Exit:** 0
- **Source:** [--subdir](../../../../docs/cli/param/028_subdir.md)
- **Commands:** run, ask

---

### EC-2: --subdir NAME → effective dir ends with /-NAME

- **Given:** No prior `--dir`; cwd is base
- **When:** `clr --subdir build --dry-run "task"`
- **Then:** Dry-run output contains effective dir ending in `/-build`
- **Exit:** 0
- **Source:** [--subdir](../../../../docs/cli/param/028_subdir.md)
- **Commands:** run, ask

---

### EC-3: --subdir . (explicit identity) → no /-NAME suffix

- **Given:** No prior `--dir`; cwd is base
- **When:** `clr --subdir . --dry-run "task"`
- **Then:** Dry-run output contains no `/-` path component; identity (`.`) is a no-op — same output as bare `clr --dry-run "task"`
- **Exit:** 0
- **Source:** [--subdir](../../../../docs/cli/param/028_subdir.md)
- **Commands:** run, ask

---

### EC-4: --help lists --subdir

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--subdir`
- **Exit:** 0
- **Source:** [command/04_help.md](../../../../docs/cli/command/04_help.md)
- **Commands:** run, ask

---

### EC-5: --subdir NAME + --dir PATH → PATH/-NAME

- **Given:** `--dir /tmp/project` and `--subdir debug`
- **When:** `clr --dir /tmp/project --subdir debug --dry-run "task"`
- **Then:** Dry-run output shows effective dir `/tmp/project/-debug`
- **Exit:** 0
- **Source:** [--subdir](../../../../docs/cli/param/028_subdir.md)
- **Commands:** run, ask

---

### EC-6: CLR_SUBDIR=NAME env var → effective dir ends with /-NAME

- **Given:** `CLR_SUBDIR=feature` set; no `--subdir` CLI flag
- **When:** `CLR_SUBDIR=feature clr --dry-run "task"`
- **Then:** Dry-run output contains effective dir ending in `/-feature`
- **Exit:** 0
- **Source:** [--subdir](../../../../docs/cli/param/028_subdir.md)
- **Commands:** run, ask

---

### EC-7: --subdir CLI wins over CLR_SUBDIR env var

- **Given:** `CLR_SUBDIR=envname` set; `--subdir cliname` on CLI
- **When:** `CLR_SUBDIR=envname clr --subdir cliname --dry-run "task"`
- **Then:** Dry-run output contains effective dir ending in `/-cliname`, NOT `/-envname`
- **Exit:** 0
- **Source:** [--subdir](../../../../docs/cli/param/028_subdir.md)
- **Commands:** run, ask
