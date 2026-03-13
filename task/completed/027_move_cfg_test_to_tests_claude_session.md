# Fix code_design Violation — Move #[cfg(test)] Modules to tests/ in claude_session

## Goal

Remove all `#[cfg(test)]` inline test modules from `claude_session/src/` and integrate
their test functions into the existing `tests/` directory, achieving full compliance with
`code_design.rulebook.md` § Test Location Rule. This unblocks L3 rulebook compliance for
the crate and ensures `grep -r '#\[cfg(test)\]' module/claude_session/src/` returns zero
matches. Verified by `w3 .test l::3` passing with no regressions.

## In Scope

- `module/claude_session/src/account.rs` — remove 9-test `#[cfg(test)] mod tests { ... }` block
- `module/claude_session/src/token.rs` — remove 5-test `#[cfg(test)] mod tests { ... }` block
- `module/claude_session/tests/account_tests.rs` — integrate extracted account tests
- `module/claude_session/tests/token_tests.rs` — integrate extracted token tests

## Out of Scope

- Changes to any test logic (tests are moved not rewritten)
- Changes to `detection.rs`, `session.rs`, or `paths.rs`
- Adding new test coverage beyond what was inline
- Any changes to public or private API signatures

## Description

`code_design.rulebook.md` prohibits `#[cfg(test)]` modules inside `src/` files — all test
code must live exclusively in `tests/`. Two source files in `claude_session` violate this:

- `src/account.rs` lines 265–346: 9 unit tests in `#[cfg(test)] mod tests { ... }`
- `src/token.rs` lines 146–195: 5 unit tests in `#[cfg(test)] mod tests { ... }`

The tests are already partially covered by the external test files (`account_tests.rs` has
18 tests, `token_tests.rs` has 7 tests). The inline tests must be checked for duplication
before merging — any already covered tests can be dropped; unique tests must be added.
Private items accessed via `super::*` will require visibility adjustments if needed.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   Specifically: `code_design.rulebook.md` § Test Location — `#[cfg(test)]` in src/ is FORBIDDEN
-   Specifically: `code_style.rulebook.md` — 2-space indent, new-line braces for all new code
-   No mocking; no workarounds; proper fixes only (`codebase_hygiene.rulebook.md`)
-   All tests must remain in `tests/` domain-named files (`test_organization.rulebook.md`)

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; confirm test location and style constraints.
2. **Audit account.rs inline tests** — list all 9 test function names from `src/account.rs`
   `#[cfg(test)]` block; cross-reference with `tests/account_tests.rs` to find unique tests.
3. **Audit token.rs inline tests** — list all 5 test function names from `src/token.rs`
   `#[cfg(test)]` block; cross-reference with `tests/token_tests.rs` to find unique tests.
4. **Move unique account tests** — add unique test functions to `tests/account_tests.rs`;
   ensure correct `use` imports; adjust visibility on any private items accessed if needed.
5. **Remove account.rs #[cfg(test)]** — delete the entire inline block from `src/account.rs`.
6. **Move unique token tests** — add unique test functions to `tests/token_tests.rs`;
   ensure correct `use` imports; adjust visibility on any private items if needed.
7. **Remove token.rs #[cfg(test)]** — delete the entire inline block from `src/token.rs`.
8. **Green state** — `w3 .test l::3` must pass with zero failures and zero warnings.
9. **Walk Validation List** — every answer must be YES. A NO blocks delivery.
10. **Update task status** — set ✅ in `task/readme.md`, recalculate advisability=0,
    move file to `task/completed/`.

## Test Matrix

*(Not applicable — this task moves existing tests, does not write new test logic.)*

## Acceptance Criteria

-   `grep -r '#\[cfg(test)\]' module/claude_session/src/` returns zero matches
-   All unique inline tests from `account.rs` are present in `tests/account_tests.rs`
-   All unique inline tests from `token.rs` are present in `tests/token_tests.rs`
-   `w3 .test l::3` passes with zero failures and zero warnings
-   No test function is silently dropped — count before = count after (duplicates excluded)

## Validation List

Desired answer for every question is YES.

**Removal**
-   [ ] Does `src/account.rs` contain zero `#[cfg(test)]` annotations?
-   [ ] Does `src/token.rs` contain zero `#[cfg(test)]` annotations?
-   [ ] Does `grep -r '#\[cfg(test)\]' module/claude_session/src/` return empty output?

**Integration**
-   [ ] Are all unique account.rs inline tests now present in `tests/account_tests.rs`?
-   [ ] Are all unique token.rs inline tests now present in `tests/token_tests.rs`?
-   [ ] Do all tests compile without `use super::*` tricks that only work inline?

**Test suite health**
-   [ ] Does `w3 .test l::3` pass with zero failures?
-   [ ] Does `w3 .test l::3` pass with zero clippy warnings?
-   [ ] Is the total test count after >= total test count before (no silent drops)?

**Out of scope confirmation**
-   [ ] Are `detection.rs`, `session.rs`, `paths.rs` byte-for-byte unchanged?

## Validation Procedure

### Measurements

**M1 — Test count before vs after**
Run `cargo nextest run --all-features 2>&1 | grep 'tests run'` before starting.
Expected after: count unchanged or higher (never lower — dropped tests are bugs).

**M2 — Zero inline annotations**
`grep -rc '#\[cfg(test)\]' module/claude_session/src/` → expected output: empty or all zeros.

### Anti-faking checks

**AF1 — Tests actually present in destination files**
Count `fn test_` occurrences in `tests/account_tests.rs` before and after.
Count must increase by (unique inline tests in account.rs). Same check for token_tests.rs.

**AF2 — Source files are smaller after removal**
`wc -l module/claude_session/src/account.rs` must decrease by ~82 lines (lines 265–346).
`wc -l module/claude_session/src/token.rs` must decrease by ~50 lines (lines 146–195).
