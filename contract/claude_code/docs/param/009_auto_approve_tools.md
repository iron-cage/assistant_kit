# auto_approve_tools

> ❌ **Refuted — this parameter does not exist.** Retained to record the error.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | ~~`CLAUDE_CODE_AUTO_APPROVE_TOOLS`~~ — not read by the binary |
| Config Key | — |

### Type

bool

### Default

n/a — the variable has no effect

### Since

Never. Documented here from an unverified assumption; refuted against v2.1.220.

### Description

The previous revision of this doc claimed: *"When true, all tool invocations are automatically approved without user confirmation prompts. Equivalent in effect to `--dangerously-skip-permissions` but applied via env var. Intended for fully automated pipelines in sandboxed environments."*

**That is false.** The literal string `CLAUDE_CODE_AUTO_APPROVE_TOOLS` occurs **0 times** in the v2.1.220 binary (a full census of every `CLAUDE_CODE_*`/`CLAUDE_*`/`ANTHROPIC_*`/`API_*`/`BASH_*`/`DISABLE_*`/`MCP_*`-shaped string in the binary — 668 real identifiers extracted via `strings` — contains no variant of this name), and it appears in no official Claude Code documentation. Setting it is a no-op. For the real, confirmed mechanism to skip permission prompts, see [018_dangerously_skip_permissions.md](018_dangerously_skip_permissions.md).

**Where the belief came from.** `claude_runner_core` exports this exact variable on every spawn (`module/claude_runner_core/src/command/mod.rs:296`), and `module/claude_version_core/src/params_catalog.rs:167` independently catalogues it as a real binary parameter. Several tests (`environment_variables_test.rs`, `default_values_test.rs`, `manual_edge_case_check.rs`) assert that it appears in the child environment. Those tests verify that *the workspace sets* the variable; none verifies that setting it changes anything in the `claude` binary. Reading a producer-side call site as evidence of consumer-side behavior is the same reasoning error already recorded for `CLAUDE_CODE_AUTO_CONTINUE` (see [010_auto_continue.md](010_auto_continue.md) and [`../behavior/011_b11_auto_continue_env.md`](../behavior/011_b11_auto_continue_env.md)).

**Live impact.** The export is still present and still a no-op. Removing it is a workspace change outside this contract crate.

**Verify:**

```bash
V=~/.local/share/claude/versions/2.1.220
grep -ac CLAUDE_CODE_AUTO_APPROVE_TOOLS "$V"   # → 0  (the claim)
grep -ac CLAUDE_CONFIG_DIR                "$V"   # → 28 (positive control)
grep -ac TOTALLY_FAKE_VAR_XYZ              "$V"   # → 0  (negative control)
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [018_dangerously_skip_permissions.md](018_dangerously_skip_permissions.md) | Real, confirmed mechanism for the same underlying need |
| doc | [046_permission_mode.md](046_permission_mode.md) | Session-level permission mode |
| doc | [010_auto_continue.md](010_auto_continue.md) | Same producer-vs-consumer error pattern, previously identified |