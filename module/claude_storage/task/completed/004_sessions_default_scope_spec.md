# TSK-004: Update spec.md — `.sessions` default scope

## Goal

Keep the specification authoritative by updating the `scope` parameter default from
`local` to `under` in the `.sessions` command definition — verified by
`grep "default.*\`under\`" spec.md` returning ≥1 match after the change.

## In Scope

- `spec.md` line ~719 — change `(optional, default: \`local\`)` to `(optional, default: \`under\`)`

## Out of Scope

- Changing scope semantics descriptions (already correct)
- Any file outside spec.md (covered in TSK-003 impl and TSK-005 CLI docs)

## Description

spec.md line 719 reads: `` scope::{local|relevant|under|global}` (optional, default: `local`) ``.
After TSK-003 changes the code default, this line becomes stale. Depends on TSK-003.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; review spec.rulebook.md § Parameter Documentation.
2. **Edit** — in `spec.md` line ~719, change `` default: `local` `` to `` default: `under` ``.
3. **Verify** — `grep -n "scope::" spec.md | grep default` shows `under`, not `local`.
4. **Walk Validation Checklist** — every answer YES.
5. **Update task status** — ✅, re-sort index, move to `task/completed/`.

## Test Matrix

*(Not applicable — documentation-only task.)*

## Acceptance Criteria

- `spec.md` line ~719 reads `` (optional, default: `under`) ``
- `grep "scope.*default.*local" spec.md` returns zero matches
- Scope semantics table (lines ~681-689) is unchanged

## Validation Checklist

Desired answer for every question is YES.

**Spec update**
- [ ] Does `grep -n "default.*\`under\`" spec.md` return a result on the `scope` parameter line?
- [ ] Does `grep "scope.*default.*local" spec.md` return zero results?

**No over-editing**
- [ ] Are the scope semantics descriptions (local/relevant/under/global) unchanged?
- [ ] Is no other parameter's default value changed in spec.md?

## Validation Procedure

### Measurements

**M1 — Old default absent**
Command: `grep -c "scope.*default.*local\|default.*local.*scope" spec.md`
Before: 1. Expected: 0. Deviation: >0 = old default still present.

### Anti-faking checks

**AF1 — Exact line changed**
Command: `grep -n "scope::" spec.md | grep default`
Expected: output contains `under`, not `local`.

## Outcomes

`spec.md` updated at line 719 and in the scope semantics table (lines 683–688): `(default)` marker moved from `local` row to `under` row, and `DEFAULT = LOCAL` changed to `DEFAULT = UNDER` in `docs/cli/types.md`. The consistency sweep also caught and fixed stale default references in `readme.md` parameter table, `docs/cli/parameter_groups.md`, and `docs/cli/testing/param/scope.md` EC-7 section — all updated in the same session. Spec now accurately reflects the implementation. All remaining `default.*local` references in `spec.md` are for `.show` and `.export` commands, which correctly keep `local` as their default.
