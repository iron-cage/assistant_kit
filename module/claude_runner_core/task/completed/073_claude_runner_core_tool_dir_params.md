# TSK-073 — Add typed builder methods for tool and directory control parameters

## Status

✅ (Completed)

## Metadata

- **Value:** 8
- **Easiness:** 7
- **Priority:** 2
- **Safety:** 7
- **Advisability:** 784

## Goal

Add typed `with_*` builder methods for `add_dir`, `allowed_tools`, `disallowed_tools`, and `tools`.

**MOST criteria:**
- **Motivated:** Tool allowlisting and directory access are critical for sandboxed automation — without typed methods, users must hand-craft raw flag strings and risk subtle mistakes.
- **Observable:** Four new typed methods; `describe()` shows correct multi-value flag syntax.
- **Scoped:** `src/command.rs` only.
- **Testable:** `ctest3` green; tests verify correct multi-value flag rendering.

## Description

These four parameters control what tools claude can use and what filesystem paths it can access. In automation contexts they are the primary security surface. All four accept multiple values (space-separated or repeated flags). The builder should accept `IntoIterator<Item = S>` to match the multi-value pattern cleanly.

`--add-dir` expands the allowed filesystem scope beyond the working directory. `--allowed-tools` restricts which tools are available. `--disallowed-tools` blocks specific tools. `--tools` overrides the full available set.

## In Scope

- `with_add_dir<I, S>(dirs: I)` — adds `--add-dir <dir>` for each dir
- `with_allowed_tools<I, S>(tools: I)` — adds `--allowed-tools <tool>...`
- `with_disallowed_tools<I, S>(tools: I)` — adds `--disallowed-tools <tool>...`
- `with_tools<I, S>(tools: I)` — adds `--tools <tool>...`
- Integration tests for all four (single value, multiple values, empty iterator)
- Update `docs/claude_params/readme.md` Builder column for all four params

## Out of Scope

- Validation of tool names against a known list
- Path existence checking for `add_dir`

## Requirements

- Follow `code_design.rulebook.md` — TDD red-green-refactor
- Follow `codebase_hygiene.rulebook.md` — no mocking
- Follow `test_organization.rulebook.md` — tests in `tests/` only
- Follow `code_style.rulebook.md` — 2-space indent, custom codestyle

## Acceptance Criteria

- `with_add_dir(["/tmp", "/home/user/lib"])` adds `--add-dir /tmp --add-dir /home/user/lib`
- `with_allowed_tools(["Bash", "Read"])` adds `--allowed-tools Bash --allowed-tools Read`
- `with_disallowed_tools(["Write"])` adds `--disallowed-tools Write`
- `with_tools(["default"])` adds `--tools default`
- Empty iterator adds nothing to the command
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
| 1 | Does `with_add_dir(["/a", "/b"])` produce two `--add-dir` flags? | YES | Fix iteration |
| 2 | Does `with_allowed_tools([])` add nothing? | YES | Fix empty guard |
| 3 | Does `with_disallowed_tools(["Write"])` add `--disallowed-tools Write`? | YES | Fix method |
| 4 | Does `ctest3` pass? | YES | Fix failures |

## Validation Procedure

### Measurements

| # | Metric | Before | After | Command |
|---|--------|--------|-------|---------|
| M1 | Public `with_*` methods on `ClaudeCommand` | 29 | 33 | `grep -c 'pub fn with_' src/command.rs` |
| M2 | `ctest3` result | passing | passing | `ctest3` |

### Anti-faking Checks

- AF1: Build command with `with_add_dir(["/tmp", "/etc"])` and count `--add-dir` occurrences in `describe()` — must be exactly 2
- AF2: Build with `with_allowed_tools(["Bash"])` and `with_disallowed_tools(["Write"])` simultaneously; assert both flags appear without interference
