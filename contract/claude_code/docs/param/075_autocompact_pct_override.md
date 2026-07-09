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

### Since

`v2.1.75` (2026-03-13)

### Description

Overrides the auto-compaction threshold as a percentage of the effective context window configured by `CLAUDE_CODE_AUTO_COMPACT_WINDOW`. When `used_tokens / window >= pct / 100`, Claude Code compacts the conversation context.

Example: `CLAUDE_CODE_AUTO_COMPACT_WINDOW=1000000` combined with `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE=80` triggers compaction at approximately 800 000 tokens used.

Naming note: this variable has **no** `_CODE_` infix, unlike most other
variables in this collection — including its own sibling
[`CLAUDE_CODE_AUTO_COMPACT_WINDOW`](074_auto_compact_window.md). The name is
`CLAUDE_AUTOCOMPACT_PCT_OVERRIDE`, not `CLAUDE_CODE_AUTOCOMPACT_PCT_OVERRIDE`.
Note also a second, independent point of divergence from that sibling:
`AUTOCOMPACT` (one word) here vs. `AUTO_COMPACT` (underscored) there. This is
easy to get wrong when working from memory or an unverified summary; the
same no-`_CODE_` trap recurs at [137_job_dir.md](137_job_dir.md),
[138_disable_adopt.md](138_disable_adopt.md),
[139_async_agent_stall_timeout_ms.md](139_async_agent_stall_timeout_ms.md),
and [140_auto_background_tasks.md](140_auto_background_tasks.md).

Introduced alongside `CLAUDE_CODE_AUTO_COMPACT_WINDOW` in Claude Code v2.1.75 (2026-03-13).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [074_auto_compact_window.md](074_auto_compact_window.md) | Token window this percentage is applied to — naming pattern differs, see note above |
| behavior | [../behavior/026_b26_autocompact_pct_override.md](../behavior/026_b26_autocompact_pct_override.md) | Behavioral contract: env var acceptance |
| doc | [100_disable_auto_compact.md](100_disable_auto_compact.md) | Disable auto-compaction entirely |
| doc | [101_disable_compact.md](101_disable_compact.md) | Disable all compaction |
| doc | [140_auto_background_tasks.md](140_auto_background_tasks.md) | Sibling no-`_CODE_`-infix variable |
