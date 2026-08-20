# Test: `--global`

Edge case test planning for the `--global` parameter. See [param/087_global.md](../../../../docs/cli/param/087_global.md) for specification.

`--global` redirects `--topic`'s base from cwd to the global topic home. Tests focus on
the three-level base precedence (`--dir` > `--global` > cwd), the flag's inertness when
there is no topic to place, and the `CLR_GLOBAL` / `CLR_TOPIC_HOME` env pair.

Every case pins `CLR_TOPIC_HOME` (or `TMPDIR`, for the default-home case) at a
`tempfile::TempDir` and runs from a tempdir cwd, so no case touches the host's real
`<temp-dir>/clr-topic`.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--global --topic NAME` → effective dir is `$CLR_TOPIC_HOME/-NAME`, not `cwd/-NAME` | Base Redirection |
| EC-2 | `--global` with no topic to place → byte-identical output; inert | Inertness |
| EC-3 | `--dir PATH` outranks `--global` | Precedence |
| EC-4 | `-g` is an exact alias for `--global` | Alias |
| EC-5 | `--help` output contains `--global` and the `-g` alias | Help |
| EC-6 | `CLR_GLOBAL=1` applied when `--global` absent | Env Var |
| EC-7 | `CLR_TOPIC_HOME` unset → falls back to `<temp-dir>/clr-topic` | Default Home |
| EC-8 | `--global --topic .` → identity; nothing placed under the global home | Identity |

## Test Coverage Summary

- Base Redirection: 1 test (EC-1)
- Inertness: 1 test (EC-2)
- Precedence: 1 test (EC-3)
- Alias: 1 test (EC-4)
- Help: 1 test (EC-5)
- Env Var: 1 test (EC-6)
- Default Home: 1 test (EC-7)
- Identity: 1 test (EC-8)

**Total:** 8 tests

**Implemented by:** `tests/param_extended_flags_test.rs::s90`–`s97`

---

### EC-1: `--global` redirects the topic base

- **Command:** `CLR_TOPIC_HOME=<home> clr --dry-run --global --topic notes "task"`, run from `<cwd>`
- **Expected behavior:** stdout contains `<home>/-notes` and does not contain `<cwd>/-notes`
- **Exit:** 0
- **Source:** [param/087_global.md](../../../../docs/cli/param/087_global.md)

---

### EC-2: `--global` is inert without a topic

- **Command:** `clr --dry-run "task"` vs `clr --dry-run --global "task"`, same env and cwd
- **Expected behavior:** stdout is byte-identical between the two — with no topic directory to place, there is no base to redirect
- **Exit:** 0 for both
- **Source:** [param/087_global.md](../../../../docs/cli/param/087_global.md)

---

### EC-3: `--dir` outranks `--global`

- **Command:** `CLR_TOPIC_HOME=<home> clr --dry-run --global --dir <base> --topic notes "task"`
- **Expected behavior:** stdout contains `<base>/-notes` and not `<home>/-notes` — an explicit path always beats a named default
- **Exit:** 0
- **Source:** [param/087_global.md](../../../../docs/cli/param/087_global.md)

---

### EC-4: `-g` alias

- **Command:** `clr --dry-run -g --topic notes "task"` vs the same with `--global`
- **Expected behavior:** stdout is byte-identical
- **Exit:** 0 for both
- **Source:** [param/087_global.md](../../../../docs/cli/param/087_global.md)

---

### EC-5: `--help` documents the flag

- **Command:** `clr --help`
- **Expected behavior:** stdout contains `--global` and `-g,`
- **Exit:** 0
- **Source:** [param/087_global.md](../../../../docs/cli/param/087_global.md)

---

### EC-6: `CLR_GLOBAL` env fallback

- **Command:** `CLR_TOPIC_HOME=<home> CLR_GLOBAL=1 clr --dry-run --topic notes "task"`
- **Expected behavior:** stdout contains `<home>/-notes` — the env var reaches the same field the flag sets
- **Exit:** 0
- **Source:** [003_env_param.md](../../../../docs/cli/003_env_param.md) Env Param 1, #65

---

### EC-7: Default global home

- **Command:** `TMPDIR=<tmp> clr --dry-run --global --topic notes "task"`, with `CLR_TOPIC_HOME` unset
- **Expected behavior:** stdout contains `<tmp>/clr-topic/-notes`. Pinning `TMPDIR` (what `std::env::temp_dir()` reads on unix) exercises the real fallback without touching the host's `/tmp/clr-topic`
- **Exit:** 0
- **Platform:** unix only
- **Source:** [003_env_param.md](../../../../docs/cli/003_env_param.md) Env Param 12

---

### EC-8: Identity topic under `--global`

- **Command:** `CLR_TOPIC_HOME=<home> clr --dry-run --global --topic . "task"`
- **Expected behavior:** stdout does not mention `<home>` — `.` is identity, so there is no topic directory for `--global` to relocate
- **Exit:** 0
- **Source:** [param/087_global.md](../../../../docs/cli/param/087_global.md)
