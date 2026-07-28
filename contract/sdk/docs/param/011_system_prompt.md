# systemPrompt

Replaces or appends to the default system prompt.

### Forms

| | Value |
|-|-------|
| TS Field | `systemPrompt?: string \| { type: 'preset'; preset: 'claude_code'; append?: string; excludeDynamicSections?: boolean }` |
| Python Field | `system_prompt` (inferred snake_case-identical) |
| CLI Equivalent | `--system-prompt` — [`../../../claude_code/docs/param/063_system_prompt.md`](../../../claude_code/docs/param/063_system_prompt.md) |

### Type

union — plain string (full replacement) or preset-object (`{ type: 'preset', preset: 'claude_code', append?, excludeDynamicSections? }`)

### Default

`undefined` (minimal prompt — explicitly narrower than the CLI's default, which always assembles the full Claude Code system prompt)

### Since

SDK GA

### Description

Two distinct modes, unlike the CLI's single-string `--system-prompt`: a plain string fully replaces the default prompt (matching the CLI flag's behavior exactly), while the `{ type: 'preset', preset: 'claude_code', append, excludeDynamicSections }` object *starts from* the full Claude Code system prompt and layers `append` text on top, optionally with `excludeDynamicSections` stripping runtime-injected sections (the kind of CLAUDE.md/context-loading material documented in `contract/claude_code/docs/behavior/033_b33_claudemd_loading_limits.md` and `034_b34_claudemd_content_pipeline.md`). The documented default being "minimal prompt" rather than "full Claude Code prompt" is a meaningful divergence from CLI behavior: an SDK caller that does nothing gets a leaner agent than a bare `claude` invocation would produce.

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| doc | [readme.md](readme.md) | Master curated parameter table |
| doc | [../../../claude_code/docs/param/063_system_prompt.md](../../../claude_code/docs/param/063_system_prompt.md) | CLI-level equivalent flag (string-replacement mode only) |
| doc | [../../../claude_code/docs/behavior/034_b34_claudemd_content_pipeline.md](../../../claude_code/docs/behavior/034_b34_claudemd_content_pipeline.md) | The dynamic sections `excludeDynamicSections` can strip |
