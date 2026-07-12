# model

Model alias or full model ID to use for the session.

### Forms

| | Value |
|-|-------|
| TS Field | `model?: string` |
| Python Field | `model` (inferred snake_case-identical) |
| CLI Equivalent | `--model` / `model` config key — [`../../../claude_code/docs/param/042_model.md`](../../../claude_code/docs/param/042_model.md) |

### Type

string (alias or full model ID)

### Default

"Default from CLI" — i.e. whatever the spawned `claude` binary's own default-model resolution would pick if launched without `--model` (settings-file `model` key, then binary built-in default).

### Since

SDK GA

### Description

Field-for-field equivalent of `claude_code`'s already-documented `--model`/`model` config key — same alias resolution (e.g. `"claude-sonnet-5"`, `"claude-opus-4-8"`), no SDK-specific behavior beyond being a typed struct field instead of an argv string. The official quickstart example sets it directly: `options: { model: "claude-opus-4-6", ... }`.

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| doc | [readme.md](readme.md) | Master curated parameter table |
| doc | [../../../claude_code/docs/param/042_model.md](../../../claude_code/docs/param/042_model.md) | CLI-level equivalent, full alias/resolution detail |
