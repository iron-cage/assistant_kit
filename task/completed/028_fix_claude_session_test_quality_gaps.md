# Fix claude_session Test Quality Gaps (Codestyle)

## Note

**Scope reduced 2026-03-21.** Issues B and C are no longer applicable:
- Issue B (`detection_tests.rs` duplication): File deleted — detection moved to
  `claude_storage_core/tests/continuation_tests.rs` (task-033).
- Issue C (`session_tests.rs` bug_reproducer): File deleted — `SessionManager`
  moved to `claude_runner_core/tests/session_dir_tests.rs` (task-034).

Only Issue A remains.

## Goal

Reformat `responsibility_no_process_execution_test.rs` to the custom 2-space / new-line
braces codestyle. Verified by `w3 .test l::3` passing with zero regressions.

## In Scope

- `module/claude_session/tests/responsibility_no_process_execution_test.rs` — reformat
  from rustfmt style (same-line `{`) to custom codestyle (new-line `{`, 2-space indent)

## Out of Scope

- Changes to any test assertion logic (only style and structure, not test semantics)
- Adding new test cases or coverage
- Changes to source files in `src/`
- Addressing task 027 violations (that is covered separately)

## Description

Three independent quality issues were found during the `/rulebook_cli` audit pass on
`claude_session/tests/`:

**Issue A — Wrong codestyle** (`responsibility_no_process_execution_test.rs`):
The entire file uses standard rustfmt style (4-space indent, same-line opening braces).
`code_style.rulebook.md` mandates 2-space indent and opening `{` on a new line. `cargo fmt`
is FORBIDDEN. The file must be manually reformatted.

**Issue B — Path escaping duplication** (`detection_tests.rs`):
The path escaping logic (replacing `/_.@#%& ` chars with hyphens) is copy-pasted
into 5 test functions instead of calling `get_claude_storage_path()` from the library.
This violates the Anti-Duplication Principle and means escaping tests are fragile to
logic drift. The fix replaces all 5 inline copies with direct calls to the actual function.

**Issue C — Missing bug_reproducer marker** (`session_tests.rs`):
`modern_detection_supersedes_deprecated_method` documents a known regression (deprecated
`session_exists()` superseded by new detection) but lacks the `test_kind: bug_reproducer`
doc attribute that `test_organization.rulebook.md` requires for all regression tests.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   `code_style.rulebook.md` — 2-space indent, new-line `{` in all Rust code including tests;
    `cargo fmt` FORBIDDEN
-   `code_design.rulebook.md` — no duplication of logic across test functions
-   `test_organization.rulebook.md` — `bug_reproducer(issue-NNN)` marker on regression tests
-   No mocking; no logic changes to existing tests; same assertions before and after

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; confirm codestyle and test organization rules.
2. **Reformat responsibility_no_process_execution_test.rs** — manually convert to 2-space
   indent and new-line opening braces throughout. No logic changes.
3. **Fix detection_tests.rs duplication** — identify the 5 test functions containing
   inline path escaping logic; refactor each to call `get_claude_storage_path()` instead.
   Ensure the import for `get_claude_storage_path` is present.
4. **Add bug_reproducer marker** — in `session_tests.rs`, add `/// test_kind: bug_reproducer`
   doc comment (or the correct marker format per `test_organization.rulebook.md`) immediately
   above `modern_detection_supersedes_deprecated_method`.
5. **Green state** — `w3 .test l::3` must pass with zero failures and zero warnings.
6. **Walk Validation List** — every answer must be YES.
7. **Update task status** — set ✅, recalculate advisability=0, move to `task/completed/`.

## Test Matrix

*(Not applicable — no new test logic is being written; this task fixes quality issues.)*

## Acceptance Criteria

-   `responsibility_no_process_execution_test.rs` uses 2-space indent throughout with no
    same-line opening braces
-   `detection_tests.rs` contains zero copies of inline path escaping logic; all 5 test
    functions delegate to `get_claude_storage_path()`
-   `session_tests.rs::modern_detection_supersedes_deprecated_method` has
    `bug_reproducer` marker
-   `w3 .test l::3` passes with zero failures and zero warnings

## Validation List

Desired answer for every question is YES.

**Codestyle (responsibility_no_process_execution_test.rs)**
-   [ ] Does the file use 2-space indentation throughout (zero 4-space indents)?
-   [ ] Do all opening `{` braces appear on a new line (zero same-line braces)?
-   [ ] Is the file free of any rustfmt-style formatting patterns?

**Duplication removal (detection_tests.rs)**
-   [ ] Does `grep -n 'replace\|escape\|hyphen' detection_tests.rs` show zero inline
    escaping logic?
-   [ ] Do all 5 affected test functions call `get_claude_storage_path()` instead?
-   [ ] Is `use claude_session::detection::get_claude_storage_path` (or equivalent) imported?

**Marker (session_tests.rs)**
-   [ ] Does `modern_detection_supersedes_deprecated_method` have a `bug_reproducer` marker?

**Test suite health**
-   [ ] Does `w3 .test l::3` pass with zero failures?
-   [ ] Does `w3 .test l::3` pass with zero clippy warnings?

**Out of scope confirmation**
-   [ ] Is all test assertion logic byte-for-byte identical before and after (only formatting
    and structure changed)?
-   [ ] Are all `src/` files unchanged?

## Validation Procedure

### Measurements

**M1 — Indentation check**
`grep -c '    ' module/claude_session/tests/responsibility_no_process_execution_test.rs`
Expected after: 0 (no 4-space indented lines remain).

**M2 — Inline escaping check**
`grep -c 'replace\|chars().map' module/claude_session/tests/detection_tests.rs`
Expected after: 0 (all inline logic removed).

### Anti-faking checks

**AF1 — Test count unchanged**
`cargo nextest run --all-features 2>&1 | grep 'tests run'` must show identical count
before and after. No test can be accidentally removed during the refactor.

**AF2 — get_claude_storage_path call count**
`grep -c 'get_claude_storage_path' module/claude_session/tests/detection_tests.rs`
Expected after: at least 5 (one call per previously duplicated test function).
