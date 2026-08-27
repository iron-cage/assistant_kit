# Behavior B11: CLAUDE_CODE_AUTO_CONTINUE Env Var — REFUTED

### Scope

- **Purpose**: Record that the hypothesised `CLAUDE_CODE_AUTO_CONTINUE` env var is not read by the `claude` binary, and flag that the workspace still sets it.
- **Responsibility**: Authoritative instance for behavior B11 — retained as a refuted hypothesis with the disconfirming evidence, per the collection's no-silent-deletion policy.
- **In Scope**: Disconfirmation of `CLAUDE_CODE_AUTO_CONTINUE`; the live workspace call site that still exports it; why the NEG-ONLY tier could not catch this.
- **Out of Scope**: `CLAUDE_CODE_SESSION_DIR` (separately refuted, → [B23](023_b23_session_dir_override.md)); the `--continue` flag itself, which is real (→ [B4](004_b4_continue_flag.md)).

### Behavior

**Status**: ❌ Refuted | **Certainty**: 95% refuted | **Tier**: NEG-ONLY (insufficient — see below) | **Refuted at**: v2.1.220 | **Evidence**: E10, E21, E72

**The original hypothesis was:** *"`CLAUDE_CODE_AUTO_CONTINUE` environment variable enables automated continuation mode in the `claude` binary."*

**That hypothesis is refuted.** The literal string `CLAUDE_CODE_AUTO_CONTINUE` does not occur anywhere in the v2.1.220 binary (0 occurrences across 271 MB), under the same scan whose positive and negative controls are recorded in E72. It appears in no official Claude Code documentation. Automated continuation is driven by the `--continue` flag (→ [B4](004_b4_continue_flag.md)), which is real and documented; no env-var equivalent exists.

**The evidence never supported the hypothesis in the first place.** E10 proves only that *this workspace sets* the variable — it says nothing about whether the *binary reads* it. That is a claim about `claude_runner_core`, not about `claude`. Reading a producer-side call site as evidence of consumer-side behavior is the specific reasoning error this refutation corrects.

**Why the test tier could not catch this.** NEG-ONLY asserts that the binary does not name the variable in stderr when it is set. A variable the binary has never heard of also goes unmentioned, so the assertion passes identically whether the variable is honored, silently ignored, or entirely absent from the binary. See [B23](023_b23_session_dir_override.md), refuted by the same method on the same day.

**Live consumer impact — unresolved.** Unlike [B23](023_b23_session_dir_override.md), whose dead export the workspace already removed under BUG-490/BUG-493, this variable is still exported on every run: `module/claude_runner_core/src/command/mod.rs:290` pushes it into the child environment, and at least six tests in `claude_runner_core` and `claude_runner` assert that it appears there. Those tests verify that the workspace sets the variable; none verifies that setting it changes anything. The export is a no-op against v2.1.220 and the assertions lock the no-op in place. Removing it is a workspace change outside this contract crate and is left to the owner's decision.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E10 | B11 | Code | `../../../../module/claude_runner_core/src/command/mod.rs` | `grep -n CLAUDE_CODE_AUTO_CONTINUE` | `pairs.push( ( "CLAUDE_CODE_AUTO_CONTINUE", auto_continue.to_string() ) )` — env var exported before spawning `claude`. Proves the workspace *sets* the variable; carries no information about whether the binary *reads* it. Location corrected 2026-08-27: previously cited as `src/command.rs` lines 647–648, a path and line range that no longer exist. |
| E21 | B11 | Test | `../../tests/behavior/b11_auto_continue.rs` | `b11_auto_continue_env_var_recognized` | Binary does not print `CLAUDE_CODE_AUTO_CONTINUE` in stderr when env var is set — negative assertion; passes identically for a variable absent from the binary, which is why it did not catch this refutation |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [023_b23_session_dir_override.md](023_b23_session_dir_override.md) | `CLAUDE_CODE_SESSION_DIR` env var (different env var) |
| behavior | [025_b25_auto_compact_window.md](025_b25_auto_compact_window.md) | `CLAUDE_CODE_AUTO_COMPACT_WINDOW` env var (same NEG-ONLY pattern) |
| behavior | [026_b26_autocompact_pct_override.md](026_b26_autocompact_pct_override.md) | `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE` env var (same NEG-ONLY pattern) |
| test | `../../tests/behavior/b11_auto_continue.rs` | Invalidation test (NEG-ONLY) |
