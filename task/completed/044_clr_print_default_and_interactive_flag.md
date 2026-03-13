# clr: Print Mode Default When Message Given + `--interactive` Flag

## Goal

`clr "message"` defaults to print mode (captured stdout) instead of interactive TTY.
Bare `clr` (no message) still opens the interactive REPL. A new `--interactive` flag opts
into TTY passthrough when a message is given. Motivated by the current UX failure where
`clr .` opens an interactive session rather than returning capturable output. Testable via
`clr --dry-run "test"` showing `--print` in the command line (without needing to pass `-p`),
and `clr --dry-run --interactive "test"` showing no `--print`.

## In Scope

- `module/claude_runner/src/main.rs`:
  - Add `interactive: bool` field to `CliArgs`
  - Parse `--interactive` in `parse_args()`
  - Update `build_claude_command()`: add `--print` when message given and not `--interactive`
  - Update `main()` dispatch logic: use print path when message + not interactive
  - Update `print_help()`: add `--interactive` line; update `-p` description
  - Update doc comments at top of file
- `module/claude_runner/tests/cli_args_test.rs`:
  - T42: message without `-p` → dry-run shows `--print`
  - T43: `--interactive` with message → dry-run shows no `--print`
  - T44: bare `clr` (no message, no flags) → dry-run shows no `--print`
  - Update T38 if affected by new default
- `module/claude_runner/tests/dry_run_test.rs`:
  - Add: message-without-p default print test
  - Add: `--interactive` suppresses `--print` test
- `module/claude_runner/tests/execution_mode_test.rs`:
  - E12: message without `-p` routes to `run_print_mode()`
  - E13: `--interactive` with message routes to `run_interactive()`

## Out of Scope

- Changes to `claude_runner_core` — only `main.rs` logic changes
- Removing `-p`/`--print` flag (kept for backward compat)
- Changes to `src/lib.rs` (already updated in this session)
- Changes to spec/docs (already applied in this session)
- `src/verbosity.rs` changes

## Description

The spec and docs were updated in this session (2026-03-28) to reflect the new default:
when `[MESSAGE]` is given, `clr` defaults to print mode. `--interactive` is the new
runner-specific flag to opt into TTY passthrough. The code implementation in `src/main.rs`
is the only remaining deliverable. The validation gate is `ctest3` across the
`claude_runner` crate.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   No mocking; real subprocess invocations in tests
-   2-space indent, custom codestyle rules (no `cargo fmt`)

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; confirm constraints for `claude_runner`.
2. **Write Test Matrix** — populate every row before touching any source file.
3. **Write failing tests** — implement test cases from Test Matrix; confirm they fail.
4. **Implement** — update `CliArgs`, `parse_args()`, `build_claude_command()`, `main()`, `print_help()`.
5. **Green state** — `ctest3` must pass with 0 failures and 0 warnings.
6. **Refactor if needed** — no function > 50 lines, no duplication.
7. **Walk Validation Checklist** — every answer must be YES.
8. **Update task status** — set status in `task/readme.md`, move file to `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `clr --dry-run "Fix bug"` | message, no `-p`, no `--interactive` | dry-run output contains `--print` |
| T02 | `clr --dry-run -p "Fix bug"` | explicit `-p` | dry-run output contains `--print` (same as T01) |
| T03 | `clr --dry-run --interactive "Fix bug"` | `--interactive` flag | dry-run output does NOT contain `--print` |
| T04 | `clr --dry-run` | no message | dry-run output does NOT contain `--print` |
| T05 | `clr --dry-run --new-session "Fix bug"` | new session + message | dry-run contains `--print` but not `-c` |
| T06 | `clr --interactive` | flag only, no message | no effect (bare REPL without message) |
| T07 | execution: message no `-p` | default print path | routes to `run_print_mode()`, not `run_interactive()` |
| T08 | execution: `--interactive "msg"` | explicit interactive | routes to `run_interactive()`, not `run_print_mode()` |

## Acceptance Criteria

-   `clr --dry-run "msg"` output contains `--print` (print is the default with message)
-   `clr --dry-run --interactive "msg"` output does NOT contain `--print`
-   `clr --dry-run` (no message) output does NOT contain `--print`
-   `clr -p "msg"` continues to work (backward compat — same as default)
-   `--interactive` appears in `clr --help` output
-   `ctest3` passes clean (all tests, no warnings, clippy clean)

## Validation Checklist

Desired answer for every question is YES.

**Default print mode**
- [ ] Does `clr --dry-run "test"` output include `--print`?
- [ ] Does `clr --dry-run` (no message) output NOT include `--print`?
- [ ] Is `--print` added in `build_claude_command()` when `message.is_some() && !cli.interactive`?

**`--interactive` flag**
- [ ] Does `--interactive` appear in `parse_args()` match arm?
- [ ] Does `clr --dry-run --interactive "test"` output NOT include `--print`?
- [ ] Is `--interactive` listed in `print_help()` output?
- [ ] Does `CliArgs` have `interactive: bool` field?

**Backward compat**
- [ ] Does `clr -p "msg"` still produce `--print` in the command?
- [ ] Does `clr` (bare, no message) still enter interactive REPL path?

**Quality**
- [ ] Does `ctest3` pass with zero failures and zero warnings?
- [ ] Does clippy produce zero warnings for modified code?

## Validation Procedure

### Measurements

**M1 — Default print in dry-run**
Run: `clr --dry-run "test"`
Before (old behavior): `claude -c "test"` (no `--print`)
Expected after: `claude -c --print "test"`
Deviation means: `build_claude_command()` not updated correctly.

**M2 — Interactive flag suppresses print**
Run: `clr --dry-run --interactive "test"`
Expected: `claude -c "test"` (no `--print`)
Deviation means: `--interactive` flag not wired to suppress `--print`.

**M3 — Test count**
Before: 93 tests. Expected after: ≥97 tests (T01–T08 = 4+ new, some may merge).
Deviation means: new tests not added.

### Anti-faking checks

**AF1 — `--interactive` parse failure**
Run: `clr --interactive` — must not error; should open REPL (no message = no effect on mode).

**AF2 — Dry-run execution parity**
`clr --dry-run "test"` must show `--print`; `clr -p --dry-run "test"` must show identical output.

## Outcomes

**All acceptance criteria met:**

- `clr --dry-run "msg"` outputs `--print` in the command (default print when message given)
- `clr --dry-run --interactive "msg"` outputs no `--print`
- `clr --dry-run` (no message) outputs no `--print` (bare REPL unchanged)
- `--interactive` appears in help output; `interactive: bool` field in `CliArgs`
- 103/103 tests pass; `ctest3` clean (0 failures, 0 warnings, clippy clean)

**Implementation summary (`src/main.rs`):**
- `CliArgs.interactive: bool` added
- `"--interactive" => { parsed.interactive = true; }` arm in `parse_args()`
- `build_claude_command()`: `--print` added when `cli.print_mode || (cli.message.is_some() && !cli.interactive)`
- `main()` dispatch: routes to `run_print_mode` on same condition
- `print_help()`: `--interactive` line added between `-p` and `--new-session`
- Module doc comment updated to reflect new mode descriptions

**Test updates:**
- T42–T45 added to `cli_args_test.rs` (all pass)
- 3 tests added to `dry_run_test.rs` (all pass)
- E12, E13 added to `execution_mode_test.rs` (all pass)
- E03 updated: now passes `--interactive "test"` to force interactive mode (message alone now defaults to print)
