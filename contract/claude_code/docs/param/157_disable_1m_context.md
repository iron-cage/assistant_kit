# disable_1m_context

Opts the session out of the 1M-token context window.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_DISABLE_1M_CONTEXT` |
| Config Key | — |

### Type

bool

### Default

`false` (1M context used where the model and entitlement allow it)

### Since

Unverified. Present in the v2.1.220 binary (3 occurrences) but named by no entry in the 2.1.74–2.1.220 changelog.

### Description

Forces the standard context window even where a 1M-token variant would otherwise be selected.

**Certainty is presence, not semantics.** Three occurrences confirm the string exists in the binary and is referenced. No test in this crate sets it and observes a context-window change, and no official documentation is cited here for it. The described effect follows from the name; treat it as expected, not confirmed.

**Why an opt-out exists at all** — the changelog shows 1M context has repeatedly been the thing that goes wrong:

| Version | Failure |
|---------|---------|
| v2.1.129 | Bedrock and Vertex users unable to select "Opus (1M context)" from `/model` (regression) |
| v2.1.172 | Sessions using 1M context *without usage credits* getting permanently stuck; the session now auto-compacts back under the standard limit |
| v2.1.172 | `opusplan` not shipping with 1M context in plan mode for entitled users |
| v2.1.173 | Fable 5 model names with a `[1m]` suffix not normalized — Fable 5 includes 1M context by default, so the suffix is now stripped |

The v2.1.172 stuck-session case is the one this variable most plausibly exists to pre-empt: an unentitled account reaching for 1M context.

**Model-name interaction.** A `[1m]` suffix on a model ID is a separate mechanism for requesting the large window — see [`042_model.md`](042_model.md). Which wins when a `[1m]` model is combined with this variable is not established here.

### Verification

```bash
V=~/.local/share/claude/versions/2.1.220
grep -ac CLAUDE_CODE_DISABLE_1M_CONTEXT "$V"   # → 3
grep -ac CLAUDE_CONFIG_DIR              "$V"   # → 28 (positive control)
grep -ac TOTALLY_FAKE_VAR_XYZ           "$V"   # → 0  (negative control)

# Changelog provenance for the failure table:
grep -rn '1M context' ../version/*.md
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [042_model.md](042_model.md) | `--model` — the `[1m]` suffix mechanism |
| doc | [../model/readme.md](../model/readme.md) | Model catalog and capabilities |
| doc | [../behavior/025_b25_auto_compact_window.md](../behavior/025_b25_auto_compact_window.md) | Auto-compact window — the fallback v2.1.172 wired in |
