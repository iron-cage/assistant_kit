# autocompact_pct_override

Sets the auto-compaction trigger as a percentage of the configured context window.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE` |
| Config Key | — |

### Type

integer (percentage, 1–100)

### Default

Binary default (auto — not publicly documented)

### Description

Overrides the auto-compaction threshold as a percentage of the effective context window configured by `CLAUDE_CODE_AUTO_COMPACT_WINDOW`. When `used_tokens / window >= pct / 100`, Claude Code compacts the conversation context.

Example: `CLAUDE_CODE_AUTO_COMPACT_WINDOW=1000000` combined with `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE=80` triggers compaction at approximately 800 000 tokens used.

Note the naming asymmetry: this variable uses a `CLAUDE_` prefix without `_CODE_`, unlike most other Claude Code env vars which use `CLAUDE_CODE_`.

Introduced alongside `CLAUDE_CODE_AUTO_COMPACT_WINDOW` in Claude Code v2.1.75 (2026-03-13).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [074_auto_compact_window.md](074_auto_compact_window.md) | Token window this percentage is applied to |
| behavior | [../behavior/026_b26_autocompact_pct_override.md](../behavior/026_b26_autocompact_pct_override.md) | Behavioral contract: env var acceptance |
