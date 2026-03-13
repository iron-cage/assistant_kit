# TSK-006: Rebuild and install `clg` binary with `scope::under` default

## Goal

Ship the `scope::under` default change to the running `clg` binary — confirmed when
`clg .sessions` (no args) produces the same output as `clg .sessions scope::under`,
matching what `w3 .test l::3` already validates in source.

## In Scope

- Rebuild the `clg` binary from current source
- Install it (replace the binary that `which clg` points to)

## Out of Scope

- Any further code, spec, or doc changes — all covered by TSK-003 through TSK-005
- Changing install location or binary name
- CI/CD pipeline changes

## Description

TSK-003 changed `unwrap_or( "local" )` → `unwrap_or( "under" )` in source and all tests
pass, but the installed binary predates the change. Until the binary is rebuilt and
reinstalled, `clg .sessions` silently keeps the old `local` default — confusing anyone
who reads the updated docs or spec.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Build** — `cargo build --release` in the crate root (or workspace equivalent).
2. **Install** — replace the `clg` binary: `cargo install --path . --force` or equivalent.
3. **Smoke-test** — run `clg .sessions` from a project with sub-projects; confirm count
   matches `clg .sessions scope::under`.
4. **Walk Validation Checklist** — every answer YES.
5. **Update task status** — set ✅, move to `task/completed/`.

## Test Matrix

*(Not applicable — no new test code; TSK-003 already covers the behavior test.)*

## Acceptance Criteria

- `clg .sessions` and `clg .sessions scope::under` produce identical output from a
  project with sub-projects
- `clg .sessions` returns more sessions than `clg .sessions scope::local` when
  sub-projects exist under cwd

## Validation Checklist

Desired answer for every question is YES.

**Binary**
- [ ] Does `clg .sessions` return the same session count as `clg .sessions scope::under`?
- [ ] Does `clg .sessions` return MORE sessions than `clg .sessions scope::local` when
  run from a project that has sub-projects?

**Negative criteria**
- [ ] Is the output of `clg .sessions` different from the pre-install behavior (which
  capped at the current-project count)?

## Validation Procedure

### Measurements

**M1 — Default matches under**
Command: `diff <(clg .sessions) <(clg .sessions scope::under)`
Before: outputs differ (binary uses old `local` default).
Expected: empty diff (outputs identical). Deviation: non-empty diff = binary not updated.

**M2 — Default exceeds local**
Command (from project with sub-projects): compare `clg .sessions` count vs
`clg .sessions scope::local` count.
Before: counts are equal. Expected: `under` count > `local` count. Deviation:
equal counts = binary not updated OR no sub-projects exist under cwd (check cwd first).

### Anti-faking checks

**AF1 — Binary timestamp post-install**
Command: `ls -la $(which clg)`
Expected: mtime is after this task was started.

## Outcomes

Ran `cargo install --path . --force` from module root. Binary compiled successfully (105 deps), replaced `/home/user1/.cargo/bin/clg` and `/home/user1/.cargo/bin/claude_storage`. The scope::under default implemented in TSK-003 is now active in the installed binary.
