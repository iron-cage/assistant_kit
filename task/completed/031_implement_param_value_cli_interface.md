# Implement param::value as Primary CLI Interface (L5 Blocker)

> **Superseded (2026-03-21):** This task's outcome was reversed. The CLI was redesigned
> back to `--flag value` syntax to mirror Claude Code's native interface. The unilang
> dependency was removed. See `-current.plan.md` in `src/-default_topic/` for details.

## Goal

Change `claude_runner`'s CLI to accept `param::value` as the primary — and only —
parameter format, replacing the current `--param value` / `--flag` style mandated by
`cli.rulebook.md` § Governing Principle. Update all documentation in `docs/cli/` to
reflect the new interface. After this task, `claude_runner .run message::\"hello\"` works
and `claude_runner .run --message \"hello\"` does not. Verified by `w3 .test l::3` passing
with updated interface tests.

## In Scope

- `module/claude_runner/src/` — parser changes to accept `param::value` format
- All 9 parameters switched to `param::value` syntax: `message`, `model`, `continue`,
  `dry_run`, `verbose`, `output_file`, `session_id`, `max_turns`, `token_limit`
- Boolean parameters use `flag::true` / `flag::false` (not bare `--flag`)
- `module/claude_runner/docs/cli/commands.md` — update all examples to `param::value`
- `module/claude_runner/docs/cli/params.md` — update Aliases column; remove `--param` refs
- `module/claude_runner/docs/cli/workflows.md` — update all 8 workflow examples
- `module/claude_runner/docs/cli/dictionary.md` — update all 9 term definitions
- `module/claude_runner/tests/` — update all test cases using old `--param` format
- `module/claude_runner/docs/cli/readme.md` — update L5 completion matrix row

## Out of Scope

- Adding `verbosity::` parameter (covered in task 032)
- Creating `parameter_interactions.md` (covered in task 029)
- Types.md formatting (covered in task 030)
- Maintaining backward compatibility with `--param` format (explicitly excluded by
  `cli.rulebook.md` — the old format must not be preserved)

## Description

`cli.rulebook.md` § Governing Principle mandates `param::value` as the ONLY format —
`--param` and `-p` are FORBIDDEN. The current implementation uses `--param value` /
`--flag` throughout, which is a critical violation.

The `docs/cli/design/evolution.md` documents a planned transition: v0.1.0 → v0.2.0 dual
support → v1.0.0 explicit. This plan defers compliance but the rulebook is authoritative
now. The dual-support transition state is NOT a valid compliance position.

Boolean flags require special attention: `--dry-run` becomes `dry_run::true`, and absence
of the parameter must be treated as `dry_run::false`. The parser must handle `param::value`
as the only accepted syntax for ALL parameters.

`docs/cli/design/evolution.md` may be updated to reflect that the migration was completed,
or left as historical context. The unilang exploration design docs in `docs/cli/design/`
can remain as architectural history — they don't affect interface compliance.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   `cli.rulebook.md` — `param::value` ONLY; `--param` FORBIDDEN in interface and docs
-   `code_style.rulebook.md` — 2-space indent, new-line braces; `cargo fmt` FORBIDDEN
-   `code_design.rulebook.md` — all tests in `tests/`; no `#[cfg(test)]` in `src/`
-   `codebase_hygiene.rulebook.md` — no mocking; no backward compat shims; delete old code
-   `test_organization.rulebook.md` — TDD: failing tests FIRST, then implementation

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; confirm `param::value` format requirements and
   all applicable constraints.
2. **Write Test Matrix** — define test cases for the new parser: valid `param::value` input,
   invalid `--param` input (must error), edge cases (boolean values, missing params).
3. **Write failing tests** — implement tests from the Test Matrix. Confirm they fail against
   current `--param` implementation.
4. **Implement parser** — change the CLI argument parser to accept only `param::value` syntax.
   Remove all `--param` / `-p` / `--flag` handling. Boolean params: `flag::true` / `flag::false`.
5. **Green state** — `w3 .test l::3` must pass with zero failures and zero warnings.
6. **Update docs/cli/commands.md** — replace every `--param` example with `param::value`.
7. **Update docs/cli/params.md** — update Aliases column; remove all `--param` notation.
8. **Update docs/cli/workflows.md** — replace all 8 workflow examples.
9. **Update docs/cli/dictionary.md** — rewrite 9 term definitions to use `param::value`.
10. **Update docs/cli/readme.md** — mark L5 interface requirement as ✅.
11. **Walk Validation List** — every answer must be YES. A NO blocks delivery.
12. **Update task status** — set ✅, recalculate advisability=0, move to `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|----------------|-------------------|-------------------|
| T01 | `message::"hello"` | standard param | Accepted; message = "hello" |
| T02 | `--message "hello"` | old-style param | Rejected with parse error |
| T03 | `-m "hello"` | short-form param | Rejected with parse error |
| T04 | `dry_run::true` | boolean param true | Accepted; dry_run = true |
| T05 | `dry_run::false` | boolean param false | Accepted; dry_run = false |
| T06 | `--dry-run` | old-style flag | Rejected with parse error |
| T07 | `model::claude-opus-4-6` | model param | Accepted; model set |
| T08 | `token_limit::1000` | numeric param | Accepted; limit = 1000 |
| T09 | `continue::true` | boolean param | Accepted; continue = true |
| T10 | `message::"hello" model::claude-sonnet-4-6` | multiple params | Both accepted |
| T11 | no parameters | empty input | Uses defaults; no error |
| T12 | `unknown_param::value` | unknown parameter | Rejected with clear error |

## Acceptance Criteria

-   `claude_runner .run message::"hello"` is accepted by the parser
-   `claude_runner .run --message "hello"` is rejected with a clear error message
-   All 12 Test Matrix rows have passing tests
-   Zero occurrences of `--param` / `--flag` syntax in `docs/cli/commands.md`,
    `docs/cli/params.md`, `docs/cli/workflows.md`, `docs/cli/dictionary.md`
-   `w3 .test l::3` passes with zero failures and zero warnings

## Validation List

Desired answer for every question is YES.

**Implementation**
-   [ ] Does the parser accept `param::value` syntax for all 9 parameters?
-   [ ] Does the parser reject `--param value` syntax with a clear error?
-   [ ] Does the parser reject `--flag` syntax with a clear error?
-   [ ] Do boolean params require explicit `::true` / `::false` values?

**Tests (T01–T12)**
-   [ ] Does T01 (`message::`) pass?
-   [ ] Does T02 (`--message`) fail with a parse error?
-   [ ] Does T04 (`dry_run::true`) pass?
-   [ ] Does T06 (`--dry-run`) fail with a parse error?
-   [ ] Do all 12 Test Matrix rows have passing tests?

**Documentation**
-   [ ] Does `docs/cli/commands.md` contain zero `--param` or `--flag` occurrences?
-   [ ] Does `docs/cli/params.md` contain zero `--param` or `--flag` occurrences?
-   [ ] Does `docs/cli/workflows.md` contain zero `--param` or `--flag` occurrences?
-   [ ] Does `docs/cli/dictionary.md` contain zero `--param` or `--flag` occurrences?
-   [ ] Does `docs/cli/readme.md` show L5 interface requirement as ✅?

**No backward compat shim**
-   [ ] Is there zero code that accepts `--param` as an alias or fallback?

## Validation Procedure

### Measurements

**M1 — Old-format occurrence count in docs**
`grep -r '\-\-[a-z]' module/claude_runner/docs/cli/{commands,params,workflows,dictionary}.md | wc -l`
Expected after: 0.

**M2 — Test count for parser tests**
`grep -c 'param::value\|param_value\|parse.*param' module/claude_runner/tests/ -r`
Expected after: ≥12 (one test per Test Matrix row).

**M3 — All tests pass**
`w3 .test l::3` → expected: 0 failures, 0 warnings.

### Anti-faking checks

**AF1 — Old-format parser code removed**
`grep -r '\-\-message\|\-\-model\|\-\-dry.run\|\-\-continue\|\-\-verbose' module/claude_runner/src/ | wc -l`
Expected: 0. Any match means the old format code was not fully removed.

**AF2 — Rejection tests actually test rejection**
The T02/T06 tests must assert that `--param` input returns `Err(...)`, not `Ok(...)`.
Verify tests contain `assert!(result.is_err())` or equivalent.
