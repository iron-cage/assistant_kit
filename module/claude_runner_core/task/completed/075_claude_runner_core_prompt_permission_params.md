# TSK-075 — Add typed builder methods for system prompt and permission parameters

## Status

✅ (Completed)

## Metadata

- **Value:** 7
- **Easiness:** 8
- **Priority:** 2
- **Safety:** 7
- **Advisability:** 784

## Goal

Add typed `with_*` builder methods for `append_system_prompt`, `permission_mode`, and `allow_dangerously_skip_permissions`.

**MOST criteria:**
- **Motivated:** System prompt augmentation and permission mode selection are common in automation setups that need to inject context without replacing the full system prompt.
- **Observable:** Three new typed methods; `describe()` reflects correct flag syntax.
- **Scoped:** `src/command.rs` and `src/types.rs` only.
- **Testable:** `ctest3` green; tests verify each flag appears correctly.

## Description

`--append-system-prompt` appends text to the default system prompt rather than replacing it (complementary to the existing `with_system_prompt()` which replaces). `--permission-mode <mode>` sets fine-grained permission mode (`default`, `acceptEdits`, `bypassPermissions`). `--allow-dangerously-skip-permissions` enables `--dangerously-skip-permissions` as an option without activating it — distinct from `with_skip_permissions()` which activates it unconditionally.

A `PermissionMode` enum should be added to `src/types.rs`.

## In Scope

- `PermissionMode` enum: `Default`, `AcceptEdits`, `BypassPermissions` — add to `src/types.rs`
- `with_append_system_prompt<S: Into<String>>(prompt: S)` — adds `--append-system-prompt <prompt>`
- `with_permission_mode(PermissionMode)` — adds `--permission-mode <mode>`
- `with_allow_dangerously_skip_permissions(bool)` — adds `--allow-dangerously-skip-permissions` when true
- Integration tests for all three methods
- Update `docs/claude_params/readme.md` Builder column for all three params

## Out of Scope

- Changes to the existing `with_system_prompt()` or `with_skip_permissions()` methods

## Requirements

- Follow `code_design.rulebook.md` — TDD red-green-refactor
- Follow `codebase_hygiene.rulebook.md` — no mocking
- Follow `test_organization.rulebook.md` — tests in `tests/` only
- Follow `code_style.rulebook.md` — 2-space indent, custom codestyle

## Acceptance Criteria

- `with_append_system_prompt("You are cautious")` adds `--append-system-prompt You are cautious`
- `with_permission_mode(PermissionMode::AcceptEdits)` adds `--permission-mode acceptEdits`
- `with_allow_dangerously_skip_permissions(true)` adds `--allow-dangerously-skip-permissions`
- `with_allow_dangerously_skip_permissions(false)` adds nothing
- `ctest3` green; zero warnings

## Work Procedure

1. Read applicable rulebooks via `kbase .rulebooks`
2. Add `PermissionMode` enum to `src/types.rs`
3. Write failing tests for all three methods (RED)
4. Implement all three `with_*` methods in `src/command.rs`
5. All tests green (GREEN)
6. Refactor if needed; verify `ctest3`
7. Update `docs/claude_params/readme.md` Builder column
8. Update task status → ✅

## Validation Checklist

| # | Question | Desirable | Fail Action |
|---|----------|-----------|-------------|
| 1 | Does `with_append_system_prompt(s)` add `--append-system-prompt`? | YES | Fix method |
| 2 | Does `with_permission_mode(BypassPermissions)` add `--permission-mode bypassPermissions`? | YES | Fix enum `as_str()` |
| 3 | Does `with_allow_dangerously_skip_permissions(false)` add nothing? | YES | Fix boolean guard |
| 4 | Does `ctest3` pass? | YES | Fix failures |

## Validation Procedure

### Measurements

| # | Metric | Before | After | Command |
|---|--------|--------|-------|---------|
| M1 | Public `with_*` methods on `ClaudeCommand` | 38 | 41 | `grep -c 'pub fn with_' src/command.rs` |
| M2 | `ctest3` result | passing | passing | `ctest3` |

### Anti-faking Checks

- AF1: Build with `with_system_prompt("A")` and `with_append_system_prompt("B")` simultaneously; assert both `--system-prompt` and `--append-system-prompt` appear — confirms they are independent flags, not conflicting
