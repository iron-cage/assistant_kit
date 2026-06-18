# Behavior B26: CLAUDE_AUTOCOMPACT_PCT_OVERRIDE Env Var

### Scope

- **Purpose**: Document that `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE` env var overrides the auto-compaction percentage threshold in the `claude` binary.
- **Responsibility**: Authoritative instance for behavior B26 — defines the behavior statement, certainty level, and supporting evidence. Tier is NEG-ONLY.
- **In Scope**: `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE` env var; naming asymmetry (`CLAUDE_` prefix without `_CODE_`).
- **Out of Scope**: Token window size (-> [B25](025_b25_auto_compact_window.md)); session storage directory (-> [B23](023_b23_session_dir_override.md)).

### Behavior

**Status**: Observed | **Certainty**: 80% | **Tier**: NEG-ONLY | **Evidence**: E50, E51

`CLAUDE_AUTOCOMPACT_PCT_OVERRIDE` environment variable overrides the auto-compaction percentage: compaction fires when `used_tokens / window >= pct / 100`. The percentage is applied to the effective window set by `CLAUDE_CODE_AUTO_COMPACT_WINDOW`.

Note the naming asymmetry: this variable uses a `CLAUDE_` prefix without `_CODE_`, unlike most other Claude Code env vars. The default percentage is not publicly documented.

**NEG-ONLY tier**: Verifying that a specific percentage actually shifts the compaction trigger requires consuming enough tokens in a live conversation — that is a `lim_it` live-API test. The binary-level contract test asserts only the negative: the binary does not exit non-zero or emit an explicit rejection for this env var.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E50 | B26 | Doc | Official Claude Code documentation (code.claude.com/docs/en/env-vars) | `CLAUDE_CODE_AUTO_COMPACT_WINDOW` entry | "`CLAUDE_AUTOCOMPACT_PCT_OVERRIDE` is applied as a percentage of this value" |
| E51 | B26 | Test | `../../tests/behavior/b26_autocompact_pct_override.rs` | `b26_autocompact_pct_override_env_var_recognized` | Binary exits 0 and does not emit rejection referencing `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE` when env var is set — negative assertion |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [025_b25_auto_compact_window.md](025_b25_auto_compact_window.md) | Companion env var: token window this percentage applies to |
| behavior | [011_b11_auto_continue_env.md](011_b11_auto_continue_env.md) | `CLAUDE_CODE_AUTO_CONTINUE` env var (same NEG-ONLY pattern) |
| param | [../params/075_autocompact_pct_override.md](../params/075_autocompact_pct_override.md) | Parameter detail: type, default, description |
| test | `../../tests/behavior/b26_autocompact_pct_override.rs` | Invalidation test (NEG-ONLY) |
