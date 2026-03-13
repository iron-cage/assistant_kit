# Add verbosity:: Parameter to claude_runner (L5 Blocker)

## Goal

Add a `verbosity::` parameter (integer 0–5) to `claude_runner`'s CLI that controls the
runner's own output level, independent of Claude's verbosity. This satisfies
`cli.rulebook.md` § Output Controls mandate for all CLIs to implement the 0–5 verbosity
system: 0=silent, 1=errors only, 2=warnings, 3=default/normal, 4=verbose, 5=debug.
After this task, `claude_runner .run message::"hi" verbosity::0` produces no runner output.
Verified by `w3 .test l::3` passing.

## In Scope

- New `VerbosityLevel` semantic newtype in `module/claude_runner/src/`
- New `verbosity::` parameter wired to `VerbosityLevel` type in CLI parser
- Runner output (progress, status lines, error detail) respects the verbosity level
- `module/claude_runner/docs/cli/params.md` — new `verbosity::` parameter entry
- `module/claude_runner/docs/cli/types.md` — new `VerbosityLevel` type entry (`5.`)
- `module/claude_runner/docs/cli/parameter_groups.md` — add `verbosity::` to
  Behavior Flags group (or appropriate group)
- `module/claude_runner/docs/cli/commands.md` — add `verbosity::` to `.run` parameter table
- `module/claude_runner/tests/` — tests for verbosity level behavior

## Out of Scope

- Changing the existing `verbose::` parameter (which forwards to Claude's verbosity) —
  `verbosity::` is a NEW parameter for runner-side output only
- The transition to `param::value` format (covered in task 031; this task assumes task
  031 is complete or uses `param::value` format from the start)
- Implementing structured logging or log filtering beyond the 0–5 level system

## Description

`cli.rulebook.md` § Output Controls requires ALL CLIs to implement a `verbosity::0-5`
system for the tool's own output. `claude_runner` currently has only a `verbose::` boolean
that forwards to Claude Code's `--verbose` flag — it controls Claude's verbosity, not the
runner's own output.

The existing `verbose::` parameter is NOT affected by this task. The new `verbosity::`
parameter is distinct: it controls what `claude_runner` itself prints to stdout/stderr
(startup messages, progress indicators, session detection results, error detail level,
debug information), independently of what Claude Code outputs.

Level semantics:
- `verbosity::0` — silent; runner emits nothing to stdout/stderr
- `verbosity::1` — errors only; runner only prints fatal errors
- `verbosity::2` — warnings; runner prints errors and warnings
- `verbosity::3` — normal (default); runner prints progress and status
- `verbosity::4` — verbose; runner prints detailed step-by-step progress
- `verbosity::5` — debug; runner prints internal state, timing, paths

`VerbosityLevel` is a semantic newtype wrapping `u8` with range validation (0–5).
It is documented as `Type :: 5.` in `types.md` (added after tasks 029/030 add Types 1–4).

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   `cli.rulebook.md` — verbosity 0–5 required; level 3 is default
-   `code_style.rulebook.md` — 2-space indent, new-line braces; `cargo fmt` FORBIDDEN
-   `code_design.rulebook.md` — all tests in `tests/`; no `#[cfg(test)]` in `src/`
-   `crate_distribution.rulebook.md` — `VerbosityLevel` type should be exported cleanly
-   TDD: failing tests first, then implementation (`test_organization.rulebook.md`)
-   `verbosity::` must NOT interfere with `verbose::` (two independent parameters)

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; confirm verbosity requirements, type conventions.
2. **Write Test Matrix** — define test cases: valid levels 0–5, invalid levels (6, -1, "x"),
   default value (3), output suppression at level 0, output presence at level 3+.
3. **Write failing tests** — implement tests from Test Matrix. Tests must fail on current
   codebase (no `verbosity::` parameter yet).
4. **Implement VerbosityLevel type** — newtype wrapping `u8`, validation (0–5),
   `FromStr`, `Default` (→ 3), `Display`. Located in `src/types.rs` or appropriate module.
5. **Wire verbosity:: parameter** — add to CLI parser; default to `VerbosityLevel::default()`
   (level 3); propagate to all runner output calls.
6. **Implement output gating** — wrap all runner stdout/stderr output in verbosity checks;
   level 0 silences all runner output; level 3 is current behavior.
7. **Green state** — `w3 .test l::3` must pass with zero failures and zero warnings.
8. **Update docs/cli/params.md** — add `verbosity::` parameter entry with all required fields.
9. **Update docs/cli/types.md** — add `### Type :: 5. \`VerbosityLevel\`` entry with
   Description, Constraints, Methods, and Example.
10. **Update docs/cli/parameter_groups.md** — add `verbosity::` to appropriate group;
    update any affected "Why NOT" sections.
11. **Update docs/cli/commands.md** — add `verbosity::` to `.run` parameter table.
12. **Walk Validation List** — every answer must be YES.
13. **Update task status** — set ✅, recalculate advisability=0, move to `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|----------------|-------------------|-------------------|
| T01 | `verbosity::0` | silence level | Runner emits no output to stdout/stderr |
| T02 | `verbosity::3` | normal level | Runner emits standard progress output |
| T03 | `verbosity::5` | debug level | Runner emits debug-level detail |
| T04 | `verbosity::6` | invalid level | Rejected: `VerbosityLevel` range error |
| T05 | no `verbosity::` | default | Behaves as `verbosity::3` |
| T06 | `verbosity::1` | errors only | Only error messages appear; no progress |
| T07 | `verbosity::0 verbose::true` | independent params | verbosity::0 silences runner; verbose::true still forwards to Claude |
| T08 | `VerbosityLevel::from_str("0")` | type parsing | Returns `Ok(VerbosityLevel(0))` |
| T09 | `VerbosityLevel::from_str("6")` | invalid parse | Returns `Err(...)` |
| T10 | `VerbosityLevel::default()` | default value | Returns `VerbosityLevel(3)` |

## Acceptance Criteria

-   `VerbosityLevel` type exists with `FromStr`, `Default` (→ 3), and range validation
-   `verbosity::` parameter is accepted by the CLI parser
-   All 10 Test Matrix rows have passing tests
-   Level 0 suppresses all runner output (T01)
-   Level 3 is the default when parameter is absent (T05)
-   `verbosity::` and `verbose::` are independent — T07 passes
-   All 4 documentation files updated (`params.md`, `types.md`, `parameter_groups.md`,
    `commands.md`)
-   `w3 .test l::3` passes with zero failures and zero warnings

## Validation List

Desired answer for every question is YES.

**Implementation**
-   [ ] Does `VerbosityLevel` type exist and compile?
-   [ ] Does `VerbosityLevel::default()` return level 3?
-   [ ] Does `VerbosityLevel::from_str("6")` return `Err`?
-   [ ] Does `VerbosityLevel::from_str("0")` return `Ok(VerbosityLevel(0))`?
-   [ ] Does `verbosity::0` in CLI suppress all runner output?
-   [ ] Is `verbosity::` independent from `verbose::` (T07 passes)?

**Tests (T01–T10)**
-   [ ] Does T01 (silence) pass?
-   [ ] Does T04 (invalid level 6) return an error?
-   [ ] Does T05 (default = 3) pass?
-   [ ] Do all 10 Test Matrix rows have passing tests?

**Documentation**
-   [ ] Does `docs/cli/params.md` have a `verbosity::` parameter entry?
-   [ ] Does `docs/cli/types.md` have `### Type :: 5. \`VerbosityLevel\`` entry?
-   [ ] Does `docs/cli/types.md` VerbosityLevel entry have a Methods section?
-   [ ] Does `docs/cli/commands.md` list `verbosity::` in the `.run` parameter table?
-   [ ] Does `docs/cli/parameter_groups.md` include `verbosity::` in a group?

**No test location violation**
-   [ ] Does `src/` contain zero `#[cfg(test)]` annotations for new code?

## Validation Procedure

### Measurements

**M1 — Test count for VerbosityLevel**
`grep -c 'verbosity\|VerbosityLevel' module/claude_runner/tests/ -r`
Expected: ≥10 (one test per Test Matrix row minimum).

**M2 — Type definition present**
`grep -r 'struct VerbosityLevel\|type VerbosityLevel' module/claude_runner/src/`
Expected: exactly 1 match.

**M3 — All tests pass**
`w3 .test l::3` → expected: 0 failures, 0 warnings.

### Anti-faking checks

**AF1 — Silence test actually tests silence**
T01 must capture stdout/stderr and assert it is empty when `verbosity::0` is set.
The test must NOT just check that the function returns Ok — it must verify output content.

**AF2 — Default value is enforced at runtime**
T05 test must run the CLI WITHOUT any `verbosity::` parameter and verify behavior
matches `verbosity::3`. Cannot just test `VerbosityLevel::default()` in isolation.
