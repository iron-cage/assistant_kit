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
| EC-8 | `--subdir ""` (empty string) → identity; no `/-` suffix in dry-run output | Edge Case |
| EC-9 | `--subdir "a/b"` (slash in name) → exit 1; error mentions no separators | Validation |
| EC-10 | `--dry-run --subdir NAME` → exit 0; no directory created on filesystem | Side-effect |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-2)
- Edge Case: 2 tests (EC-3, EC-8)
- Documentation: 1 test (EC-4)
- Interaction: 1 test (EC-5)
- Env Var: 1 test (EC-6)
- CLI-wins: 1 test (EC-7)
- Validation: 1 test (EC-9)
- Side-effect: 1 test (EC-10)

**Total:** 10 edge cases

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
- **Source:** [command/02_help.md](../../../../docs/cli/command/02_help.md)
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

---

### EC-8: --subdir "" (empty string) → identity

- **Given:** No prior `--dir`; cwd is base
- **When:** `clr --subdir "" --dry-run "task"`
- **Then:** Dry-run output contains no `/-` path component; empty string is treated as identity — same output as bare `clr --dry-run "task"` (Fix: BUG-229)
- **Exit:** 0
- **Source:** [--subdir](../../../../docs/cli/param/028_subdir.md)
- **Commands:** run, ask

---

### EC-9: --subdir "a/b" (slash in name) → rejected with error

- **Given:** clean environment
- **When:** `clr --subdir "a/b" "task"`
- **Then:** Exit 1; stderr contains error message about no `/` separators; no directory is created (Fix: BUG-230)
- **Exit:** 1
- **Source:** [--subdir](../../../../docs/cli/param/028_subdir.md)
- **Commands:** run, ask

---

### EC-10: --dry-run --subdir NAME → no filesystem side effects

- **Given:** clean directory (no pre-existing `/-NAME` subdir); cwd is base
- **When:** `clr --dry-run --subdir sidecheck "task"`
- **Then:** Exit 0; dry-run output shows effective dir ending in `/-sidecheck`; but `cwd/-sidecheck` directory does NOT exist on the filesystem after the invocation (Fix: BUG-231)
- **Exit:** 0
- **Source:** [--subdir](../../../../docs/cli/param/028_subdir.md)
- **Commands:** run, ask
