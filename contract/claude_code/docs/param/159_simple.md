# simple

Marker variable set by `--bare` to signal minimal mode to child processes.

### Forms

| | Value |
|-|-------|
| CLI Flag | — (set indirectly by `--bare`) |
| Env Var | `CLAUDE_CODE_SIMPLE` |
| Config Key | — |

### Type

bool (`1` when set)

### Default

unset

### Since

≤v2.1.81 (2026-03-20) — bounded by `--bare`, which sets it and was added in that release ([`../version/008_v2_1_81.md`](../version/008_v2_1_81.md)). No changelog entry names the variable directly.

### Description

`claude --help` documents this variable inside the `--bare` entry: *"Sets `CLAUDE_CODE_SIMPLE=1`."* That single sentence is the whole first-party specification — the variable has no `--help` entry of its own, which is why it was absent from this collection until now.

**Its documented direction is outward.** `--bare` *sets* it, which makes it observable by anything the session spawns — hooks that survive, subprocesses, tooling that inspects its environment. That is a useful marker: a child process can detect it is running under minimal mode without being told.

**Whether setting it manually enables minimal mode is NOT established.** The help text documents `--bare → CLAUDE_CODE_SIMPLE=1`, not the converse. Exporting the variable and expecting `--bare` semantics is exactly the inference this collection got wrong once before with `CLAUDE_CODE_AUTO_CONTINUE` (see [`010_auto_continue.md`](010_auto_continue.md)) — reading a variable the workspace *writes* as one the binary *reads*. Twenty occurrences prove the binary references the string; they do not prove it honors it as an input.

**To get minimal mode, pass `--bare`.** That is the documented, unambiguous path.

### Verification

```bash
claude --help | grep -A12 -- '--bare' | grep SIMPLE   # → "Sets CLAUDE_CODE_SIMPLE=1."

V=~/.local/share/claude/versions/2.1.220
grep -ac CLAUDE_CODE_SIMPLE   "$V"   # → 20
grep -ac TOTALLY_FAKE_VAR_XYZ "$V"   # → 0 (negative control)
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [143_bare.md](143_bare.md) | `--bare` — the flag that sets this variable |
| doc | [010_auto_continue.md](010_auto_continue.md) | The refutation this doc's caveat is modeled on |
| doc | [132_claudecode.md](132_claudecode.md) | `CLAUDECODE` — broader subprocess marker |
