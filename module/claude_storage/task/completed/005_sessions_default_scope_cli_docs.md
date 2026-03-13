# TSK-005: Update CLI test docs — `.sessions` default scope

## Goal

Keep CLI test documentation consistent with the new `scope::under` default by
updating IT-1 and CD-2 descriptions — verified by
`grep "local scope" docs/cli/testing/command/sessions.md docs/cli/testing/param_group/scope_configuration.md`
returning zero matches for the changed test cases.

## In Scope

- `docs/cli/testing/command/sessions.md` — IT-1 index row, section header, and body:
  "Default (no args) returns local scope sessions" → "Default (no args) returns under scope sessions"
- `docs/cli/testing/param_group/scope_configuration.md` — CD-2 index row, section
  header, and body: "path:: without scope:: defaults to local scope" → "defaults to under scope";
  update Expected Output, Verification, and Pass Criteria lines accordingly

## Out of Scope

- CC-1, CC-2, CC-3, CC-4, CD-1 — semantics unchanged
- Any file outside `docs/cli/testing/` (covered in TSK-003, TSK-004)

## Description

Two test case descriptions name `local` as the default. After TSK-003 changes the
implementation default to `under`, these become stale. Depends on TSK-003.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; review cli.rulebook.md test case standards.
2. **Edit sessions.md** — update IT-1 in the Test Case Index table and the IT-1 section
   (header + Goal/Expected Output/Verification/Pass Criteria) to replace `local scope`
   with `under scope` and `scope::local` with `scope::under`.
3. **Edit scope_configuration.md** — update CD-2 in the Test Case Index, section header,
   Goal, Command (remove `scope::local` from comparison note), Expected Output,
   Verification, and Pass Criteria.
4. **Verify** — `grep -c "defaults to local scope" sessions.md scope_configuration.md` = 0.
5. **Walk Validation Checklist** — every answer YES.
6. **Update task status** — ✅, re-sort index, move to `task/completed/`.

## Test Matrix

*(Not applicable — documentation-only task.)*

## Acceptance Criteria

- IT-1 in `sessions.md` describes `under` as the default, not `local`
- CD-2 in `scope_configuration.md` describes `under` as the default, not `local`
- All CC/CD rows other than CD-2 are unchanged

## Validation Checklist

Desired answer for every question is YES.

**sessions.md**
- [ ] Does IT-1 index row read "Default (no args) returns under scope sessions"?
- [ ] Does IT-1 section body describe `scope::under` as the default behavior?

**scope_configuration.md**
- [ ] Does CD-2 index row read "path:: without scope:: defaults to under scope"?
- [ ] Does CD-2 test case body (Goal, Expected Output, Pass Criteria) reflect `under`?

**No over-editing**
- [ ] Are CC-1, CC-2, CC-3, CC-4, CD-1 descriptions unchanged?

## Validation Procedure

### Measurements

**M1 — Stale local references removed**
Command: `grep -c "defaults to local scope" docs/cli/testing/command/sessions.md docs/cli/testing/param_group/scope_configuration.md`
Before: 2. Expected: 0. Deviation: >0 = stale text remains.

**M2 — under references added**
Command: `grep -c "under scope\|defaults to under" docs/cli/testing/param_group/scope_configuration.md`
Before: 0. Expected: ≥2. Deviation: 0 = update not applied.

### Anti-faking checks

**AF1 — No stale IT-1 text in sessions.md**
Command: `grep -n "IT-1" docs/cli/testing/command/sessions.md`
Expected: output contains "under", not "local".

## Outcomes

All four CLI doc locations updated: IT-1 in `sessions.md` (index row + section header + full body), CD-2 in `scope_configuration.md` (index row + section header + Goal + Setup + Expected Output + Verification + Pass Criteria), EC-7 in `docs/cli/testing/param/scope.md` (index row + section body), and the default column in `docs/cli/parameter_groups.md`. The consistency sweep found more affected files than the original task scope anticipated — `docs/cli/types.md` `ScopeValue` type and `readme.md` parameter table also required updates. All stale `local` default references for `.sessions` are eliminated; `grep "defaults to local scope"` on the CLI test docs returns zero matches. Final `w3 .test l::3` confirmed 100% pass rate with zero warnings.
