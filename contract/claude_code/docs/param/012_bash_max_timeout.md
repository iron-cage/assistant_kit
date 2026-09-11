# bash_max_timeout

> ❌ **Refuted — this parameter does not exist.** The real mechanism is [098_bash_max_timeout_ms.md](098_bash_max_timeout_ms.md) (`BASH_MAX_TIMEOUT_MS`).

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | ~~`CLAUDE_CODE_BASH_MAX_TIMEOUT`~~ — not read by the binary |
| Config Key | — |

### Type

integer (milliseconds)

### Default

n/a — the variable has no effect

### Since

Never. Documented here from an unverified assumption; refuted against v2.1.220.

### Description

The previous revision of this doc claimed: *"The maximum timeout that any individual bash command is permitted to use, regardless of what Claude requests. Acts as a ceiling on `CLAUDE_CODE_BASH_TIMEOUT`."*

**That is false under this name.** The literal string `CLAUDE_CODE_BASH_MAX_TIMEOUT` occurs **0 times** in the v2.1.220 binary — confirmed against the full 668-identifier census of `CLAUDE_CODE_*`/`CLAUDE_*`/`BASH_*`-shaped strings. (A bare substring match on `BASH_MAX_TIMEOUT` returns 5 hits, but those are exactly the 5 occurrences of the real, differently-named `BASH_MAX_TIMEOUT_MS` — not independent evidence of a `CLAUDE_CODE_`-prefixed variant.) The actual binary-read variable is the already-documented [`BASH_MAX_TIMEOUT_MS`](098_bash_max_timeout_ms.md) (#98) — same default value (`600000`ms / 10 minutes), no `CLAUDE_CODE_` prefix. This entry appears to be a duplicate of #98 invented under the wrong, `CLAUDE_CODE`-prefixed name — the same pattern as its sibling [013_bash_timeout.md](013_bash_timeout.md).

**Where the belief came from.** `claude_runner_core` exports `CLAUDE_CODE_BASH_MAX_TIMEOUT` on every spawn where `.with_bash_max_timeout()` is used (`module/claude_runner_core/src/command/mod.rs:284`), and `module/claude_version_core/src/params_catalog.rs:231` independently catalogues it as a real binary parameter. All of this verifies what the workspace *sets*, never that the binary *reads* it under this name — same producer-vs-consumer conflation already recorded for `CLAUDE_CODE_AUTO_CONTINUE` (see [`../behavior/011_b11_auto_continue_env.md`](../behavior/011_b11_auto_continue_env.md)).

**Live impact.** The export is still present and still a no-op against the real binary. Removing it is a workspace change outside this contract crate. Users who actually need to change the bash max timeout should set `BASH_MAX_TIMEOUT_MS` (#98) instead.

**Verify:**

```bash
V=~/.local/share/claude/versions/2.1.220
grep -ac CLAUDE_CODE_BASH_MAX_TIMEOUT "$V"   # → 0  (the claim)
grep -ac BASH_MAX_TIMEOUT_MS          "$V"   # → nonzero (the real mechanism, #98)
grep -ac CLAUDE_CONFIG_DIR            "$V"   # → 28 (positive control)
grep -ac TOTALLY_FAKE_VAR_XYZ         "$V"   # → 0  (negative control)
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [013_bash_timeout.md](013_bash_timeout.md) | Same refutation pattern (sibling default-timeout parameter) |
| doc | [098_bash_max_timeout_ms.md](098_bash_max_timeout_ms.md) | `BASH_MAX_TIMEOUT_MS` — the real, confirmed mechanism for this need |
| doc | [../tool/004_bash.md](../tool/004_bash.md) | Bash tool this was believed to constrain |
| doc | [010_auto_continue.md](010_auto_continue.md) | Same producer-vs-consumer error pattern, previously identified |