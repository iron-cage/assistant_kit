# log_level

> ❌ **Refuted — this parameter does not exist under this name.** Retained to record the error.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | ~~`CLAUDE_CODE_LOG_LEVEL`~~ — not read by the binary |
| Config Key | — |

### Type

enum — `Error` `Warn` `Info` `Debug` `Trace`

### Default

n/a — the variable has no effect

### Since

Never (as documented). Refuted against v2.1.220.

### Description

The previous revision of this doc claimed: *"Controls the minimum severity level of log messages emitted by Claude Code's internal logger. `Error` shows only errors; `Trace` shows everything."*

**The literal string `CLAUDE_CODE_LOG_LEVEL` occurs 0 times** in the v2.1.220 binary. A full census of every `CLAUDE_CODE_*`-shaped identifier string in the binary (668 real identifiers, extracted via `strings`) contains a differently-named `CLAUDE_CODE_DEBUG_LOG_LEVEL` instead — plausibly the same concept under a more specific name, but this is not confirmed (no doc instance has verified its accepted values or default). Until that is independently verified, treat this parameter as absent; the confirmed way to get more diagnostic output is `--debug`/`--debug-file` (→ [019_debug.md](019_debug.md), [020_debug_file.md](020_debug_file.md)).

**Where the belief came from.** `claude_runner_core` exports `CLAUDE_CODE_LOG_LEVEL` on every spawn where `.with_log_level()` is used (`module/claude_runner_core/src/command/mod.rs:302`), and `module/claude_version_core/src/params_catalog.rs:703` independently catalogues it as a real binary parameter; `module/claude_runner_core/src/types.rs:72` carries a doc comment asserting it "Maps to `CLAUDE_CODE_LOG_LEVEL` environment variable." Tests assert only that the workspace sets this variable, never that the binary honors it — the same producer-vs-consumer conflation already recorded for `CLAUDE_CODE_AUTO_CONTINUE` (see [`../behavior/011_b11_auto_continue_env.md`](../behavior/011_b11_auto_continue_env.md)).

**Live impact.** The export is still present and still a no-op against the real binary under this exact name. Removing it, or renaming it to the real `CLAUDE_CODE_DEBUG_LOG_LEVEL` once that is independently verified, is a workspace change outside this contract crate.

**Verify:**

```bash
V=~/.local/share/claude/versions/2.1.220
grep -ac CLAUDE_CODE_LOG_LEVEL       "$V"   # → 0  (the claim)
grep -ac CLAUDE_CODE_DEBUG_LOG_LEVEL "$V"   # → nonzero (differently-named string exists; semantics unconfirmed)
grep -ac CLAUDE_CONFIG_DIR           "$V"   # → 28 (positive control)
grep -ac TOTALLY_FAKE_VAR_XYZ        "$V"   # → 0  (negative control)
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [019_debug.md](019_debug.md) | Real, confirmed mechanism for diagnostic output |
| doc | [020_debug_file.md](020_debug_file.md) | Write debug output to file |
| doc | [071_verbose.md](071_verbose.md) | Verbose output mode |
| doc | [010_auto_continue.md](010_auto_continue.md) | Same producer-vs-consumer error pattern, previously identified |