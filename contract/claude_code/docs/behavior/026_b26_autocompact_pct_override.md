# Behavior B26: CLAUDE_AUTOCOMPACT_PCT_OVERRIDE Env Var

### Scope

- **Purpose**: Document that `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE` env var overrides the auto-compaction percentage threshold in the `claude` binary.
- **Responsibility**: Authoritative instance for behavior B26 — defines the behavior statement, certainty level, and supporting evidence. Tier is NEG-ONLY.
- **In Scope**: `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE` env var; naming asymmetry (`CLAUDE_` prefix without `_CODE_`).
- **Out of Scope**: Token window size (-> [B25](025_b25_auto_compact_window.md)); session storage directory (-> [B23](023_b23_session_dir_override.md)).

### Behavior

**Status**: 🎯 Observed | **Certainty**: 85% | **Tier**: NEG-ONLY (existence separately confirmed — see below) | **Since**: v2.1.75 | **Evidence**: E50, E51, E76

`CLAUDE_AUTOCOMPACT_PCT_OVERRIDE` environment variable overrides the auto-compaction percentage: compaction fires when `used_tokens / window >= pct / 100`. The percentage is applied to the effective window set by `CLAUDE_CODE_AUTO_COMPACT_WINDOW`.

**Lowers only.** Official documentation states the override *cannot raise* the compaction threshold — it can only make compaction fire earlier than the default, never later. A value above the built-in threshold is therefore not an error but has no effect. This one-directional constraint mirrors `CLAUDE_CODE_AUTO_COMPACT_WINDOW`'s own cap at the model context window ([B25](025_b25_auto_compact_window.md)): both variables can shrink the compaction budget, neither can grow it.

**Applies to subagents.** The override governs both main conversations and subagent conversations. This matters for cost: each subagent builds an isolated cache prefix from zero ([B37](037_b37_subagent_cache_ttl.md)), so a lowered threshold multiplies compaction work across every dispatched agent rather than affecting one conversation.

Note the naming asymmetry: this variable uses a `CLAUDE_` prefix without `_CODE_`, unlike most other Claude Code env vars. The default percentage is not publicly documented.

**Existence confirmed.** 6 occurrences in the v2.1.220 binary (E76), under the same scan whose controls are recorded in E72. Certainty is raised from 80% to 85%: the variable demonstrably exists and is officially documented, leaving only the exact arithmetic inferred. Compare [B11](011_b11_auto_continue_env.md) and [B23](023_b23_session_dir_override.md), which carried comparable certainty on NEG-ONLY evidence alone and were both refuted at 0 occurrences.

**NEG-ONLY tier**: Verifying that a specific percentage actually shifts the compaction trigger requires consuming enough tokens in a live conversation — that is a `lim_it` live-API test. The binary-level contract test asserts only the negative: the binary does not exit non-zero or emit an explicit rejection for this env var. That assertion alone would pass for a nonexistent variable, which is why E76's scan is recorded separately.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E50 | B26 | Doc | Official Claude Code documentation (code.claude.com/docs/en/env-vars) | `CLAUDE_CODE_AUTO_COMPACT_WINDOW` entry | "`CLAUDE_AUTOCOMPACT_PCT_OVERRIDE` is applied as a percentage of this value" |
| E51 | B26 | Test | `../../tests/behavior/b26_autocompact_pct_override.rs` | `b26_autocompact_pct_override_env_var_recognized` | Binary exits 0 and does not emit rejection referencing `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE` when env var is set — negative assertion |
| E76 | B25, B26 | Experiment | Binary string scan — `grep -ac <VAR> ~/.local/share/claude/versions/2.1.220` (2026-08-27) | v2.1.220 native binary | `CLAUDE_CODE_AUTO_COMPACT_WINDOW` = 14, `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE` = 6 — both present. Same scan, same controls as E72, which returned 0 for the two refuted variables: positive control `CLAUDE_CONFIG_DIR` = 28, negative control `TOTALLY_FAKE_VAR_XYZ` = 0. Establishes that these two NEG-ONLY behaviors describe variables the binary actually contains, which the NEG-ONLY test itself cannot show. |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [025_b25_auto_compact_window.md](025_b25_auto_compact_window.md) | Companion env var: token window this percentage applies to |
| behavior | [011_b11_auto_continue_env.md](011_b11_auto_continue_env.md) | `CLAUDE_CODE_AUTO_CONTINUE` env var (same NEG-ONLY pattern) |
| param | [../param/075_autocompact_pct_override.md](../param/075_autocompact_pct_override.md) | Parameter detail: type, default, description |
| test | `../../tests/behavior/b26_autocompact_pct_override.rs` | Invalidation test (NEG-ONLY) |
