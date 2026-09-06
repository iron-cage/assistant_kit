# action_mode

> ❌ **Refuted — this parameter does not exist under this name.** Retained to record the error.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | ~~`CLAUDE_CODE_ACTION_MODE`~~ — not read by the binary |
| Config Key | — |

### Type

enum — `Ask` `Auto` `Plan`

### Default

n/a — the variable has no effect

### Since

Never (as documented). Refuted against v2.1.220.

### Description

The previous revision of this doc claimed: *"Controls the default action mode for tool execution. `Ask` (default) prompts the user before each tool call. `Auto` executes tools without prompting. `Plan` puts Claude in read-only planning mode."*

**The literal string `CLAUDE_CODE_ACTION_MODE` occurs 0 times** in the v2.1.220 binary. A full census of every `CLAUDE_CODE_*`-shaped identifier string in the binary (668 real identifiers, extracted via `strings`) contains a differently-named `CLAUDE_CODE_ACTION` (no `_MODE` suffix) instead — but its semantics are unconfirmed. Given the binary's own naming convention for environment-detection markers (`CLAUDE_CODE_ENTRYPOINT`, `CLAUDE_CODE_CHILD_SESSION`, `CLAUDE_CODE_REMOTE_ENVIRONMENT_TYPE`), `CLAUDE_CODE_ACTION` more plausibly names a GitHub-Actions-wrapper marker than an Ask/Auto/Plan permission toggle — this is not verified either way, and no doc instance should assume they are the same feature. The real, confirmed mechanism for controlling permission behavior is `--permission-mode` (→ [046_permission_mode.md](046_permission_mode.md)), whose `default`/`bypassPermissions`/`plan`/`auto` enum values already cover the Ask/Auto/Plan concept this entry described.

**Where the belief came from.** `claude_runner_core` exports `CLAUDE_CODE_ACTION_MODE` on every spawn (`module/claude_runner_core/src/command/mod.rs:299`), and `module/claude_version_core/src/params_catalog.rs:87` independently catalogues it as a real binary parameter. `module/claude_runner_core/src/types.rs:12` even carries a doc comment asserting it "Maps to `CLAUDE_CODE_ACTION_MODE` environment variable." Tests assert only that the workspace sets this variable, never that the binary honors it — the same producer-vs-consumer conflation already recorded for `CLAUDE_CODE_AUTO_CONTINUE` (see [`../behavior/011_b11_auto_continue_env.md`](../behavior/011_b11_auto_continue_env.md)).

**Live impact.** The export is still present and still a no-op against the real binary. Removing it is a workspace change outside this contract crate.

**Verify:**

```bash
V=~/.local/share/claude/versions/2.1.220
grep -ac CLAUDE_CODE_ACTION_MODE "$V"   # → 0  (the claim)
grep -ac CLAUDE_CODE_ACTION      "$V"   # → nonzero (differently-named string exists; semantics unconfirmed)
grep -ac CLAUDE_CONFIG_DIR       "$V"   # → 28 (positive control)
grep -ac TOTALLY_FAKE_VAR_XYZ    "$V"   # → 0  (negative control)
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [046_permission_mode.md](046_permission_mode.md) | Real, confirmed mechanism covering the same Ask/Auto/Plan concept |
| doc | [010_auto_continue.md](010_auto_continue.md) | Same producer-vs-consumer error pattern, previously identified |