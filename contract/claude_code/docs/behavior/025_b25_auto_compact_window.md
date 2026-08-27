# Behavior B25: CLAUDE_CODE_AUTO_COMPACT_WINDOW Env Var

### Scope

- **Purpose**: Document that `CLAUDE_CODE_AUTO_COMPACT_WINDOW` env var controls the token window threshold for auto-compaction in the `claude` binary.
- **Responsibility**: Authoritative instance for behavior B25 — defines the behavior statement, certainty level, and supporting evidence. Tier is NEG-ONLY.
- **In Scope**: `CLAUDE_CODE_AUTO_COMPACT_WINDOW` env var; introduced in v2.1.75 for 1M context window tuning.
- **Out of Scope**: Compaction percentage override (-> [B26](026_b26_autocompact_pct_override.md)); session storage directory (-> [B23](023_b23_session_dir_override.md)).

### Behavior

**Status**: 🎯 Observed | **Certainty**: 90% | **Tier**: NEG-ONLY (existence separately confirmed — see below) | **Since**: v2.1.75 | **Evidence**: E48, E49, E76

`CLAUDE_CODE_AUTO_COMPACT_WINDOW` environment variable sets the effective context window in tokens for auto-compaction threshold calculations. When the active conversation approaches this token count (as a percentage governed by `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE`), Claude Code automatically compacts the context.

Introduced in Claude Code v2.1.75 (2026-03-13) alongside 1M context windows for Opus 4.6. The value is capped at the model's actual context window; setting a lower value triggers compaction earlier. Cannot increase the effective window beyond the model limit.

**Precedence.** Official documentation states this env var takes precedence over the other three ways of setting the same window: the `/autocompact` slash command, the `--autocompact` CLI flag, and the `autoCompactWindow` settings key. When more than one is set, the env var wins.

**Existence confirmed.** Unlike [B11](011_b11_auto_continue_env.md) and [B23](023_b23_session_dir_override.md) — both NEG-ONLY behaviors refuted on 2026-08-27 because the variables did not exist in the binary at all — this variable is present: 14 occurrences in v2.1.220 (E76). It is also documented officially (E48). Certainty is raised from 85% to 90% on that basis: existence is now established, and only the precise runtime semantics remain inferred.

**NEG-ONLY tier**: Full validation requires consuming approximately 200K tokens of conversation to trigger compaction — that is a `lim_it` live-API test beyond the scope of invalidation tests. The binary-level contract test asserts only the negative: the binary does not exit non-zero or print an explicit rejection for this env var. That assertion alone would pass for a nonexistent variable, which is why E76's scan is recorded separately rather than treated as redundant.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E48 | B25 | Doc | Official Claude Code documentation (code.claude.com/docs/en/env-vars) | `CLAUDE_CODE_AUTO_COMPACT_WINDOW` entry | "Set the context capacity in tokens used for auto-compaction calculations. Defaults to the model's context window: 200K for standard models or 1M for extended context models." |
| E49 | B25 | Test | `../../tests/behavior/b25_auto_compact_window.rs` | `b25_auto_compact_window_env_var_recognized` | Binary exits 0 and does not emit rejection referencing `CLAUDE_CODE_AUTO_COMPACT_WINDOW` when env var is set — negative assertion |
| E76 | B25, B26 | Experiment | Binary string scan — `grep -ac <VAR> ~/.local/share/claude/versions/2.1.220` (2026-08-27) | v2.1.220 native binary | `CLAUDE_CODE_AUTO_COMPACT_WINDOW` = 14, `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE` = 6 — both present. Same scan, same controls as E72, which returned 0 for the two refuted variables: positive control `CLAUDE_CONFIG_DIR` = 28, negative control `TOTALLY_FAKE_VAR_XYZ` = 0. Establishes that these two NEG-ONLY behaviors describe variables the binary actually contains, which the NEG-ONLY test itself cannot show. |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [026_b26_autocompact_pct_override.md](026_b26_autocompact_pct_override.md) | Companion env var: percentage applied to this window |
| behavior | [011_b11_auto_continue_env.md](011_b11_auto_continue_env.md) | `CLAUDE_CODE_AUTO_CONTINUE` env var (same NEG-ONLY pattern) |
| param | [../param/074_auto_compact_window.md](../param/074_auto_compact_window.md) | Parameter detail: type, default, description |
| test | `../../tests/behavior/b25_auto_compact_window.rs` | Invalidation test (NEG-ONLY) |
