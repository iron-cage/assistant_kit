# auto_compact_window

Sets the context capacity in tokens used for auto-compaction calculations.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_AUTO_COMPACT_WINDOW` |
| Config Key | — |

### Type

integer (tokens)

### Default

Model's context window: `200 000` (standard) or `1 000 000` (extended context)

### Since

`v2.1.75` (2026-03-13)

### Description

Sets the effective context window size in tokens for auto-compaction threshold calculations. When the active conversation approaches this token count (as a percentage governed by `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE`), Claude Code automatically compacts the conversation context.

Only a value **lower** than the model's actual context window is meaningful — the value is capped at the model's actual window. Use a lower value (e.g. `500000` on a 1M model) to trigger compaction earlier, preserving headroom for continued work. Setting this decouples the compaction threshold from the status line's `used_percentage`, which always reports against the model's full context window regardless of this setting.

Introduced in Claude Code v2.1.75 (2026-03-13), alongside 1M context window support for Opus 4.6 on Max, Team, and Enterprise plans.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [075_autocompact_pct_override.md](075_autocompact_pct_override.md) | Percentage applied to this window value |
| behavior | [../behavior/025_b25_auto_compact_window.md](../behavior/025_b25_auto_compact_window.md) | Behavioral contract: env var acceptance |
| doc | [100_disable_auto_compact.md](100_disable_auto_compact.md) | Disable auto-compaction entirely |
| doc | [101_disable_compact.md](101_disable_compact.md) | Disable all compaction |
