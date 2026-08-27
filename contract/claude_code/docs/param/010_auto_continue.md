# auto_continue

> ❌ **Refuted — this parameter does not exist.** Retained to record the error. See [B11](../behavior/011_b11_auto_continue_env.md).

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | ~~`CLAUDE_CODE_AUTO_CONTINUE`~~ — not read by the binary |
| Config Key | — |

### Type

bool

### Default

n/a — the variable has no effect

### Since

Never. Documented here from an unverified assumption; refuted against v2.1.220.

### Description

The previous revision of this doc claimed: *"When true, Claude automatically continues long responses that would otherwise be truncated, without requiring a user prompt to proceed. Enables fully unattended automation in `--print` mode. Without this, a truncated response in `--print` mode may hang waiting for input."*

**That is false.** The literal string `CLAUDE_CODE_AUTO_CONTINUE` occurs **0 times** in the v2.1.220 binary and appears in no official Claude Code documentation. Setting it is a no-op, and the described failure mode — `--print` hanging on a truncated response — is not something this variable ever prevented.

**Where the belief came from.** `claude_runner_core` exports this variable on every spawn (`module/claude_runner_core/src/command/mod.rs:290`), and several tests assert that it appears in the child environment. Those tests verify that *the workspace sets* the variable; none verifies that setting it changes anything. Reading a producer-side call site as evidence of consumer-side behavior is the specific error this correction records.

**Live impact.** The export is still present and still a no-op. Removing it is a workspace change outside this contract crate.

**For the underlying need** — bounding an unattended run — see [`076_max_turns.md`](076_max_turns.md), which is a real, accepted flag.

**Verify:**

```bash
V=~/.local/share/claude/versions/2.1.220
grep -ac CLAUDE_CODE_AUTO_CONTINUE "$V"   # → 0  (the claim)
grep -ac CLAUDE_CONFIG_DIR         "$V"   # → 28 (positive control)
grep -ac TOTALLY_FAKE_VAR_XYZ      "$V"   # → 0  (negative control)
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [051_print.md](051_print.md) | Print mode (enables unattended automation) |
| doc | [076_max_turns.md](076_max_turns.md) | Maximum continuation turns — a real flag serving the adjacent need |
| behavior | [../behavior/011_b11_auto_continue_env.md](../behavior/011_b11_auto_continue_env.md) | Refutation record with the disconfirming evidence (E10, E21, E72) |