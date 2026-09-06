# top_k

> ❌ **Refuted — this parameter does not exist.** Retained to record the error.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | ~~`CLAUDE_CODE_TOP_K`~~ — not read by the binary |
| Config Key | — |

### Type

integer

### Default

n/a — the variable has no effect

### Since

Never. Documented here from an unverified assumption; refuted against v2.1.220.

### Description

The previous revision of this doc claimed: *"Top-k sampling cutoff. Limits token sampling to the k highest-probability tokens at each step."*

**That is false as an exposed configuration knob.** The literal string `CLAUDE_CODE_TOP_K` occurs **0 times** in the v2.1.220 binary (confirmed against the full 668-identifier census of `CLAUDE_CODE_*`/`CLAUDE_*` strings). The bare identifiers `topK` (7 occurrences) and `top_k` (18 occurrences) exist internally — consistent with an internal Messages-API request field passed through to the model, not with a user-facing environment-variable knob. No CLI flag or config key exists for this either.

**Where the belief came from.** `claude_runner_core` exports `CLAUDE_CODE_TOP_K` on every spawn where `.with_top_k()` is used (`module/claude_runner_core/src/command/mod.rs:317`), and `module/claude_version_core/src/params_catalog.rs:1127` independently catalogues it as a real binary parameter. Tests assert only what the workspace *sets*, never that the binary *reads* it. Same producer-vs-consumer conflation already recorded for `CLAUDE_CODE_AUTO_CONTINUE` (see [`../behavior/011_b11_auto_continue_env.md`](../behavior/011_b11_auto_continue_env.md)).

**Live impact.** The export is still present and still a no-op against the real binary. Removing it is a workspace change outside this contract crate.

**Verify:**

```bash
V=~/.local/share/claude/versions/2.1.220
grep -ac CLAUDE_CODE_TOP_K    "$V"   # → 0  (the claim)
grep -ac CLAUDE_CONFIG_DIR    "$V"   # → 28 (positive control)
grep -ac TOTALLY_FAKE_VAR_XYZ "$V"   # → 0  (negative control)
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [065_temperature.md](065_temperature.md) | Same refutation pattern (sibling sampling parameter) |
| doc | [070_top_p.md](070_top_p.md) | Same refutation pattern (sibling sampling parameter) |
| doc | [010_auto_continue.md](010_auto_continue.md) | Same producer-vs-consumer error pattern, previously identified |