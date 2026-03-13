---
id: 058
title: Implement default-on skip-permissions in clr binary
status: ✅ Complete
category: feature
created: 2026-03-31
completed: 2026-03-31
---

## Goal

Make `--dangerously-skip-permissions` default-on in `clr`; expose `--no-skip-permissions`
as the opt-out. Update all affected tests and documentation files.

## Scope

- `module/claude_runner/src/main.rs`
- `module/claude_runner/tests/cli_args_test.rs`
- `module/claude_runner/tests/dry_run_test.rs`
- `module/claude_runner/tests/manual/readme.md`
- `module/claude_runner/readme.md`

## Implementation

1. `CliArgs`: rename `skip_permissions: bool` → `no_skip_permissions: bool`
   (`#[derive(Default)]` makes it `false`, meaning bypass is ON by default)
2. `parse_args()`: remove `--dangerously-skip-permissions` arm; add `--no-skip-permissions`
3. `build_claude_command()`: change condition from `if cli.skip_permissions` to `if !cli.no_skip_permissions`
4. `print_help()`: replace old flag line with `--no-skip-permissions` line

## Done When

- `CliArgs.no_skip_permissions: bool` (default false via `#[derive(Default)]`)
- `parse_args()` handles `--no-skip-permissions`; `--dangerously-skip-permissions` removed
- `build_claude_command()` always injects unless `no_skip_permissions` is true
- `print_help()` shows `--no-skip-permissions`
- Tests T05 redesigned (default-on), T10 fixed, T16 fixed, T38 updated, T46 added
- `dry_run_test.rs::combined_flags_all_appear` updated
- `tests/manual/readme.md` TC-6 updated
- `readme.md` flags list updated
- `w3 .test l::3` passes
