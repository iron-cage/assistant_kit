# TSK-003: Change `.sessions` default scope from `local` to `under`

## Goal

Unblock the common `clg .sessions scope::under` workflow by making `under` the
default scope so bare `clg .sessions` shows all sessions in the subtree rooted at
cwd — confirmed green when `w3 .test l::3` passes with zero regressions and
`clg .sessions` (no args) produces the same output as `clg .sessions scope::under`.

## In Scope

- `src/cli/mod.rs` line ~2136 — change `unwrap_or( "local" )` to `unwrap_or( "under" )`
- `unilang.commands.yaml` — change `default: "local"` to `default: "under"` for the `.sessions` `scope` parameter
- `tests/sessions_command_test.rs` — rename `ec7_omitted_scope_defaults_to_local` to
  `ec7_omitted_scope_defaults_to_under`; add discriminating fixture proving under behavior

## Out of Scope

- Changing default scope for any other command (`.list`, `.show`, etc.)
- Updating spec.md or CLI docs — covered in TSK-004 (spec) and TSK-005 (CLI docs)
- Adding new scope values or changing scope semantics

## Description

The user always runs `clg .sessions scope::under` — the extra argument is redundant.
Changing the default from `local` to `under` removes it. `under` shows all sessions
for every project whose path starts with cwd, which is the most useful default for a
workspace tool. Spec and CLI doc updates in TSK-004, TSK-005.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note constraints on CLI parameter defaults
   and code style.
2. **Write Test Matrix** — populate every row before writing test code.
3. **Write failing test** — rename `ec7_omitted_scope_defaults_to_local` to
   `ec7_omitted_scope_defaults_to_under`; update fixture with parent + child project
   to discriminate `local` vs `under` behavior. Confirm it fails with current code.
4. **Implement** — change `unwrap_or( "local" )` → `unwrap_or( "under" )` in
   `src/cli/mod.rs`; change `default: "local"` → `default: "under"` in
   `unilang.commands.yaml`.
5. **Green state** — `w3 .test l::3` passes with zero failures and zero warnings.
6. **Refactor if needed** — no function exceeds 50 lines; no duplication.
7. **Walk Validation Checklist** — every answer YES before marking done.
8. **Update task status** — set ✅, recalculate advisability to 0, re-sort index,
   move to `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `.sessions path::parent` (no scope), parent + child projects | New default | Same output as `scope::under path::parent` |
| T02 | `.sessions path::parent` (no scope), parent + child | New default | `session-child` (sub-project) visible |
| T03 | `.sessions scope::local` | Explicit override | Unchanged — shows only exact-match project |
| T04 | `.sessions scope::under` | Explicit value | Unchanged — shows subtree projects |

## Acceptance Criteria

- `unwrap_or( "local" )` no longer exists in `src/cli/mod.rs`
- `default: "under"` in `unilang.commands.yaml` for `.sessions` scope parameter
- `ec7_omitted_scope_defaults_to_under` test exists and passes
- `ec7_omitted_scope_defaults_to_local` no longer exists
- `w3 .test l::3` passes with zero failures

## Validation Checklist

Desired answer for every question is YES.

**Code changes**
- [ ] Is `unwrap_or( "under" )` present in `src/cli/mod.rs` line ~2136?
- [ ] Is `default: "under"` set for `scope` in `.sessions` block of `unilang.commands.yaml`?

**Tests**
- [ ] Does `ec7_omitted_scope_defaults_to_under` exist in `tests/sessions_command_test.rs`?
- [ ] Does `ec7_omitted_scope_defaults_to_local` no longer exist?

**Negative criteria**
- [ ] Does `grep -n 'unwrap_or.*"local"' src/cli/mod.rs` return zero results?

## Validation Procedure

### Measurements

**M1 — Default changed in source**
Command: `grep -c 'unwrap_or.*"under"' src/cli/mod.rs`
Before: 0. Expected: ≥1. Deviation: 0 = change not applied.

**M2 — YAML default updated**
Command: `grep -c 'default: "under"' unilang.commands.yaml`
Before: 0. Expected: ≥1. Deviation: 0 = YAML not updated.

**M3 — Test suite clean**
Command: `w3 .test l::3`
Before: 268 tests pass. Expected: ≥269 pass, 0 fail, 0 warnings.

### Anti-faking checks

**AF1 — Old default absent from source**
Command: `grep -n 'unwrap_or.*"local"' src/cli/mod.rs`
Expected: zero results.

## Outcomes

All three deliverables implemented and verified green: `unwrap_or( "under" )` in `src/cli/mod.rs:2136`, `default: "under"` in `unilang.commands.yaml`, and `ec7_omitted_scope_defaults_to_under` test with a discriminating parent+child project fixture. The critical design insight was that the old test used an empty HOME, producing identical output for both `local` and `under` (no projects → no discrimination). The new test creates two projects — parent at `/parent_proj` and child at `/parent_proj/child_sub` — so only `scope::under` includes the child session, giving a genuine discriminating assertion. `w3 .test l::3` passes at 100% (269 tests, zero warnings). Note: the installed `clg` binary still uses the old default until rebuilt — see TSK-006.
