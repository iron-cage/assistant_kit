# TSK-071 — Add dry-run mode and compact inspection to `ClaudeCommand`

## Status

✅ (Completed)

## Metadata

- **Value:** 7
- **Easiness:** 8
- **Priority:** 2
- **Safety:** 8
- **Advisability:** 896

## Goal

Add `with_dry_run(bool)` to `ClaudeCommand` and a `describe_compact()` method that renders the full execution plan (working directory + env vars + CLI command) in one clear, copy-pasteable block.

**MOST criteria:**
- **Motivated:** `execute()` is a black box — users cannot see what `claude` invocation it will produce without calling `describe()` and `describe_env()` separately and mentally assembling them.
- **Observable:** `with_dry_run(true)` makes `execute()` return `Ok(ExecutionOutput)` with `stdout = describe_compact()` and no process spawned; `describe_compact()` is also callable standalone for inspection without execution.
- **Scoped:** `src/command.rs` only — no other crate changes.
- **Testable:** `ctest3` green; new tests cover both the no-spawn guarantee and the exact compact output format.

## Description

Two distinct but related gaps in the builder API:

**Gap 1 — dry-run:** There is no way to say "build the command but do not run it". The only inspection paths are `describe()` (CLI string) and `describe_env()` (env var list), but calling `execute()` always spawns a process. Users wiring up automation pipelines need to verify what will be run before enabling live execution.

**Gap 2 — compact inspection:** `describe()` and `describe_env()` are separate methods with no combined view. Reading them together requires two calls and manual formatting. A unified `describe_compact()` should render everything in one block that can be copied straight into a terminal.

**Suggested output for `describe_compact()`:**

```
dir:  /home/user/project
env:  CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000
      CLAUDE_CODE_BASH_TIMEOUT=3600000
      CLAUDE_CODE_BASH_MAX_TIMEOUT=7200000
      CLAUDE_CODE_AUTO_CONTINUE=true
      CLAUDE_CODE_TELEMETRY=false
cmd:  claude --dangerously-skip-permissions "hello world"
```

When no working directory is set, the `dir:` line is omitted. When no env vars are set (all `None`), the `env:` block is omitted. Alignment: values on each `env:` continuation line align with the first value column.

## In Scope

- `with_dry_run(bool)` builder method — sets `dry_run: bool` field on `ClaudeCommand`
- `execute()` and `execute_interactive()` both check `dry_run`; when `true`, skip spawning and return a sentinel result indicating dry-run (details in Acceptance Criteria)
- `describe_compact() -> String` — single-call unified view: `dir:` / `env:` / `cmd:` block
- Integration tests for all new behaviour
- Update `docs/claude_params/readme.md` — add `dry_run` row to Builder column (or Notes)

## Out of Scope

- Adding `--dry-run` as a `claude` CLI flag (not a real claude flag; builder-only concept)
- Changing existing `describe()` or `describe_env()` signatures
- Any changes outside `src/command.rs` and `tests/`

## Requirements

- Follow `code_design.rulebook.md` — TDD red-green-refactor
- Follow `codebase_hygiene.rulebook.md` — no mocking, real process checks
- Follow `test_organization.rulebook.md` — tests in `tests/` only
- Follow `code_style.rulebook.md` — 2-space indent, custom codestyle

## Acceptance Criteria

- `ClaudeCommand::new().with_dry_run(true).execute()` returns `Ok(ExecutionOutput)` with `stdout == describe_compact()`, `stderr == ""`, `exit_code == 0` — no child process launched
- `ClaudeCommand::new().with_dry_run(true).execute_interactive()` returns `Ok(ExitStatus)` with code 0 — no child process launched; prints `describe_compact()` to stdout before returning
- `with_dry_run(false)` (or default) behaves identically to current behaviour
- `describe_compact()` output contains `cmd:` line matching `describe()` output
- `describe_compact()` output contains all env var lines from `describe_env()`
- `describe_compact()` omits `dir:` line when no working directory set
- `describe_compact()` omits `env:` block when no env vars configured
- All new and existing tests pass; `ctest3` green; zero warnings

## Work Procedure

1. Read applicable rulebooks via `kbase .rulebooks`
2. Write failing tests: dry-run no-spawn, dry-run return values, `describe_compact()` format (RED)
3. Add `dry_run: bool` field to `ClaudeCommand` struct; default `false`
4. Add `with_dry_run(bool)` builder method
5. Guard `execute()` and `execute_interactive()` with `if self.dry_run` early-return
6. Implement `describe_compact()` — assemble `dir:` / `env:` / `cmd:` block
7. All tests green (GREEN)
8. Refactor if needed; verify `ctest3`
9. Update `docs/claude_params/readme.md` to document `dry_run`
10. Update task status → ✅

## Validation Checklist

| # | Question | Desirable | Fail Action |
|---|----------|-----------|-------------|
| 1 | Does `with_dry_run(true).execute()` return without spawning a process? | YES | Fix dry-run guard in `execute()` |
| 2 | Does `with_dry_run(true).execute_interactive()` return without spawning? | YES | Fix dry-run guard in `execute_interactive()` |
| 3 | Does `describe_compact()` output include the exact `describe()` command line? | YES | Fix `cmd:` assembly |
| 4 | Does `describe_compact()` output include all env vars from `describe_env()`? | YES | Fix `env:` assembly |
| 5 | Is `dir:` line absent when no working directory is set? | YES | Add conditional |
| 6 | Is `env:` block absent when all env fields are `None`? | YES | Add conditional |
| 7 | Does `ctest3` pass with zero warnings? | YES | Fix failures/warnings |

## Validation Procedure

### Measurements

| # | Metric | Before | After | Command |
|---|--------|--------|-------|---------|
| M1 | Public `with_*` methods on `ClaudeCommand` | 22 | 23 | `grep -c 'pub fn with_' src/command.rs` |
| M2 | `ctest3` result | passing | passing | `ctest3` |

### Anti-faking Checks

- AF1: Run a test that calls `with_dry_run(true).execute()` and asserts no child `claude` process appears in `ps` output during the call — confirms the no-spawn guarantee is real, not just a flag check
- AF2: Call `with_dry_run(true).execute()` on a fully configured builder; assert `result.stdout == cmd.describe_compact()` — confirms dry-run stdout IS the compact description, not a stub string
- AF3: Call `describe_compact()` on a fully configured builder and verify every line of `describe_env()` appears verbatim in the output — confirms env block is not hand-crafted
