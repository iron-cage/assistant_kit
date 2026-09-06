# temperature

> ❌ **Refuted — this parameter does not exist.** Retained to record the error.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | ~~`CLAUDE_CODE_TEMPERATURE`~~ — not read by the binary |
| Config Key | — |

### Type

float — valid range: 0.0–1.0

### Default

n/a — the variable has no effect

### Since

Never. Documented here from an unverified assumption; refuted against v2.1.220.

### Description

The previous revision of this doc claimed: *"Controls the randomness of Claude's responses. Lower values (toward 0.0) make responses more deterministic and focused. Higher values (toward 1.0) increase variability and creativity."*

**That is false as an exposed configuration knob.** The literal string `CLAUDE_CODE_TEMPERATURE` occurs **0 times** in the v2.1.220 binary (confirmed against the full 668-identifier census of `CLAUDE_CODE_*`/`CLAUDE_*` strings). The bare identifier `temperature` does occur 72 times and `"temperature"` (quoted) 3 times — consistent with an internal Messages-API request field the binary passes straight through to the model, not with a user-facing environment-variable knob. No CLI flag or config key exists for this either (this doc's own Forms table already showed both as `—`). There is currently no confirmed way for a `claude` CLI user to set sampling temperature.

**Where the belief came from.** `claude_runner_core` exports `CLAUDE_CODE_TEMPERATURE` on every spawn where `.with_temperature()` is used (`module/claude_runner_core/src/command/mod.rs:305`), and `module/claude_version_core/src/params_catalog.rs:1079` independently catalogues it as a real binary parameter. An entire dedicated test file (`module/claude_runner_core/tests/float_edge_cases_test.rs`) asserts this variable's exact string formatting across edge cases (`0`, `1`, `0.7`, etc.) — all of it verifying what the workspace *sets*, none of it verifying the binary *reads* it. Same producer-vs-consumer conflation already recorded for `CLAUDE_CODE_AUTO_CONTINUE` (see [`../behavior/011_b11_auto_continue_env.md`](../behavior/011_b11_auto_continue_env.md)).

**Live impact.** The export is still present and still a no-op against the real binary. Removing it is a workspace change outside this contract crate.

**Verify:**

```bash
V=~/.local/share/claude/versions/2.1.220
grep -ac CLAUDE_CODE_TEMPERATURE "$V"   # → 0  (the claim)
grep -ac CLAUDE_CONFIG_DIR       "$V"   # → 28 (positive control)
grep -ac TOTALLY_FAKE_VAR_XYZ    "$V"   # → 0  (negative control)
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [069_top_k.md](069_top_k.md) | Same refutation pattern (sibling sampling parameter) |
| doc | [070_top_p.md](070_top_p.md) | Same refutation pattern (sibling sampling parameter) |
| doc | [010_auto_continue.md](010_auto_continue.md) | Same producer-vs-consumer error pattern, previously identified |