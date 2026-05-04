# Remove POSIX-like flags (--version/-V, --help/-h) from clp CLI

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** ✅ (Complete)

## Goal

Remove POSIX-style flag syntax (`--version`/`-V`, `--help`/`-h`) from the `clp` CLI so the interface is uniform `key::value` syntax throughout, eliminating the confusing exception for two flags (Motivated: removes inconsistency; Observable: `clp --version` and `clp --help` both produce "unexpected flag" error, version info accessible via `clp .version`, help via `clp .help`; Scoped: `src/lib.rs` and `src/adapter.rs` in `module/claude_profile/`; Testable: `w3 .test level::3` passes, `run_cli(["--version"])` produces error not version string).

Currently, `cli::run()` intercepts `--version`/`-V` before entering the unilang pipeline (line 268 of `lib.rs`), and `adapter::argv_to_unilang_tokens()` intercepts `--help`/`-h` as a special case (lines 97-101 of `adapter.rs`). All other dash-prefixed arguments are already rejected with "unexpected flag". This special treatment creates an inconsistency: `clp --version` works but `clp --quiet` errors. After this change, all arguments starting with `-` are uniformly rejected.

The help error message `"Run '{binary} --help' for usage."` also needs updating to `"Run '{binary} .help' for usage."` to match the new interface.

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/src/lib.rs` § `cli::run()` — remove `--version`/`-V` intercept (lines 267-272), remove `--help` mention in error string (line 282)
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/src/lib.rs` § `print_usage()` — remove `--version, -V` and `--help, -h` from Options block (lines 244-245)
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/src/adapter.rs` § `argv_to_unilang_tokens()` — remove `--help`/`-h` special case (lines 97-101); update doc comment (lines 72-76)
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/tests/cli_adapter_test.rs` — update/add tests covering `--version`, `--help`, `-V`, `-h` now all rejected as unexpected flags

## Out of Scope

- Documentation updates (completed by doc_tsk)
- Email-based account names (Task 110)
- Changes to any other module or crate

## Requirements

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
- Code style: 2-space indentation, custom formatting per `code_style.rulebook.md`; never use `cargo fmt`
- Tests in `tests/` directory of the crate; no inline `#[cfg(test)]` modules

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note `code_style.rulebook.md` formatting constraints and `test_organization.rulebook.md` test placement rules.
2. **Read adapter source** — Read `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/src/adapter.rs` in full to understand current `--help`/`-h` handling (lines 97-101) and the doc comment (lines 72-76).
3. **Read lib source** — Read `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/src/lib.rs` in full to understand `--version`/`-V` handling (lines 267-272), error string (line 282), and Options block in `print_usage()` (lines 244-245).
4. **Read adapter tests** — Read `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/tests/cli_adapter_test.rs` to understand existing test coverage for flag handling.
5. **Write failing tests** — Add tests in `cli_adapter_test.rs` asserting that `["--version"]`, `["-V"]`, `["--help"]`, `["-h"]` each produce an "unexpected flag" error from `argv_to_unilang_tokens()`. These tests must fail before implementation.
6. **Remove `--help`/`-h` from adapter** — Delete the `--help`/`-h` intercept block (lines 97-101 of `adapter.rs`). Update the doc comment to remove mention of `--help`/`-h` as accepted. The subsequent generic "starts with `-`" check (lines 103-108) already handles these uniformly.
7. **Remove `--version`/`-V` from `cli::run()`** — Delete the `--version`/`-V` intercept block (lines 267-272 of `lib.rs`). After deletion, `--version` and `-V` reach the adapter where they fail with "unexpected flag", then `cli::run()` prints the error and exits 1.
8. **Update error message** — In `cli::run()` at the adapter error handler (around line 282), change `"Run '{binary} --help' for usage."` to `"Run '{binary} .help' for usage."`.
9. **Update `print_usage()` Options block** — Remove the two lines showing `--version, -V` and `--help, -h` from the Options section. Keep only the parameter lines (`v::`, `format::`, `dry::`, `name::`).
10. **Validate** — Run `w3 .test level::3` inside Docker (`run/docker .test`). All tests must pass.
11. **Walk Validation Checklist** — check every item. Every answer must be YES.

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|----------------|-------------------|-------------------|
| `["--version"]` | `argv_to_unilang_tokens` | Returns `Err("unexpected flag '--version'...")` |
| `["-V"]` | `argv_to_unilang_tokens` | Returns `Err("unexpected flag '-V'...")` |
| `["--help"]` | `argv_to_unilang_tokens` | Returns `Err("unexpected flag '--help'...")` |
| `["-h"]` | `argv_to_unilang_tokens` | Returns `Err("unexpected flag '-h'...")` |
| `[".account.list", "--verbose"]` | `argv_to_unilang_tokens` | Returns `Err("unexpected flag '--verbose'...")` |
| `[]` (empty) | `argv_to_unilang_tokens` | Returns `Ok((tokens, true))` — help still works via empty args |
| `[".help"]` | `argv_to_unilang_tokens` | Returns `Ok((tokens, true))` — help still works via `.help` |

## Acceptance Criteria

- `argv_to_unilang_tokens(&["--version".to_string()])` returns `Err(...)` containing "unexpected flag"
- `argv_to_unilang_tokens(&["-V".to_string()])` returns `Err(...)` containing "unexpected flag"
- `argv_to_unilang_tokens(&["--help".to_string()])` returns `Err(...)` containing "unexpected flag"
- `argv_to_unilang_tokens(&["-h".to_string()])` returns `Err(...)` containing "unexpected flag"
- `print_usage()` output does not contain `--version` or `--help`
- Error message on adapter failure reads `"Run 'clp .help' for usage."` (not `--help`)
- All existing adapter tests continue to pass (empty args → help, `.` → help, `.help` → help)

## Validation

### Checklist

Desired answer for every question is YES.

**Flag rejection**
- [x] Does `argv_to_unilang_tokens(&["--version".to_string()])` return an error?
- [x] Does `argv_to_unilang_tokens(&["-V".to_string()])` return an error?
- [x] Does `argv_to_unilang_tokens(&["--help".to_string()])` return an error?
- [x] Does `argv_to_unilang_tokens(&["-h".to_string()])` return an error?
- [x] Does the adapter's doc comment no longer mention `--help` or `-h` as accepted inputs?

**Help still works**
- [x] Does empty argv still produce `needs_help=true`?
- [x] Does `.help` arg still produce `needs_help=true`?
- [x] Does `.` arg still produce `needs_help=true`?

**Version info**
- [x] Is `--version`/`-V` handling removed from `cli::run()`?

**Help output**
- [x] Does `print_usage()` no longer print `--version, -V` or `--help, -h`?
- [x] Does the error recovery message say `.help` instead of `--help`?

**Out of Scope confirmation**
- [x] Are files in `claude_profile_core/` unchanged?
- [x] Is the email validation change (Task 110) NOT included in this task?

### Measurements

**M1 — Flag rejection tests pass**
Command: `cargo nextest run --test cli_adapter_test 2>&1 | tail -3`
Before: new tests fail (no rejection for --version/-V/--help/-h). Expected: `test result: ok. X passed`. Deviation: any FAILED line.

### Invariants

- [x] I1 — full test suite: `w3 .test level::3` → 0 failures

### Anti-faking checks

**AF1 — Version intercept removed**
Check: `grep -n -- "--version\|--V" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/src/lib.rs`
Expected: 0 lines from the `cli::run()` intercept (comment in `print_usage()` is acceptable if it appears as a note, but the `if argv.first().is_some_and` block must not exist).
Why: confirms the removal was not a commenting-out but a deletion.

**AF2 — Help intercept removed from adapter**
Check: `grep -n -- "--help\|-h" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/src/adapter.rs`
Expected: 0 lines (the special-case block removed).
Why: confirms adapter no longer treats -h/--help differently from other dash-prefixed args.

## Outcomes

Completed. All source changes applied and all tests pass under `w3 .test level::3` in Docker.

**Source changes applied:**
- `src/lib.rs` — Removed `--version`/`-V` intercept from `cli::run()`; removed `--version, -V` and `--help, -h` lines from `print_usage()` output; updated error recovery message from `--help` to `.help`
- `src/adapter.rs` — Removed `--help`/`-h` special-case branch from `argv_to_unilang_tokens()`; all dash-prefixed args now uniformly rejected with "unexpected flag"

**Validation:** `w3 .test level::3` — all crates ✅; `grep` confirms zero `"--version"`, `"--help"`, `"-V"`, `"-h"` literal strings remain in source
