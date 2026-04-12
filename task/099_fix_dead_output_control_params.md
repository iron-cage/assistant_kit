# Fix dead output-control params in version_install, version_guard, and processes_kill

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** 🎯 (Available)

## Goal

Fix three mutation commands that register output-control parameters but never apply them — `version_install_routine` ignores `format::`, `version_guard_routine` ignores `v::`, and `.processes.kill` lacks both `verbosity` and `format` entirely — so that all three commands honour the standard output-control contract, verified by `w3 .test level::3`. (Motivated: users passing `format::json` to `.version.install` or `v::0` to `.version.guard` silently receive wrong output; Observable: all three routines read and apply their output-control params; Scoped: only output-control param plumbing in `commands.rs` and registration in `lib.rs`; Testable: `cargo nextest run --test integration --features enabled 2>&1 | grep -E 'PASS|FAIL'`)

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_manager/src/commands.rs` — `version_install_routine`: call `OutputOptions::from_cmd()` and apply `opts.format` / `opts.verbosity` to output; `version_guard_routine`: apply `opts.verbosity` from `v()` param; `processes_kill_routine`: extract and apply `verbosity` and `format` params
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_manager/src/lib.rs` — `register_commands()`: add `v()` and `fmt()` to `.processes.kill` registration
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_manager/unilang.commands.yaml` — `.processes.kill`: add `verbosity` and `format` argument entries
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_manager/tests/integration/mutation_commands_test.rs` — add TC-360x: `format::json` on `.version.install dry::1` produces valid JSON; add TC-401x: `v::0` on `.version.guard dry::1` produces shorter output; add TC-310x: `format::json` on `.processes.kill dry::1` produces JSON array
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_manager/tests/integration/cross_cutting_test.rs` — add TC-261: `format::json` on `.version.install` at adapter level accepted; TC-262: `v::0` on `.version.guard` accepted

## Out of Scope

- Adding `format::` to `.version.guard` (no JSON output defined for guard; separate design decision)
- Changing the content or structure of guard/install text output (only format routing)
- Any changes to `version_show_routine`, `version_list_routine`, or settings commands

## Description

Three mutation commands register output-control parameters but silently ignore them at runtime. `version_install_routine` registers `format::` but never calls `OutputOptions::from_cmd()`, so `format::json` produces text output regardless. `version_guard_routine` registers `v::` but passes a hardcoded verbosity to `guard_once` and its helpers, so `v::0` has no visible effect. `.processes.kill` is missing both `verbosity` and `format` registrations entirely — any attempt to pass them exits 1 with "unknown parameter".

The expected contract is that every command honours `format::json` and `v::N` consistently. The infrastructure (`OutputOptions`, `json_escape`, the `v()` and `fmt()` registration helpers) already exists and is used correctly in the read commands. The gap is purely in the three mutation commands' wiring.

The fix is mechanical: plumb `OutputOptions::from_cmd(&cmd)?` into `version_install_routine` and `version_guard_routine`, thread verbosity through the guard helper chain, and add the missing `v()` and `fmt()` entries to the `.processes.kill` registration in `lib.rs` and `unilang.commands.yaml`.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   TDD: write failing tests before implementing; confirm they fail before fixing
-   No mocking; all tests must use the real binary via `run_clm` / `run_clm_with_env`

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note code_style constraints on 2-space indent, 50-line function limit, and test organisation rules.
2. **Write Test Matrix** — populate all rows below before opening any test file. The matrix is the contract; tests implement it.
3. **Write failing tests** — add TC-360x, TC-401x, TC-310x, TC-261, TC-262 to the appropriate integration test files. Run `cargo nextest run --features enabled` and confirm each new test fails.
4. **Implement — version_install_routine format** — read `src/commands.rs` `version_install_routine`; call `OutputOptions::from_cmd(&cmd)?`; route dry-run text output through `opts.format` (JSON wraps the dry-run message in a JSON string; text emits as-is).
5. **Implement — version_guard_routine verbosity** — read `version_guard_routine`, `guard_once`, `guard_once_latest`, `guard_once_pinned`; thread verbosity from `OutputOptions` through the three helpers so `v::0` emits bare values and `v::1` emits labeled output.
6. **Implement — processes_kill params** — add `v()` and `fmt()` to the `.processes.kill` registration in `lib.rs` and the corresponding yaml entries; read `processes_kill_routine` and apply `OutputOptions::from_cmd()` for both format and verbosity.
7. **Green state** — `w3 .test level::3` must pass with zero failures and zero warnings before proceeding.
8. **Refactor if needed** — ensure no function exceeds 50 lines; no duplication.
9. **Submit for Validation** — trigger SUBMIT transition; validator walks Validation section.
10. **Update task status** — on validation pass set ✅ in `task/readme.md`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `.version.install dry::1 format::json` | version_install_routine format routing | stdout starts with `{` or `[`, exit 0 |
| T02 | `.version.install version::stable dry::1 format::json` | JSON format with explicit version | stdout is valid JSON object, exit 0 |
| T03 | `.version.guard dry::1 v::0` | version_guard_routine verbosity 0 | output shorter than `v::1` for same command |
| T04 | `.version.guard dry::1 v::1` | version_guard_routine verbosity 1 | output longer than `v::0` (labeled fields) |
| T05 | `.processes.kill dry::1 format::json` | processes_kill format routing | stdout starts with `{` or `[`, or `"no active processes"` as JSON string, exit 0 |
| T06 | `.processes.kill v::0` | processes_kill verbosity routing | accepted without exit 1 |
| T07 | `.processes.kill format::JSON` | format case validation | exit 1 |

## Acceptance Criteria

-   `version_install_routine` calls `OutputOptions::from_cmd()` and branches on `opts.format`
-   `version_guard_routine` calls `OutputOptions::from_cmd()` and passes `opts.verbosity` to `guard_once`, `guard_once_latest`, `guard_once_pinned`
-   `.processes.kill` has `verbosity` and `format` registered in `lib.rs` and `unilang.commands.yaml`
-   `processes_kill_routine` calls `OutputOptions::from_cmd()` and applies both params
-   T01–T07 all pass
-   `w3 .test level::3` passes with zero failures

## Validation

### Checklist

Desired answer for every question is YES.

**version_install_routine**
- [ ] C1 — Does `version_install_routine` call `OutputOptions::from_cmd()`?
- [ ] C2 — Does `.version.install format::json dry::1` produce stdout starting with `{` or `[`?
- [ ] C3 — Does `.version.install format::JSON` exit 1?

**version_guard_routine**
- [ ] C4 — Does `version_guard_routine` call `OutputOptions::from_cmd()`?
- [ ] C5 — Does `.version.guard v::0 dry::1` produce fewer output lines than `v::1`?
- [ ] C6 — Do `guard_once`, `guard_once_latest`, `guard_once_pinned` accept a verbosity parameter?

**processes_kill registration**
- [ ] C7 — Is `verbosity` registered for `.processes.kill` in `lib.rs` (`v()` in the `reg_cmd` call)?
- [ ] C8 — Is `format` registered for `.processes.kill` in `lib.rs` (`fmt()` in the `reg_cmd` call)?
- [ ] C9 — Does `unilang.commands.yaml` `.processes.kill` have `verbosity` and `format` arguments?
- [ ] C10 — Does `.processes.kill v::0` exit 0?
- [ ] C11 — Does `.processes.kill format::json dry::1` produce JSON output?

**Out of Scope confirmation**
- [ ] C12 — Does `.version.guard` still lack a `format` argument in `lib.rs` and yaml (not added here)?
- [ ] C13 — Are `version_show_routine`, `version_list_routine`, settings commands unchanged?

### Measurements

- [ ] M1 — install format: `cm .version.install dry::1 format::json | head -1` → line starting with `{` or `[` (was: text line starting with `[dry-run]`)
- [ ] M2 — guard verbosity: `cm .version.guard dry::1 v::0 | wc -l` < `cm .version.guard dry::1 v::1 | wc -l` (was: identical regardless of v)
- [ ] M3 — kill params: `cm .processes.kill v::0; echo $?` → exit 0 (was: exit 1 "unknown parameter")

### Invariants

- [ ] I1 — test suite: `w3 .test level::3` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check --features enabled` → 0 warnings

### Anti-faking checks

- [ ] AF1 — install format applied: `grep -n "OutputOptions::from_cmd" src/commands.rs | grep -i install` → at least one match (confirms call is present, not just format param registered)
- [ ] AF2 — guard verbosity applied: `grep -n "opts\.verbosity\|verbosity" src/commands.rs | grep -i guard` → at least one match in guard routine body
- [ ] AF3 — kill params registered: `grep -n "processes.kill" src/lib.rs` output line contains `v()` and `fmt()`

## Outcomes

[Added upon task completion.]
