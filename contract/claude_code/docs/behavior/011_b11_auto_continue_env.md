# Behavior B11: CLAUDE_CODE_AUTO_CONTINUE Env Var

### Scope

- **Purpose**: Document that `CLAUDE_CODE_AUTO_CONTINUE` env var enables automated continuation mode in the `claude` binary.
- **Responsibility**: Authoritative instance for behavior B11 — defines the behavior statement, certainty level, and supporting evidence. Tier is NEG-ONLY.
- **In Scope**: `CLAUDE_CODE_AUTO_CONTINUE` env var; how it is set by the `clr` runner; NEG-ONLY test tier explanation.
- **Out of Scope**: `CLAUDE_CODE_SESSION_DIR` (different env var, different purpose, → [B23](023_b23_session_dir_override.md)).

### Behavior

**Status**: 🎯 Observed | **Certainty**: 85% | **Tier**: NEG-ONLY | **Since**: pre-v1.0 | **Evidence**: E10, E21

`CLAUDE_CODE_AUTO_CONTINUE` environment variable enables automated continuation mode in the `claude` binary.

This env var is set by the `clr` runner before spawning `claude` (via `cmd.env("CLAUDE_CODE_AUTO_CONTINUE", auto_continue.to_string())`). The binary's acceptance of this env var cannot be confirmed beyond the negative assertion that it does not explicitly reject it.

**NEG-ONLY tier**: The test asserts that the binary does not print `CLAUDE_CODE_AUTO_CONTINUE` in stderr when the env var is set — it cannot distinguish acceptance from silent ignore. Exit code is trivially 0 and cannot discriminate.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E10 | B11 | Code | `../../../../module/claude_runner_core/src/command.rs` | line 647-648 | `cmd.env("CLAUDE_CODE_AUTO_CONTINUE", auto_continue.to_string())` — env var set before spawning `claude` |
| E21 | B11 | Test | `../../tests/behavior/b11_auto_continue.rs` | `b11_auto_continue_env_var_recognized` | Binary does not print `CLAUDE_CODE_AUTO_CONTINUE` in stderr when env var is set — negative assertion |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [023_b23_session_dir_override.md](023_b23_session_dir_override.md) | `CLAUDE_CODE_SESSION_DIR` env var (different env var) |
| behavior | [025_b25_auto_compact_window.md](025_b25_auto_compact_window.md) | `CLAUDE_CODE_AUTO_COMPACT_WINDOW` env var (same NEG-ONLY pattern) |
| behavior | [026_b26_autocompact_pct_override.md](026_b26_autocompact_pct_override.md) | `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE` env var (same NEG-ONLY pattern) |
| test | `../../tests/behavior/b11_auto_continue.rs` | Invalidation test (NEG-ONLY) |
