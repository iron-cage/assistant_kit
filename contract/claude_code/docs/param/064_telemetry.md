# telemetry

> ❌ **Refuted — this parameter does not exist under this name.** The real, confirmed opt-out is [118_disable_telemetry.md](118_disable_telemetry.md) (`DISABLE_TELEMETRY`).

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | ~~`CLAUDE_CODE_TELEMETRY`~~ — not read by the binary |
| Config Key | — |

### Type

bool

### Default

n/a — the variable has no effect

### Since

Never (as documented). Refuted against v2.1.220.

### Description

The previous revision of this doc claimed: *"Controls whether Claude Code sends anonymous usage telemetry to Anthropic. The binary default is `true` (opt-out model). Set to `false` to disable."*

**The literal string `CLAUDE_CODE_TELEMETRY` occurs 0 times** in the v2.1.220 binary. A full census of every `CLAUDE_CODE_*`/`DISABLE_*`-shaped identifier string in the binary (668 real identifiers, extracted via `strings`) shows two different real telemetry-related names instead: `DISABLE_TELEMETRY` (no `CLAUDE_CODE_` prefix — already documented separately, confirmed real, see [118_disable_telemetry.md](118_disable_telemetry.md)) and `CLAUDE_CODE_ENABLE_TELEMETRY` (a positive-boolean form this doc collection does not otherwise document; its exact semantics and default are unconfirmed). This entry's exact name (`CLAUDE_CODE_TELEMETRY`, without `ENABLE_`) does not exist under either real spelling — treat `DISABLE_TELEMETRY` as the confirmed, user-facing opt-out mechanism.

**Where the belief came from.** `claude_runner_core` exports `CLAUDE_CODE_TELEMETRY` on every spawn where `.with_telemetry()` is used (`module/claude_runner_core/src/command/mod.rs:293`), and `module/claude_version_core/src/params_catalog.rs:1071` independently catalogues it as a real binary parameter. Tests assert only that the workspace sets this variable, never that the binary honors it — the same producer-vs-consumer conflation already recorded for `CLAUDE_CODE_AUTO_CONTINUE` (see [`../behavior/011_b11_auto_continue_env.md`](../behavior/011_b11_auto_continue_env.md)).

**Live impact.** The export is still present and still a no-op against the real binary under this exact name. Removing it is a workspace change outside this contract crate.

**Verify:**

```bash
V=~/.local/share/claude/versions/2.1.220
grep -ac CLAUDE_CODE_TELEMETRY        "$V"   # → 0  (the claim)
grep -ac DISABLE_TELEMETRY            "$V"   # → nonzero (the real, confirmed opt-out, #118)
grep -ac CLAUDE_CODE_ENABLE_TELEMETRY "$V"   # → nonzero (differently-named string exists; semantics unconfirmed)
grep -ac CLAUDE_CONFIG_DIR            "$V"   # → 28 (positive control)
grep -ac TOTALLY_FAKE_VAR_XYZ         "$V"   # → 0  (negative control)
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [118_disable_telemetry.md](118_disable_telemetry.md) | `DISABLE_TELEMETRY` — the real, confirmed opt-out mechanism |
| doc | [010_auto_continue.md](010_auto_continue.md) | Same producer-vs-consumer error pattern, previously identified |