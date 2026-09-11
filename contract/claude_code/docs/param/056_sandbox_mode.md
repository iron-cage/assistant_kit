# sandbox_mode

> ❌ **Refuted — this parameter does not exist under this name.** Retained to record the error.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | ~~`CLAUDE_CODE_SANDBOX_MODE`~~ — not read by the binary |
| Config Key | — |

### Type

bool

### Default

n/a — the variable has no effect

### Since

Never (as documented). Refuted against v2.1.220.

### Description

The previous revision of this doc claimed: *"Enables sandbox mode, which restricts certain system-level operations Claude Code can perform. When true (the default), the process runs with additional isolation constraints. The `claude_runner_core` builder also defaults this to `true` — no difference between builder and binary default for this parameter."*

**The literal string `CLAUDE_CODE_SANDBOX_MODE` occurs 0 times** in the v2.1.220 binary, including every case-variant tried (`sandboxMode`, `SANDBOX_MODE`). The "no difference between builder and binary default" claim was never actually checked against the binary — it compared the builder's own default to itself. A full census of every `CLAUDE_CODE_*`-shaped identifier string in the binary (668 real identifiers) does show sandbox-related names — `CLAUDE_CODE_SANDBOXED`, `CLAUDE_CODE_FORCE_SANDBOX`, `CLAUDE_CODE_BASH_SANDBOX_SHOW_INDICATOR`, `CLAUDE_CODE_BUBBLEWRAP` — but none is a drop-in bool toggle matching this entry's shape (a default-`true`, user-settable on/off switch); `CLAUDE_CODE_SANDBOXED` reads more like a read-only "am I currently sandboxed" marker and `CLAUDE_CODE_FORCE_SANDBOX` a one-way force-on switch, not a toggle a user flips to opt out. None of this is confirmed either way.

**Where the belief came from.** `claude_runner_core` exports `CLAUDE_CODE_SANDBOX_MODE` on every spawn where `.with_sandbox_mode()` is used (`module/claude_runner_core/src/command/mod.rs:308`), and `module/claude_version_core/src/params_catalog.rs:991` independently catalogues it as a real binary parameter. Tests assert only that the workspace sets this variable, never that the binary honors it — the same producer-vs-consumer conflation already recorded for `CLAUDE_CODE_AUTO_CONTINUE` (see [`../behavior/011_b11_auto_continue_env.md`](../behavior/011_b11_auto_continue_env.md)).

**Live impact.** The export is still present and still a no-op against the real binary under this exact name. Removing it is a workspace change outside this contract crate.

**Verify:**

```bash
V=~/.local/share/claude/versions/2.1.220
grep -ac CLAUDE_CODE_SANDBOX_MODE  "$V"   # → 0  (the claim)
grep -ac CLAUDE_CODE_SANDBOXED     "$V"   # → nonzero (differently-shaped string exists; semantics unconfirmed)
grep -ac CLAUDE_CODE_FORCE_SANDBOX "$V"   # → nonzero (differently-shaped string exists; semantics unconfirmed)
grep -ac CLAUDE_CONFIG_DIR         "$V"   # → 28 (positive control)
grep -ac TOTALLY_FAKE_VAR_XYZ      "$V"   # → 0  (negative control)
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [093_sandbox_allow_apple_events.md](093_sandbox_allow_apple_events.md) | Apple Events exception in sandbox (macOS) — a real, confirmed config key |
| doc | [018_dangerously_skip_permissions.md](018_dangerously_skip_permissions.md) | Bypass permissions (used alongside sandbox) |
| doc | [010_auto_continue.md](010_auto_continue.md) | Same producer-vs-consumer error pattern, previously identified |