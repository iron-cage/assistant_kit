# Behavior B23: CLAUDE_CODE_SESSION_DIR Overrides Storage Directory

### Scope

- **Purpose**: Document that `CLAUDE_CODE_SESSION_DIR` env var redirects session JSONL storage to a custom path.
- **Responsibility**: Authoritative instance for behavior B23 — defines the behavior statement, certainty level, and supporting evidence. Tier is NEG-ONLY.
- **In Scope**: `CLAUDE_CODE_SESSION_DIR` env var; custom path override; per-invocation scope; CI/multi-user use cases.
- **Out of Scope**: `--no-session-persistence` that disables storage entirely (→ [B22](022_b22_no_session_persistence.md)); default path encoding (→ [B9](009_b9_storage_path_encoding.md)).

### Behavior

**Status**: 🎯 Observed | **Certainty**: 80% | **Tier**: NEG-ONLY | **Since**: pre-v1.0 | **Evidence**: E43, E44

When set, session `.jsonl` files are read from and written to the specified path instead of the default `~/.claude/projects/{encoded-path}/`. Useful for:
- CI pipelines where sessions must be stored in a known, writable location
- Multi-user environments where each user has a separate session directory

The env var affects only the current invocation; other Claude Code behavior is unchanged.

**NEG-ONLY tier**: The test asserts that the binary does not explicitly reject `CLAUDE_CODE_SESSION_DIR` env var at startup — it cannot confirm whether the env var is actively used or silently ignored.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E43 | B23 | Doc | `../params/057_session_dir.md` | Description | Documents `CLAUDE_CODE_SESSION_DIR` env var that overrides session storage directory |
| E44 | B23 | Test | `../../tests/behavior/b23_session_dir_override.rs` | `b23_session_dir_env_var_not_rejected` | Binary does not explicitly reject `CLAUDE_CODE_SESSION_DIR` env var at startup |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [009_b9_storage_path_encoding.md](009_b9_storage_path_encoding.md) | Default path encoding that this env var overrides |
| behavior | [011_b11_auto_continue_env.md](011_b11_auto_continue_env.md) | `CLAUDE_CODE_AUTO_CONTINUE` env var (related env var, same NEG-ONLY pattern) |
| behavior | [022_b22_no_session_persistence.md](022_b22_no_session_persistence.md) | `--no-session-persistence` (disables rather than redirects) |
| params | `../params/057_session_dir.md` | Canonical parameter definition |
| test | `../../tests/behavior/b23_session_dir_override.rs` | Invalidation test (NEG-ONLY) |
