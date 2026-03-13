# TSK-077 — Add typed builder methods for MCP and extension parameters

## Status

✅ (Completed)

## Metadata

- **Value:** 5
- **Easiness:** 6
- **Priority:** 2
- **Safety:** 7
- **Advisability:** 420

## Goal

Add typed `with_*` builder methods for `mcp_config`, `strict_mcp_config`, `settings`, `setting_sources`, `agent`, `agents`, and `plugin_dir`.

**MOST criteria:**
- **Motivated:** MCP server configuration and agent overrides are key extension points for advanced automation; raw `with_arg()` calls are error-prone for multi-value flags.
- **Observable:** Seven new typed methods; `describe()` reflects correct flag syntax.
- **Scoped:** `src/command.rs` only.
- **Testable:** `ctest3` green; tests verify flag rendering.

## Description

`--mcp-config <configs...>` loads MCP servers from JSON config files (multi-value). `--strict-mcp-config` disables all non-`--mcp-config` MCP servers. `--settings <file-or-json>` loads a settings file or JSON string. `--setting-sources <sources>` filters which setting sources load. `--agent <agent>` overrides the agent for the session. `--agents <json>` defines custom agents as JSON. `--plugin-dir <paths...>` loads plugins from directories (multi-value).

`mcp_debug` (`--mcp-debug`) is deprecated in favor of `--debug` and is intentionally excluded.

## In Scope

- `with_mcp_config<I, S>(configs: I)` — adds `--mcp-config <path>` for each
- `with_strict_mcp_config(bool)` — adds `--strict-mcp-config` when true
- `with_settings<S: Into<String>>(settings: S)` — adds `--settings <value>`
- `with_setting_sources<S: Into<String>>(sources: S)` — adds `--setting-sources <value>`
- `with_agent<S: Into<String>>(agent: S)` — adds `--agent <agent>`
- `with_agents<S: Into<String>>(json: S)` — adds `--agents <json>`
- `with_plugin_dir<I, S>(dirs: I)` — adds `--plugin-dir <path>` for each
- Integration tests for all seven methods
- Update `docs/claude_params/readme.md` Builder column for all seven params

## Out of Scope

- `mcp_debug` — deprecated, intentionally omitted
- JSON validation for `--agents` value

## Requirements

- Follow `code_design.rulebook.md` — TDD red-green-refactor
- Follow `codebase_hygiene.rulebook.md` — no mocking
- Follow `test_organization.rulebook.md` — tests in `tests/` only
- Follow `code_style.rulebook.md` — 2-space indent, custom codestyle

## Acceptance Criteria

- `with_mcp_config(["/path/a.json", "/path/b.json"])` adds two `--mcp-config` flags
- `with_strict_mcp_config(true)` adds `--strict-mcp-config`
- `with_settings("/path/settings.json")` adds `--settings /path/settings.json`
- `with_agent("reviewer")` adds `--agent reviewer`
- `with_plugin_dir(["/plugins"])` adds `--plugin-dir /plugins`
- Boolean methods with `false` add nothing
- `ctest3` green; zero warnings

## Work Procedure

1. Read applicable rulebooks via `kbase .rulebooks`
2. Write failing tests for all seven methods (RED)
3. Implement all seven `with_*` methods in `src/command.rs`
4. All tests green (GREEN)
5. Refactor if needed; verify `ctest3`
6. Update `docs/claude_params/readme.md` Builder column
7. Update task status → ✅

## Validation Checklist

| # | Question | Desirable | Fail Action |
|---|----------|-----------|-------------|
| 1 | Does `with_mcp_config(["a", "b"])` produce two `--mcp-config` flags? | YES | Fix iteration |
| 2 | Does `with_strict_mcp_config(false)` add nothing? | YES | Fix boolean guard |
| 3 | Does `with_plugin_dir(["/x", "/y"])` produce two `--plugin-dir` flags? | YES | Fix iteration |
| 4 | Does `ctest3` pass? | YES | Fix failures |

## Validation Procedure

### Measurements

| # | Metric | Before | After | Command |
|---|--------|--------|-------|---------|
| M1 | Public `with_*` methods on `ClaudeCommand` | 44 | 51 | `grep -c 'pub fn with_' src/command.rs` |
| M2 | `ctest3` result | passing | passing | `ctest3` |

### Anti-faking Checks

- AF1: Build with `with_mcp_config(["a.json"])` and `with_strict_mcp_config(true)`; assert both flags appear — confirms they work together as intended
