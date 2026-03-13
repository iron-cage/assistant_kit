# TSK-078 — Add typed builder methods for debug and advanced CLI parameters

## Status

✅ (Completed)

## Metadata

- **Value:** 5
- **Easiness:** 7
- **Priority:** 2
- **Safety:** 8
- **Advisability:** 560

## Goal

Add typed `with_*` builder methods for `debug`, `debug_file`, `betas`, `brief`, `disable_slash_commands`, and `file`.

**MOST criteria:**
- **Motivated:** Debug mode and file resource loading are useful for automated diagnostic pipelines; typed methods reduce error surface vs raw arg strings.
- **Observable:** Six new typed methods; `describe()` reflects correct flag syntax.
- **Scoped:** `src/command.rs` only.
- **Testable:** `ctest3` green; tests verify flag rendering.

## Description

`-d` / `--debug [filter]` enables debug output with an optional category filter string. `--debug-file <path>` redirects debug logs to a file. `--betas <betas...>` enables beta API headers (multi-value; API key users only). `--brief` enables the `SendUserMessage` tool for sub-agents. `--disable-slash-commands` disables all slash command skills. `--file <specs...>` downloads file resources at startup (multi-value).

`--debug` takes an optional argument — the builder should support both `with_debug(None)` (all categories) and `with_debug(Some("filter"))`.

## In Scope

- `with_debug(filter: Option<&str>)` — adds `-d` alone or `-d <filter>`
- `with_debug_file<S: Into<String>>(path: S)` — adds `--debug-file <path>`
- `with_betas<I, S>(betas: I)` — adds `--betas <beta>...`
- `with_brief(bool)` — adds `--brief` when true
- `with_disable_slash_commands(bool)` — adds `--disable-slash-commands` when true
- `with_file<I, S>(specs: I)` — adds `--file <spec>` for each
- Integration tests for all six methods
- Update `docs/claude_params/readme.md` Builder column for all six params

## Out of Scope

- `mcp_debug` (`--mcp-debug`) — deprecated, excluded

## Requirements

- Follow `code_design.rulebook.md` — TDD red-green-refactor
- Follow `codebase_hygiene.rulebook.md` — no mocking
- Follow `test_organization.rulebook.md` — tests in `tests/` only
- Follow `code_style.rulebook.md` — 2-space indent, custom codestyle

## Acceptance Criteria

- `with_debug(None)` adds `-d` with no filter
- `with_debug(Some("mcp"))` adds `-d mcp`
- `with_debug_file("/tmp/debug.log")` adds `--debug-file /tmp/debug.log`
- `with_betas(["beta1", "beta2"])` adds two `--betas` flags
- `with_brief(true)` adds `--brief`; `with_brief(false)` adds nothing
- `with_disable_slash_commands(true)` adds `--disable-slash-commands`
- `with_file(["spec1", "spec2"])` adds two `--file` flags
- `ctest3` green; zero warnings

## Work Procedure

1. Read applicable rulebooks via `kbase .rulebooks`
2. Write failing tests for all six methods (RED)
3. Implement all six `with_*` methods in `src/command.rs`
4. All tests green (GREEN)
5. Refactor if needed; verify `ctest3`
6. Update `docs/claude_params/readme.md` Builder column
7. Update task status → ✅

## Validation Checklist

| # | Question | Desirable | Fail Action |
|---|----------|-----------|-------------|
| 1 | Does `with_debug(None)` add `-d` without trailing value? | YES | Fix option handling |
| 2 | Does `with_debug(Some("mcp"))` add `-d mcp`? | YES | Fix method |
| 3 | Does `with_betas(["b1", "b2"])` produce two `--betas` flags? | YES | Fix iteration |
| 4 | Does `ctest3` pass? | YES | Fix failures |

## Validation Procedure

### Measurements

| # | Metric | Before | After | Command |
|---|--------|--------|-------|---------|
| M1 | Public `with_*` methods on `ClaudeCommand` | 51 | 57 | `grep -c 'pub fn with_' src/command.rs` |
| M2 | `ctest3` result | passing | passing | `ctest3` |

### Anti-faking Checks

- AF1: Build with `with_debug(None)` and assert `describe()` contains `-d` but NOT `-d ` followed by a word — confirms optional-filter handling
