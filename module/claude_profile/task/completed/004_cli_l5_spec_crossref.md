# Add IT-N Spec Cross-References to CLI Test File Matrices

## Execution State

- **Executor Type:** any
- **filed_by:** agent
- **actor:** null
- **started_at:** 2026-07-01
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Done)
- **closes:** null
- **dir:** tests/cli/
- **validated_by:** tst subagent
- **validation_date:** 2026-07-01

## Goal

The 2026-07-01 L5 normalization session added 46 IT-N/EC-N spec detail entries across
6 spec files (`13_account_rotate.md`, `14_account_renewal.md`, `15_account_inspect.md`,
`16_account_assign.md`, `18_account_unclaim.md`, `63_owner.md`). All 46 entries have
Rust test function implementations. However, the test functions have no `Spec:` doc
comment annotations referencing these IT-N/EC-N identifiers, leaving the
spec→implementation traceability link undocumented.

Observable end-state: every IT-N entry in the 6 spec files is traceable to its
implementing test function via a `/// Spec: [tests/docs/cli/command/NN_*.md IT-N]` or
`/// Spec: [tests/docs/cli/param/NN_*.md EC-NN]` doc comment line placed above the
`#[test]` attribute; an IT-N column is added to the existing matrix in
`account_rotate_test.rs` and `account_renewal_test.rs`; a `/// Spec:` annotation
is added to `arc02` in `account_renewal_test_b.rs` (single annotation, no matrix change
in that file); for `account_inspect_test.rs`, an IT-N column is added to the existing
matrix; for `account_inspect_test_b.rs` (no existing matrix), a `## Spec Map` subsection
is added to the file-level doc comment; `/// Spec:` annotations are added to `ft12_account_assign_fully_deregistered` and
`ft11b_account_unclaim_no_args` in `accounts_ft_test.rs`; a `## EC-N Spec Map`
subsection mapping ec1..ec9 function names to EC-01..EC-09 spec IDs is added to the
file-level doc comment in `account_owner_param_test.rs`; all tests pass.

## In Scope

- `tests/cli/account_rotate_test.rs` — add `/// Spec: [tests/docs/cli/command/13_account_rotate.md IT-N]`
  doc comment above the `#[test]` attribute of `rot01` (IT-1), `rot02` (IT-2), `rot03` (IT-3);
  the existing inline `// IT-N:` body comments remain unchanged;
  append an `IT-N` column (rightmost) to the existing matrix header with values
  IT-1 / IT-2 / IT-3 in the rot01 / rot02 / rot03 rows respectively

- `tests/cli/account_renewal_test.rs` — add `/// Spec:` doc comment lines to the
  15 mapped functions (ft03, ft13, ft16 are excluded — no corresponding IT-N entry in spec):
  ft01=IT-1, ft02=IT-2, ft04=IT-3, ft07=IT-4, ft08=IT-5, ft09=IT-6,
  ft10=IT-7, ft06=IT-8, ft05=IT-9, ft11=IT-10, ft12=IT-11, ft14=IT-13,
  ft15=IT-14, ft17=IT-15, ft18=IT-16; append an `IT-N` column (rightmost) to the
  matrix header; each mapped row's IT-N cell uses the value from the mapping above;
  rows for ft03/ft13/ft16 have no IT-N cell (no spec entry for those functions);
  the matrix rows use `arnNN` IDs — each row's function column contains the `ft`-prefixed
  function name; match the `ft` prefix against the mapping above to determine the IT-N
  cell value for each `arn*` row (e.g., row `arn01` lists function `ft01...` → IT-1)

- `tests/cli/account_renewal_test_b.rs` — add `Spec:` line to `arc02`
  pointing to `tests/docs/cli/command/14_account_renewal.md` IT-12

- `tests/cli/account_inspect_test.rs` — add `/// Spec:` doc comment lines to
  ai06=IT-1, ai07=IT-2/IT-10, ai11=IT-3, ai13=IT-4, ai02=IT-6,
  ai05=IT-7 (closest match — no exact `unknown::x` test exists; annotate `ai05`
  with `/// Spec: [tests/docs/cli/command/15_account_inspect.md IT-7]` as the
  accepted closest match for parameter rejection behavior); multi-IT functions get one combined annotation
  line (e.g., `/// Spec: [tests/docs/cli/command/15_account_inspect.md IT-2/IT-10]`);
  append an IT-N column (rightmost) to the existing matrix using IT-N values from
  the mapping above (multi-IT cell values e.g. "IT-2/IT-10" for ai07)

- `tests/cli/account_inspect_test_b.rs` — add `/// Spec:` doc comment lines to
  lim_it_ai20=IT-5+IT-12, lim_it_ai16=IT-8+IT-9, lim_it_ai15=IT-11,
  lim_it_ai22=IT-13, lim_it_ai23=IT-14, lim_it_ai24=IT-15, lim_it_ai19=IT-16;
  multi-IT functions get one combined annotation line; this file has no existing
  matrix — add a `## Spec Map` subsection to the file-level doc comment with a table
  listing each lim_it_* function name alongside its IT-N value(s)

- `tests/cli/accounts_ft_test.rs` — add `Spec: [tests/docs/cli/command/16_account_assign.md IT-1]`
  to `ft12_account_assign_fully_deregistered`; add
  `Spec: [tests/docs/cli/command/18_account_unclaim.md IT-1]` to
  `ft11b_account_unclaim_no_args`

- `tests/cli/account_owner_param_test.rs` — add a `## EC-N Spec Map` subsection to
  the file-level matrix explicitly mapping ec1..ec9 to EC-01..EC-09 from
  `tests/docs/cli/param/63_owner.md`

## Out of Scope

- Any behavioral changes to test function bodies
- New test function creation (all 46 spec entries are already implemented)
- `account_assign_test.rs` assign-param tests (not command-redirect tests)
- `account_ownership_test.rs` `ft03` — multi-assertion function; traceability via
  `accounts_ft_test.rs` ft12 / ft11b is sufficient
- Source code under `src/` — no production code changes needed
- Spec file edits (completed in 2026-07-01 normalization session)
- EC-10..EC-20 in `tests/docs/cli/param/63_owner.md` — per-function `Spec:` annotations
  for these entries already exist in `account_owner_param_test.rs`; only the file-level
  matrix consolidation for EC-01..EC-09 is missing

## Work Procedure

1. Read `tests/docs/cli/command/13_account_rotate.md` to confirm IT-1/2/3 IDs
2. Read `tests/cli/account_rotate_test.rs`; add `/// Spec:` lines to rot01/02/03;
   add IT-N column to the matrix header
3. Read `tests/docs/cli/command/14_account_renewal.md` to confirm IT-1..16 IDs
4. Read `tests/cli/account_renewal_test.rs`; build ft→IT mapping from the In Scope
   table; add `/// Spec:` lines to ft01–ft18; add IT-N column to matrix header;
   read `tests/cli/account_renewal_test_b.rs`; add `/// Spec:` line to arc02 (IT-12)
5. Read `tests/docs/cli/command/15_account_inspect.md` to confirm IT-1..16 IDs
6. Read `tests/cli/account_inspect_test.rs`; add `/// Spec:` doc comment lines per
   the In Scope mapping; add IT-N column to the existing matrix; then read
   `tests/cli/account_inspect_test_b.rs`; add `/// Spec:` doc comment lines to the
   lim_it_* functions; add a `## Spec Map` subsection to the file-level doc comment
   (that file has no existing matrix) listing each annotated function and its IT-N value
7. Read `tests/cli/accounts_ft_test.rs`; add `Spec:` lines to ft12 and ft11b
8. Read `tests/cli/account_owner_param_test.rs`; add EC-N Spec Map subsection to
   file-level doc comment
9. Run `w3 .test level::1` in container to confirm all tests pass and no compile errors

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|----------------|-------------------|-------------------|
| Grep for `Spec.*13_account_rotate` | `account_rotate_test.rs` | 3 annotations found (IT-1, IT-2, IT-3) |
| Grep for `Spec.*14_account_renewal` | `account_renewal_test.rs` + `account_renewal_test_b.rs` | 16 annotations found (IT-1..IT-16) |
| Grep for `Spec.*15_account_inspect` | `account_inspect_test.rs` | 6 annotation lines found (ai06, ai07, ai11, ai13, ai02, ai05) |
| Grep for `Spec.*15_account_inspect` | `account_inspect_test_b.rs` | 7 annotation lines found (lim_it_ai20, lim_it_ai16, lim_it_ai15, lim_it_ai22, lim_it_ai23, lim_it_ai24, lim_it_ai19) |
| Grep for `Spec.*16_account_assign` | `accounts_ft_test.rs` | 1 annotation found (IT-1 on ft12) |
| Grep for `Spec.*18_account_unclaim` | `accounts_ft_test.rs` | 1 annotation found (IT-1 on ft11b) |
| Grep for `## EC-N Spec Map` | `account_owner_param_test.rs` | 1 match (section header present in file-level doc comment) |
| Grep for `\| IT-N` | `account_rotate_test.rs` | 1 match (IT-N column header in matrix) |
| Grep for `\| IT-N` | `account_renewal_test.rs` | 1 match (IT-N column header in matrix) |
| Grep for `\| IT-N` | `account_inspect_test.rs` | 1 match (IT-N column header in matrix) |
| Grep for `## Spec Map` | `account_inspect_test_b.rs` | 1 match (subsection header in file-level doc comment) |
| `w3 .test level::1` | full test suite | All tests pass; zero compile errors |

## Related Documentation

- `tests/docs/cli/command/13_account_rotate.md` — spec for rotate redirect (IT-1..3)
- `tests/docs/cli/command/14_account_renewal.md` — spec for .account.renewal (IT-1..16)
- `tests/docs/cli/command/15_account_inspect.md` — spec for .account.inspect (IT-1..16)
- `tests/docs/cli/command/16_account_assign.md` — spec for .account.assign stub (IT-1)
- `tests/docs/cli/command/18_account_unclaim.md` — spec for .account.unclaim stub (IT-1)
- `tests/docs/cli/param/63_owner.md` — spec for owner:: parameter (EC-01..EC-09)
- `docs/cli/readme.md` — completion matrix (L5 achieved after normalization session)

## History

- **[2026-07-01]** `CREATED` — Link IT-N/EC-N spec IDs to implementing test functions for the 6 collection instances updated in the L5 normalization session.

## Verification Findings

First MAAV dispatch (2026-07-01) — FAIL on dimension 4:

**Dimension 1 (Scope Coherence): CONDITIONAL PASS**
`63_owner.md` contains EC-01..EC-20 (20 entries); task In Scope covers only EC-01..EC-09
with no explicit exclusion of EC-10..EC-20, leaving scope boundary ambiguous.
Fix: added explicit Out of Scope bullet for EC-10..EC-20.

**Dimension 2 (MOST Quality): PASS**

**Dimension 3 (Value/YAGNI): PASS**

**Dimension 4 (Implementation Readiness): FAIL**
`arc02` (IT-12 mapping) lives in `tests/cli/account_renewal_test_b.rs`, not
`tests/cli/account_renewal_test.rs`. Step 4 and In Scope both named only
`account_renewal_test.rs`, making `arc02` unlocatable for the implementer.
Fix: split renewal In Scope bullet into two entries (one per file); amended step 4
to read both files and name arc02 target explicitly; updated Test Matrix row.

Second MAAV dispatch (2026-07-01) — FAIL on dimensions 2 and 4:

**Dimension 1 (Scope Coherence): PASS**

**Dimension 2 (MOST Quality): FAIL**
- Observable: "spec-ID column or a `Spec Map` subsection" used disjunctive "or"; an external
  verifier could not determine compliance without consulting In Scope section.
  Fix: rewrote end-state to name specific matrix update for each file type explicitly.
- Testable: EC-0[1-9] grep expected behavior was "Spec Map lists EC-01..EC-09" — prose
  assertion rather than count-based predicate.
  Fix: changed to "9 matches found; `## EC-N Spec Map` subsection present mapping ec1..ec9
  to EC-01..EC-09."

**Dimension 3 (Value/YAGNI): PASS**

**Dimension 4 (Implementation Readiness): FAIL**
- FAIL-A: In Scope said "add `Spec:` lines to `ft01`–`ft18`" but ft03, ft13, ft16 are within
  that range and have no IT mapping; developer could not determine whether to skip or annotate them.
  Fix: changed "ft01–ft18" to "15 mapped functions" with explicit exclusion note for ft03/ft13/ft16.
- FAIL-B: `account_rotate_test.rs` bullet did not specify annotation placement (above `#[test]`
  vs inside body like existing `// IT-N:` comments) or IT-N column values.
  Fix: expanded bullet to name exact annotation format, note that body comments remain, and
  specify IT-1/IT-2/IT-3 column values for rot01/rot02/rot03.

Third MAAV dispatch (2026-07-01) — FAIL on dimension 2:

**Dimension 1 (Scope Coherence): PASS**

**Dimension 2 (MOST Quality): FAIL**
- Observable (O): Goal paragraph did not cover inspect files in the matrix-update description;
  delegated to In Scope via "per its In Scope specification".
  Fix: rewrote Goal to name IT-N column for rotate/renewal/inspect/inspect_b explicitly;
  named EC-N Spec Map for account_owner_param_test.rs directly.
- Testable (T-1): "9 matches found" for `EC-0[1-9]` grep ambiguous — pre-task baseline
  unknown; pre-existing EC-N content could inflate count.
  Fix: changed row to grep for `## EC-N Spec Map` (1 match) — uniquely identifies the
  deliverable with a baseline of 0 pre-task.
- Testable (T-2): subsection content assertion "mapping ec1..ec9 to EC-01..EC-09" was
  prose requiring human judgment.
  Fix: row now verifies section header existence (grep → 1 match), which is mechanically
  verifiable; content verified implicitly by Work Procedure step 8.
- Testable (T-3): "16 annotations found" for inspect ambiguous — 13 annotation lines cover
  16 IT entries because ai07/lim_it_ai20/lim_it_ai16 each cover 2 ITs.
  Fix: changed to "13 annotation lines found (covering IT-1..IT-16; ai07, lim_it_ai20,
  lim_it_ai16 each cover 2 ITs)".
- Also: inspect In Scope bullet lacked explicit IT-N column instruction.
  Fix: added "add an IT-N column to the matrix in each file" with multi-IT format example.

**Dimension 3 (Value/YAGNI): PASS**

**Dimension 4 (Implementation Readiness): PASS** (minor note: inspect matrix column
per-row values inferrable from mapping table; not blocking)

Fourth MAAV dispatch (2026-07-01) — FAIL on dimensions 2 and 4:

**Dimension 1 (Scope Coherence): PASS**

**Dimension 2 (MOST Quality): FAIL**
- Observable: `account_renewal_test_b.rs` not named in Goal end-state as an annotation
  target; external verifier could not detect a missing arc02 annotation by reading Goal alone.
  Fix: added explicit "a `/// Spec:` annotation is added to `arc02` in `account_renewal_test_b.rs`
  (single annotation, no matrix change)" to Goal paragraph.
- Testable (T-3): "13 annotation lines found (covering IT-1..IT-16)" claimed coverage
  completeness as a parenthetical, which is prose not mechanically verifiable.
  Fix: split into two file-specific rows (6 functions in `_test.rs`, 7 in `_test_b.rs`)
  with explicit function names in parenthetical — count-based and per-file verifiable.

**Dimension 3 (Value/YAGNI): PASS**

**Dimension 4 (Implementation Readiness): FAIL**
- Q1/Ambiguity A: `account_inspect_test_b.rs` has no existing matrix; "add IT-N column"
  was unexecutable for that file. Fix: split inspect into two separate In Scope bullets
  (one per file); `_b.rs` bullet specifies "add `## Spec Map` subsection to file-level
  doc comment" instead of column addition.
- Q4/Ambiguity B: multi-IT function annotation format unspecified (one combined line vs
  two separate lines). Fix: added "(multi-IT functions get one combined annotation line)"
  to both inspect bullets with format example.
- Q4/Ambiguity C: IT-7/ai05 semantic mismatch (ai05 tests invalid format value; IT-7
  specifies unknown parameter key). Fix: added "(closest match — tests rejection of an
  invalid format value, analogous to IT-7 parameter rejection scenario)" note.
- Step 6 "update or add matrix section to both files" was ambiguous for `_b.rs` (no
  matrix). Fix: rewrote step 6 to distinguish _test.rs (add IT-N column) from _test_b.rs
  (add ## Spec Map subsection to file-level doc comment).

Fifth MAAV dispatch (2026-07-01) — FAIL on dimensions 1, 2, and 4:

**Dimension 1 (Scope Coherence): FAIL**
- Matrix-column additions (rotate, renewal, inspect) and `## Spec Map` subsection
  (inspect_b) are In Scope items with no corresponding Test Matrix verification rows.
  Fix: added 4 grep rows for column/subsection presence (`\| IT-N` for matrix files,
  `## Spec Map` for inspect_b).
- Per-row IT-N cell values for renewal matrix unspecified.
  Fix: added "each mapped row's IT-N cell uses the value from the mapping above;
  rows for ft03/ft13/ft16 have no IT-N cell."

**Dimension 2 (MOST Quality): FAIL**
- Observable: `accounts_ft_test.rs` (ft12 + ft11b) absent from Goal end-state.
  Fix: added "/// Spec: annotations are added to ft12... and ft11b... in
  accounts_ft_test.rs" to Goal paragraph.

**Dimension 3 (Value/YAGNI): PASS**

**Dimension 4 (Implementation Readiness): FAIL**
- IT-7/ai05 directive missing: "closest match" note was advisory prose without
  explicit instruction. Fix: changed to "annotate `ai05` with `/// Spec: [...IT-7]`
  as the accepted closest match; no exact `unknown::x` test exists."
- IT-N column position unspecified. Fix: changed all three column instructions
  to "append (rightmost)."
- Inspect _test.rs multi-IT cell values in matrix unspecified.
  Fix: added "(multi-IT cell values e.g. 'IT-2/IT-10' for ai07)" to inspect bullet.

Sixth MAAV dispatch (2026-07-01) — FAIL on dimensions 2 and 4:

**Dimension 1 (Scope Coherence): PASS**

**Dimension 2 (MOST Quality): FAIL**
- Observable: Goal named `## EC-N Spec Map` subsection but did not state its content
  (ec1..ec9 → EC-01..EC-09). Test Matrix verifies only header presence.
  Fix: added "mapping ec1..ec9 function names to EC-01..EC-09 spec IDs" to Goal paragraph.

**Dimension 3 (Value/YAGNI): PASS**

**Dimension 4 (Implementation Readiness): FAIL**
- Renewal matrix rows use `arnNN` IDs while In Scope mapping uses `ftNN` identifiers;
  no cross-reference provided — implementer could not determine IT-N cell value for each
  `arn*` row without undocumented intermediate lookup.
  Fix: added note to renewal In Scope bullet explaining `arnNN` row IDs, that each row's
  function column contains the `ft`-prefixed function name, and how to match against the
  mapping (e.g., row arn01 lists function ft01... → IT-1).

## Verification Record

Seventh MAAV dispatch (2026-07-01) — ALL 4 PASS.

**Dimension 1 (Scope Coherence): PASS**
Every In Scope deliverable maps to at least one Test Matrix row. Seven files covered, no
scope gaps, no ambiguities. Out of Scope explicit with 6 bullets.

**Dimension 2 (MOST Quality): PASS**
Motivated: concrete gap from 2026-07-01 normalization session. Observable: Goal paragraph
self-sufficient; names every file and artifact; EC-N Spec Map content (ec1..ec9 → EC-01..EC-09)
stated directly. Scoped: bounded to 7 test files. Testable: all 12 Test Matrix rows are
count-based or compile-pass binary predicates; no prose assertions.

**Dimension 3 (Value/YAGNI): PASS**
46 IT-N/EC-N spec entries confirmed in 6 spec files; all test functions confirmed present;
no `Spec:` annotations exist pre-task; work is annotation-only with no behavioral changes.

**Dimension 4 (Implementation Readiness): PASS**
All named functions verified in specified files (30+ functions grepped). IT-7/ai05 handling
explicit ("annotate ai05 as accepted closest match"). `arnNN`/`ftNN` cross-reference explained
with example. Column positions specified (rightmost). Step 6 unambiguous for both inspect
files. No blocking implicit knowledge requirements remain.
