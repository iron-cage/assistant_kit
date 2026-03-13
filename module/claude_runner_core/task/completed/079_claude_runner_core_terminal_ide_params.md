# TSK-079 — Add typed builder methods for terminal and IDE integration parameters

## Status

✅ (Completed)

## Metadata

- **Value:** 4
- **Easiness:** 7
- **Priority:** 2
- **Safety:** 8
- **Advisability:** 448

## Goal

Add typed `with_*` builder methods for `worktree`, `tmux`, `ide`, and `chrome`.

**MOST criteria:**
- **Motivated:** Worktree and tmux integration are useful in automated multi-session workflows; typed methods are cleaner than raw flag strings.
- **Observable:** Four new typed methods; `describe()` reflects correct flag syntax.
- **Scoped:** `src/command.rs` only.
- **Testable:** `ctest3` green; tests verify flag rendering.

## Description

`-w` / `--worktree [name]` creates a git worktree for the session with an optional name. `--tmux` creates a tmux session for the worktree. `--ide` auto-connects to IDE on startup. `--chrome` / `--no-chrome` toggles Claude-in-Chrome integration.

`worktree` takes an optional name — the builder should support both `with_worktree(None)` (auto-name) and `with_worktree(Some("name"))`. `chrome` is a tri-state (`Some(true)` = `--chrome`, `Some(false)` = `--no-chrome`, `None` = omit).

## In Scope

- `with_worktree(name: Option<&str>)` — adds `-w` alone or `-w <name>`
- `with_tmux(bool)` — adds `--tmux` when true
- `with_ide(bool)` — adds `--ide` when true
- `with_chrome(enabled: Option<bool>)` — adds `--chrome`, `--no-chrome`, or nothing
- Integration tests for all four methods
- Update `docs/claude_params/readme.md` Builder column for all four params

## Out of Scope

- Worktree existence validation
- tmux session management beyond passing the flag

## Requirements

- Follow `code_design.rulebook.md` — TDD red-green-refactor
- Follow `codebase_hygiene.rulebook.md` — no mocking
- Follow `test_organization.rulebook.md` — tests in `tests/` only
- Follow `code_style.rulebook.md` — 2-space indent, custom codestyle

## Acceptance Criteria

- `with_worktree(None)` adds `-w` with no name
- `with_worktree(Some("feature"))` adds `-w feature`
- `with_tmux(true)` adds `--tmux`; `with_tmux(false)` adds nothing
- `with_ide(true)` adds `--ide`
- `with_chrome(Some(true))` adds `--chrome`
- `with_chrome(Some(false))` adds `--no-chrome`
- `with_chrome(None)` adds nothing
- `ctest3` green; zero warnings

## Work Procedure

1. Read applicable rulebooks via `kbase .rulebooks`
2. Write failing tests for all four methods (RED)
3. Implement all four `with_*` methods in `src/command.rs`
4. All tests green (GREEN)
5. Refactor if needed; verify `ctest3`
6. Update `docs/claude_params/readme.md` Builder column
7. Update task status → ✅

## Validation Checklist

| # | Question | Desirable | Fail Action |
|---|----------|-----------|-------------|
| 1 | Does `with_worktree(None)` add `-w` without trailing value? | YES | Fix option handling |
| 2 | Does `with_chrome(Some(false))` add `--no-chrome` (not just nothing)? | YES | Fix tri-state logic |
| 3 | Does `with_chrome(None)` add nothing? | YES | Fix option guard |
| 4 | Does `ctest3` pass? | YES | Fix failures |

## Validation Procedure

### Measurements

| # | Metric | Before | After | Command |
|---|--------|--------|-------|---------|
| M1 | Public `with_*` methods on `ClaudeCommand` | 57 | 61 | `grep -c 'pub fn with_' src/command.rs` |
| M2 | `ctest3` result | passing | passing | `ctest3` |

### Anti-faking Checks

- AF1: Build with `with_chrome(Some(false))` and assert `describe()` contains `--no-chrome` — confirms the negative flag is explicitly emitted, not just omitted
- AF2: Build with `with_worktree(Some("feat"))` and `with_tmux(true)`; assert both `-w feat` and `--tmux` appear together
