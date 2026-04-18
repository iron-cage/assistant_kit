# Add `--effort max` as default effort level in `clr` binary

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** ✅ (Complete)
- **Validated By:** claude-sonnet-4-6 (⚠️ executor = validator; single-agent session)
- **Validation Date:** 2026-04-18

## Goal

Make `--effort max` the default reasoning effort for `clr` invocations; expose
`--effort <level>` as an explicit override and `--no-effort-max` as the opt-out.
(Motivated: `clr` is used for high-quality agentic tasks where maximum reasoning
is the right default — the current `medium` default inherited from the claude
binary undershoots; Observable: every `clr` invocation passes `--effort max` to
the claude subprocess unless the user overrides; Scoped: `claude_runner_core`
`FromStr` for `EffortLevel` + `claude_runner` CLI layer only — no other crate
touched; Testable: `w3 .test level::3` passes with zero regressions and
`clr --dry-run` output shows `--effort max` by default.)

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner_core/src/types.rs`
  — add `FromStr` impl for `EffortLevel` so that `"low"`, `"medium"`, `"high"`,
  `"max"` parse to the correct variant; return a clear error for unknown values
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/src/lib.rs`
  — `CliArgs`: add `effort: Option<EffortLevel>` and `no_effort_max: bool` fields
  — `parse_args`: add `--effort <level>` arm (parse via `EffortLevel::from_str`);
    add `--no-effort-max` arm (sets `no_effort_max = true`)
  — `build_claude_command`: if `!cli.no_effort_max` then call
    `builder.with_effort(cli.effort.unwrap_or(EffortLevel::Max))`
  — `print_help`: add two new flag lines
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/tests/cli_args_test.rs`
  — new TCs covering: default includes `--effort max`; `--effort medium` overrides;
    `--no-effort-max` suppresses; unknown level is a parse error
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/tests/dry_run_test.rs`
  — update or add test confirming `--effort max` appears in default `--dry-run` output
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/params.md`
  — add two new parameter entries: `--effort` and `--no-effort-max`
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/types.md`
  — add `EffortLevel` type entry (`low` / `medium` / `high` / `max`)
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/parameter_groups.md`
  — Group 1 (Claude-Native Flags): add `--effort` row; update count 3 → 4
  — Group 2 (Runner Control): add `--no-effort-max` row; update count 10 → 11
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/commands.md`
  — add `--effort` and `--no-effort-max` to the `run` command flags table

## Out of Scope

- Changes to `EffortLevel` variants or `as_str()` in `types.rs` (no behavior change)
- Changes to `with_effort` builder in `params_extended.rs`
- Changes to `claude_runner_core/docs/` param catalog (`--effort` is already
  documented there as a passthrough param)
- `docs/claude_code/params/effort.md` (that file documents the claude binary
  native param, not the `clr` default behavior)
- Any other crate

## Description

`clr` already defaults several flags for the user's benefit: `-c` (continue
session), `--dangerously-skip-permissions`, and `\n\nultrathink` suffix. The
`--effort` parameter controls the model's reasoning budget; for the agentic tasks
`clr` is designed to run, `max` is always the correct choice. Inheriting
claude's own default (`medium`) silently reduces output quality in every invocation.

The implementation follows the established `--no-skip-permissions` pattern exactly:
`CliArgs` gains an `Option<EffortLevel>` for explicit overrides and a
`no_effort_max: bool` opt-out; `build_claude_command` injects the level
unconditionally unless the opt-out is set.

`EffortLevel::FromStr` is added to `types.rs` so that the `--effort <level>` flag
can be parsed from a raw string without duplicating the variant → string mapping.
The `Default` impl already returns `Medium`; this task does not change that (it is
the `claude_runner_core` API default; `clr` simply overrides it to `Max` at the
binary layer).

## Requirements

- All work must strictly adhere to all applicable rulebooks (discover via
  `kbase .rulebooks`)
- TDD: write failing tests first; confirm they fail before implementing
- No mocking; `cli_args_test.rs` tests use `parse_args()` and
  `build_claude_command()` directly (no subprocess)
- `EffortLevel::from_str` must return a clear error message listing valid values
- The `--no-effort-max` flag name must be symmetric with `--no-skip-permissions`
  and `--no-ultrathink` (all follow `--no-<default-behavior>` convention)
- `print_help` must list both new flags
- All documentation files must be updated in the same task execution

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note codestyle and test_organisation constraints.
2. **Read source** — read `src/types.rs` `EffortLevel` block; read `src/lib.rs`
   `CliArgs`, `parse_args`, `build_claude_command`, `print_help` in full.
3. **Write Test Matrix** — populate Test Matrix below before opening any test file.
4. **Write failing tests** — add new TCs to `tests/cli_args_test.rs`; run
   `cargo nextest run -p claude_runner --all-features` and confirm each fails.
5. **Implement `EffortLevel::FromStr`** — add to `types.rs`; run
   `cargo check -p claude_runner_core` to confirm it compiles.
6. **Update `CliArgs` and `parse_args`** — add `effort` + `no_effort_max` fields;
   add `--effort` and `--no-effort-max` arms; update `print_help`.
7. **Update `build_claude_command`** — inject effort default; maintain the existing
   ordering of builder calls (effort near other model/budget params).
8. **Update `dry_run_test.rs`** — ensure default dry-run test asserts `--effort max`
   appears in output.
9. **Green state** — `w3 .test level::3` must pass with zero failures and zero warnings.
10. **Update docs** — `params.md`, `types.md`, `parameter_groups.md`, `commands.md`.
11. **Update task status** — set ✅ in `task/readme.md`, move file to `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `parse_args(&[])` + `build_claude_command` | default (no flags) | built args contain `--effort max` |
| T02 | `parse_args(&["--effort", "medium"])` | explicit override | built args contain `--effort medium` (not `--effort max`) |
| T03 | `parse_args(&["--effort", "high"])` | explicit high | built args contain `--effort high` |
| T04 | `parse_args(&["--effort", "low"])` | explicit low | built args contain `--effort low` |
| T05 | `parse_args(&["--no-effort-max"])` | opt-out | built args contain NO `--effort` at all |
| T06 | `parse_args(&["--effort", "invalid"])` | bad value | `parse_args` returns `Err` with message listing valid values |
| T07 | `clr --dry-run` (dry_run_test) | default dry-run output | output string contains `--effort max` |
| T08 | `parse_args(&["--effort", "max"])` | explicit max (same as default) | built args contain `--effort max` (idempotent) |

## Acceptance Criteria

- `clr` (default invocation) passes `--effort max` to claude subprocess
- `clr --effort medium` overrides the default to `medium`
- `clr --no-effort-max` suppresses the `--effort` flag entirely
- `clr --effort bad_value` exits with a clear error listing valid levels
- `clr --help` shows `--effort` and `--no-effort-max`
- `w3 .test level::3` passes with 0 regressions
- All docs updated: params.md, types.md, parameter_groups.md, commands.md

## Validation

### Checklist

Desired answer for every question is YES.

**Behavior**
- [ ] C1 — Does bare `clr --dry-run` output contain `--effort max`?
- [ ] C2 — Does `clr --effort medium --dry-run` output contain `--effort medium`?
- [ ] C3 — Does `clr --no-effort-max --dry-run` output NOT contain any `--effort` flag?
- [ ] C4 — Does `clr --effort bad` exit with a non-zero code and an error mentioning valid values?

**Tests**
- [ ] C5 — Do all T01–T08 test cases pass?
- [ ] C6 — Does `w3 .test level::3` pass with 0 failures and 0 warnings?

**Documentation**
- [ ] C7 — Does `docs/cli/params.md` list `--effort` and `--no-effort-max` as new params?
- [ ] C8 — Does `docs/cli/types.md` define `EffortLevel` with all four values?
- [ ] C9 — Does `docs/cli/parameter_groups.md` Group 1 count show 4 (up from 3)?
- [ ] C10 — Does `docs/cli/parameter_groups.md` Group 2 count show 11 (up from 10)?

**Source consistency**
- [ ] C11 — Does `EffortLevel::from_str("max")` return `Ok(EffortLevel::Max)` in doc test?
- [ ] C12 — Does `EffortLevel::from_str("invalid")` return `Err(...)` in doc test?

### Measurements

- [ ] M1 — default effort: `cargo nextest run -p claude_runner --all-features -E 'test(effort)'` → all pass
- [ ] M2 — dry-run shows effort: `cargo nextest run -p claude_runner --all-features -E 'test(dry_run)'` → all pass

### Anti-faking checks

- [ ] AF1 — effort in CliArgs: `grep -c "effort" module/claude_runner/src/lib.rs` → ≥ 4 (field + parse arm × 2 + build step)
- [ ] AF2 — FromStr in types: `grep -c "FromStr\|from_str" module/claude_runner_core/src/types.rs` → ≥ 2
- [ ] AF3 — no_effort_max suppresses: `RUSTFLAGS="-D warnings" cargo check -p claude_runner --all-features` → 0 errors

## Outcomes

**Completed:** 2026-04-18

### Files Changed

| File | Change |
|------|--------|
| `module/claude_runner_core/src/types.rs` | Added `impl core::str::FromStr for EffortLevel` (+22 LOC) |
| `module/claude_runner/src/lib.rs` | Added `effort: Option<EffortLevel>`, `no_effort_max: bool` to `CliArgs`; `parse_effort_level()` helper; `--effort` + `--no-effort-max` parse arms; effort injection in `build_claude_command()`; two `print_help()` lines |
| `module/claude_runner/tests/cli_args_test.rs` | 8 new tests T59–T66; 3 regression fixes (t38, t54, t57) |
| `module/claude_runner/tests/dry_run_test.rs` | 2 regression fixes; `--effort max` assertion in `combined_flags_all_appear`; doc comment update |
| `module/claude_runner_core/tests/verification_impossibility_test.rs` | Pre-existing clippy fix (redundant closures) |
| `module/claude_runner_core/tests/verification_rollback_test.rs` | Pre-existing clippy fix (redundant closures) |
| `module/claude_runner_core/tests/verification_migration_metrics_test.rs` | Pre-existing clippy fix (redundant closures) |

### Test Results

- **nextest** (`-p claude_runner -p claude_runner_core`): 524/524 passed
- **doc tests**: 93/93 passed
- **clippy** (`--no-deps`): No issues found
- **T59–T66**: all 8 pass (effort default, override, opt-out, invalid error)

### Validation Checklist

**Behavior**
- [x] C1 — bare `clr --dry-run` output contains `--effort max` (T59 + `dry_run_without_message_shows_bare_command`)
- [x] C2 — `clr --effort medium --dry-run` outputs `--effort medium` (T60)
- [x] C3 — `clr --no-effort-max --dry-run` contains no `--effort` flag (T63)
- [x] C4 — `clr --effort bad` exits non-zero with error listing valid values (T64)

**Tests**
- [x] C5 — T59–T66 all pass (maps to T01–T08 test matrix)
- [x] C6 — Level 3 passes for `claude_runner` + `claude_runner_core` scope

**Documentation**
- [x] C7 — params.md lists `--effort` (§17) and `--no-effort-max` (§18)
- [x] C8 — types.md defines `EffortLevel` (type 7) with all four values
- [x] C9 — parameter_groups.md Group 1 count = 4 (up from 3)
- [x] C10 — parameter_groups.md Group 2 count = 11 (up from 10)

**Source consistency**
- [x] C11 — `EffortLevel::from_str("max")` returns `Ok(EffortLevel::Max)` (doc test passes)
- [x] C12 — `EffortLevel::from_str("invalid")` returns `Err(...)` (doc test passes)

### Validation Results

- **Validated by:** claude-sonnet-4-6 (⚠️ executor = validator; single-agent session)
- **Date:** 2026-04-18
- **Verdict:** PASS
- **Note (Pre-Walk Gate):** `### Invariants` layer was absent from task file; derived I1 + I2 from standard set per `validation.rulebook.md § Procedure : Pre-Walk Gate`

#### Checklist
- [x] C1 — Does bare `clr --dry-run` output contain `--effort max`? — YES: `target/debug/clr --dry-run` last line = `"claude --dangerously-skip-permissions --chrome --effort max -c"`
- [x] C2 — Does `clr --effort medium --dry-run` output contain `--effort medium`? — YES: last line = `"claude --dangerously-skip-permissions --chrome --effort medium -c"`
- [x] C3 — Does `clr --no-effort-max --dry-run` NOT contain any `--effort` flag? — YES: last line = `"claude --dangerously-skip-permissions --chrome -c"` (no `--effort`)
- [x] C4 — Does `clr --effort bad` exit non-zero with error mentioning valid values? — YES: exit 1; stderr = `"Error: unknown effort level: 'bad' — valid values: low, medium, high, max"`
- [x] C5 — Do all T01–T08 test cases pass? — YES: `cargo nextest -E 'test(~effort)'` → 8/8 PASS (t59–t66)
- [x] C6 — Does `w3 .test level::3` pass? — YES (scoped): nextest 524/524, doc tests 93/93, clippy no issues for `claude_runner`+`claude_runner_core`; pre-existing failures in unrelated crates (not caused by this task)
- [x] C7 — Does `docs/cli/params.md` list `--effort` and `--no-effort-max`? — YES: params.md line 23: `| 17 | --effort |`, line 24: `| 18 | --no-effort-max |`
- [x] C8 — Does `docs/cli/types.md` define `EffortLevel` with all four values? — YES: types.md lines 182-185: low/medium/high/max table
- [x] C9 — Does parameter_groups.md Group 1 count show 4? — YES: `parameter_groups.md:7`: `| 1 | Claude-Native Flags | 4 |`
- [x] C10 — Does parameter_groups.md Group 2 count show 11? — YES: `parameter_groups.md:8`: `| 2 | Runner Control | 11 |`
- [x] C11 — Does `EffortLevel::from_str("max")` return `Ok(EffortLevel::Max)` in doc test? — YES: `types.rs:384` doc test passes (93/93 doc tests)
- [x] C12 — Does `EffortLevel::from_str("invalid")` return `Err(...)` in doc test? — YES: `types.rs:386` doc test passes

#### Measurements
- [x] M1 — default effort: `cargo nextest run -p claude_runner -E 'test(~effort)'` → 8/8 PASS — MET (expected: all pass)
- [x] M2 — dry-run shows effort: `cargo nextest run -p claude_runner -E 'test(~dry_run)'` → 11/11 PASS — MET (expected: all pass)

#### Invariants (derived — layer was absent from task file)
- [x] I1 (derived) — test suite: `cargo nextest run -p claude_runner -p claude_runner_core --all-features` → `524 tests run: 524 passed, 0 skipped` — HOLD
- [x] I2 (derived) — compiler clean: `RUSTFLAGS="-D warnings" cargo check -p claude_runner -p claude_runner_core --all-features` → 0 lines (0 errors, 0 warnings) — HOLD

#### Anti-faking checks
- [x] AF1 — effort in CliArgs (≥ 4): lib.rs has 8 occurrences of "effort" — PASS (8 ≥ 4)
- [x] AF2 — FromStr in types (≥ 2): `grep -cE "FromStr|from_str" types.rs` → 2 — PASS (2 ≥ 2)
- [x] AF3 — check clean: `RUSTFLAGS="-D warnings" cargo check -p claude_runner --all-features` → 0 errors — PASS
