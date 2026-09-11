# bash_timeout

> ❌ **Refuted — this parameter does not exist.** The real mechanism is [096_bash_default_timeout_ms.md](096_bash_default_timeout_ms.md) (`BASH_DEFAULT_TIMEOUT_MS`).

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | ~~`CLAUDE_CODE_BASH_TIMEOUT`~~ — not read by the binary |
| Config Key | — |

### Type

integer (milliseconds)

### Default

n/a — the variable has no effect

### Since

Never. Documented here from an unverified assumption; refuted against v2.1.220.

### Description

The previous revision of this doc claimed: *"Sets the default timeout for each bash command Claude executes. If a bash command runs longer than this value, Claude Code terminates it and returns a timeout error. The binary default is 2 minutes."*

**That is false under this name.** The literal string `CLAUDE_CODE_BASH_TIMEOUT` occurs **0 times** in the v2.1.220 binary — confirmed both directly and against the full 668-identifier census of `CLAUDE_CODE_*`/`CLAUDE_*`/`BASH_*`-shaped strings, which contains no variant of this name at all (not even without the `_CODE_` infix). The concept itself is real, but the actual binary-read variable is the differently-named, already-documented [`BASH_DEFAULT_TIMEOUT_MS`](096_bash_default_timeout_ms.md) (#96) — same default value (`120000`ms / 2 minutes), no `CLAUDE_CODE_` prefix. This entry appears to be a duplicate of #96 invented under the wrong, `CLAUDE_CODE`-prefixed name.

**Where the belief came from.** `claude_runner_core` exports `CLAUDE_CODE_BASH_TIMEOUT` on every spawn where `.with_bash_timeout()` is used (`module/claude_runner_core/src/command/mod.rs:281`), and `module/claude_version_core/src/params_catalog.rs:247` independently catalogues it as a real binary parameter, as does a dedicated CLI subcommand test (`module/claude_version/tests/cli/params_command_test.rs:133`). All of this verifies what the workspace *sets*, never that the binary *reads* it under this name — same producer-vs-consumer conflation already recorded for `CLAUDE_CODE_AUTO_CONTINUE` (see [`../behavior/011_b11_auto_continue_env.md`](../behavior/011_b11_auto_continue_env.md)).

**Live impact.** The export is still present and still a no-op against the real binary. Removing it is a workspace change outside this contract crate. Users who actually need to change the bash default timeout should set `BASH_DEFAULT_TIMEOUT_MS` (#96) instead.

**Verify:**

```bash
V=~/.local/share/claude/versions/2.1.220
grep -ac CLAUDE_CODE_BASH_TIMEOUT "$V"   # → 0  (the claim)
grep -ac BASH_DEFAULT_TIMEOUT_MS  "$V"   # → nonzero (the real mechanism, #96)
grep -ac CLAUDE_CONFIG_DIR        "$V"   # → 28 (positive control)
grep -ac TOTALLY_FAKE_VAR_XYZ     "$V"   # → 0  (negative control)
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [012_bash_max_timeout.md](012_bash_max_timeout.md) | Same refutation pattern (sibling ceiling parameter) |
| doc | [096_bash_default_timeout_ms.md](096_bash_default_timeout_ms.md) | `BASH_DEFAULT_TIMEOUT_MS` — the real, confirmed mechanism for this need |
| doc | [../tool/004_bash.md](../tool/004_bash.md) | Bash tool this was believed to configure |
| doc | [010_auto_continue.md](010_auto_continue.md) | Same producer-vs-consumer error pattern, previously identified |