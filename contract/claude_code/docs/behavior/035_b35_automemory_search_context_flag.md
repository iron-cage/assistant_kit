# Behavior B35: Auto-Memory Search Context Section Gated by tengu_coral_fern Flag

### Scope

- **Purpose**: Document that the `tengu_coral_fern` Statsig feature flag, when true, appends a `## Searching past context` section to the auto-memory system prompt — providing the model with grep commands for recovering context from prior sessions via memory topic files and session JSONL transcripts.
- **Responsibility**: Authoritative instance for behavior B35 — defines the flag-gated section content, the bash vs. tool mode command format variants, the session directory path resolution, and the default-off state.
- **In Scope**: `tengu_coral_fern` Statsig flag; `VfT()` function; `## Searching past context` section content; memory topic file grep command format; session JSONL grep command format; bash mode vs. tool mode command variants (`Yz()` gate).
- **Out of Scope**: Auto-memory directory creation and structure (→ `../storage/`); CLAUDE.md @-reference loading (→ [B32](032_b32_claudemd_at_ref_path_filter.md)); MEMORY.md 200-line truncation (→ [B33](033_b33_claudemd_loading_limits.md)); `tengu_paper_halyard` flag that suppresses CLAUDE.md files (→ [B34](034_b34_claudemd_content_pipeline.md)).

### Behavior

**Status**: ✅ Confirmed | **Certainty**: 99% | **Tier**: UNVERIFIED | **Since**: v2.1.74 | **Evidence**: E61

When the `tengu_coral_fern` Statsig feature flag is `true`, the `VfT(memDir)` function returns a `## Searching past context` section that is spread into the auto-memory system prompt. When the flag is `false` (the default — `Wq("tengu_coral_fern", !1)`), `VfT()` returns `[]` and no search section appears.

**Injected section content (when flag is true):**

```
## Searching past context

When looking for past context:
1. Search topic files in your memory directory:
   <grep command for memory .md files>
2. Session transcript logs (last resort — large files, slow):
   <grep command for session .jsonl files>
Use narrow search terms (error messages, file paths, function names) rather than broad keywords.
```

**Command format — two variants based on `Yz()` (bash mode check):**

| Mode | Memory files command | Session JSONL command |
|------|---------------------|----------------------|
| Bash mode | `grep -rn "<search term>" {memDir} --include="*.md"` | `grep -rn "<search term>" {sessionDir}/ --include="*.jsonl"` |
| Tool mode (default) | `{GR} with pattern="<search term>" path="{memDir}" glob="*.md"` | `{GR} with pattern="<search term>" path="{sessionDir}/" glob="*.jsonl"` |

- `memDir` = the auto-memory directory (e.g., `~/.claude/projects/-home-user1-pro/memory/`)
- `sessionDir` = `qw(R8())` — the session storage directory (e.g., `~/.claude/projects/-home-user1-pro/`)
- `GR` = the Grep tool name as used in the current Claude Code session context

**Practical effect:**

When enabled, the model's auto-memory section includes actionable recovery commands alongside the standard "How to save memories" guidance. The intended purpose is context continuity across sessions — the model can search its own memory topic files and past session JSONL transcripts when prior context would help the current task.

Without the flag (default state), the "Searching past context" section is absent entirely. Users seeing grep commands in their auto-memory section are on a session where this flag has been enabled server-side.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E61 | B35 | Code | Binary analysis — `strings /home/user1/.local/share/claude/versions/2.1.74` — v2.1.74 session (2026-06-29) | `VfT()` at binary offset ~111,396,000 (adjacent to auto-memory functions `cy4`, `om6`) | Full function extracted: `function VfT(T){if(!Wq("tengu_coral_fern",!1))return[];let _=qw(R8()),q=Yz(),K=q?\`grep -rn "<search term>" ${T} --include="*.md"\`:\`${GR} with pattern="<search term>" path="${T}" glob="*.md"\`,O=q?\`grep -rn "<search term>" ${_}/ --include="*.jsonl"\`:\`${GR} with pattern="<search term>" path="${_}/" glob="*.jsonl"\`;return["## Searching past context","","When looking for past context:","1. Search topic files in your memory directory:","```",K,"```","2. Session transcript logs (last resort \u2014 large files, slow):","```",O,"```","Use narrow search terms (error messages, file paths, function names) rather than broad keywords.",""]}`. Default confirmed false: `Wq("tengu_coral_fern",!1)` — second arg is the default. |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, overview table, invalidation tests |
| behavior | [034_b34_claudemd_content_pipeline.md](034_b34_claudemd_content_pipeline.md) | B34: `tengu_paper_halyard` — contrasting flag that suppresses Project/Local CLAUDE.md files |
| behavior | [030_b30_subagent_context_inheritance.md](030_b30_subagent_context_inheritance.md) | B30: subagents inherit full CLAUDE.md and auto-memory; this flag changes what instructions appear in the auto-memory section |
