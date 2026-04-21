# Implement `scope::around` for `.projects` and make it the default

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** 🎯 (Available)
- **Validated By:** null
- **Validation Date:** null

## Goal

Add `scope::around` (bidirectional: ancestors + current + descendants, equivalent to `relevant ∪ under` deduplicated) to `projects_routine` and change the default from `"under"` to `"around"`, so that bare `clg .projects` surfaces the neighborhood a developer is actively working in rather than the entire subtree. (Motivated: `scope::under` default causes `task/-default_topic` to always appear as "Active project" in the dream workspace because it's the most-recently-active session under cwd — a systematic false-positive that misleads the user; Observable: `clg .projects` from a project directory shows ancestor and descendant projects but excludes sibling subtrees; Scoped: `src/cli/mod.rs` § `projects_routine` — default string, validation match, error message, and `project_matches` closure; Testable: `cargo nextest run --test projects_scope_around_test` passes all 3 new integration tests.)

The `around` scope mirrors kbase's 5th `DiscoveryScope` variant. It is the union of `relevant` (ancestor walk ↑) and `under` (subtree descent ↓), with deduplication guaranteed by the BTreeMap keying on decoded project path. This gives a "neighborhood" view: everything above cwd + everything below cwd, minus unrelated sibling branches.

Making it the default for `.projects` aligns the command with its primary use case: a developer running `clg .projects` from their current project wants to see related context (parent workspaces and submodules), not an arbitrary subtree.

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs`
  § `projects_routine` line ~2462 — change `unwrap_or("under")` to `unwrap_or("around")`
  § `projects_routine` line ~2464 — add `"around"` to valid values match
  § `projects_routine` line ~2468 — update error message to include `around`
  § `project_matches` closure line ~2604 — add `"around"` arm (union of `relevant` + `under`)
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/` — new test file `projects_scope_around_test.rs`
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/readme.md` — register new test file

## Out of Scope

- Documentation updates (already completed by doc_tsk in this session)
- `unilang.commands.yaml` description change (already completed by doc_tsk in this session)
- Other commands using `scope::` — only `.projects` gains the new default
- Changing `scope::around` semantics for any other command

## Description

The `project_matches` closure in `projects_routine` handles 4 scope arms: `global`, `local`, `under`, `relevant`. The new `around` arm combines `under` and `relevant`:

```rust
"around" =>
{
  // Union of under + relevant — bidirectional neighborhood.
  // BTreeMap key on decoded path deduplicates projects matched by both arms.
  let is_under = {
    if dir_name != eb && !dir_name.starts_with( &format!( "{eb}-" ) ) { false }
    else if dir_name == eb { true }
    else {
      let candidate_base = dir_name.find( "--" ).map_or( dir_name, | i | &dir_name[ ..i ] );
      decode_path_via_fs( candidate_base )
        .map_or( true, | p | p.starts_with( &base_path ) )
    }
  };
  let is_relevant = {
    if !is_relevant_encoded( dir_name, eb ) { false }
    else {
      let candidate_base = dir_name.find( "--" ).map_or( dir_name, | i | &dir_name[ ..i ] );
      if candidate_base == eb { true }
      else {
        decode_path_via_fs( candidate_base )
          .map_or( true, | p | base_path.starts_with( &p ) )
      }
    }
  };
  is_under || is_relevant
},
```

The fallback arm `_          => false` remains to satisfy exhaustiveness.

## Requirements

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
- No mocking — all tests use real `clg` binary via `common::clg_cmd()`
- TDD: write failing test first, then implement, then verify
- The `"around"` arm must reuse the exact same conditions as `"under"` and `"relevant"` arms — no divergent logic
- Fix documentation comment: 3-field format (`Fix(issue)`, `Root cause`, `Pitfall`) is required if a known issue is addressed; for a new feature no issue tag is needed

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note `code_style.rulebook.md` indent rules and `test_organization.rulebook.md` IT numbering.
2. **Read source** — Read `src/cli/mod.rs` lines 2435–2610 (full `projects_routine` + `project_matches` closure) to understand current state before writing any test.
3. **Determine next IT number** — `grep -r "^fn it[0-9]*_" tests/` to find the highest current IT number; allocate three sequential IDs for new tests.
4. **Write failing tests** — Create `tests/projects_scope_around_test.rs` with IT-N, IT-N+1, IT-N+2 covering the three test matrix rows. Run `cargo nextest run --test projects_scope_around_test` and confirm all three FAIL (scope::around not yet valid).
5. **Implement `around` arm** — Edit `src/cli/mod.rs`:
   a. Line ~2462: `unwrap_or("under")` → `unwrap_or("around")`
   b. Line ~2464: add `"around"` to the valid values match
   c. Line ~2468: update error message string
   d. Add `"around"` arm to `project_matches` closure (before the `_          => false` fallback)
6. **Validate targeted tests** — `cargo nextest run --test projects_scope_around_test` — all 3 must pass.
7. **Validate full suite** — `w3 .test level::3` — zero failures, zero warnings.
8. **Register test file** — Add row to `tests/readme.md` Responsibility Table; update test count.
9. **Walk Validation Checklist** — check every item. Every answer must be YES.

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|----------------|-------------------|-------------------|
| `scope::around` from mid-level dir (A/B), project at A and A/B/C exist | around union | All three (A, A/B, A/B/C) visible; A/B/D sibling excluded |
| Default (no scope) from project dir with parent and child project | Default = around | Same as explicit `scope::around` — parent + self + child visible |
| `scope::around` with no ancestor or descendant projects | Degenerate case | Only the exact cwd project appears (same as scope::local) |

## Acceptance Criteria

- `grep -c 'unwrap_or.*"around"' src/cli/mod.rs` returns `1`
- `grep -c '"around"' src/cli/mod.rs` returns ≥3 (valid values, error message, closure arm)
- `cargo nextest run --test projects_scope_around_test` exits 0
- `w3 .test level::3` exits 0 (full suite clean)
- `tests/projects_scope_around_test.rs` registered in `tests/readme.md`

## Validation

### Checklist

Desired answer for every question is YES.

**Default change**
- [ ] Is `unwrap_or("around")` present in `src/cli/mod.rs`?
- [ ] Is `unwrap_or("under")` absent from `projects_routine` (the old default)?

**Validation guard**
- [ ] Does the valid-values match in `projects_routine` include `"around"`?
- [ ] Does the error message string include `around`?

**Closure arm**
- [ ] Is an `"around"` arm present in the `project_matches` closure?
- [ ] Does it return `true` for a descendant project (same as `under`)?
- [ ] Does it return `true` for an ancestor project (same as `relevant`)?
- [ ] Does it return `false` for a sibling project?

**Tests**
- [ ] Does `tests/projects_scope_around_test.rs` exist?
- [ ] Do all 3 new integration tests pass?
- [ ] Is the file registered in `tests/readme.md`?

**Out of Scope confirmation**
- [ ] Are all other commands' scope defaults unchanged?
- [ ] Is `docs/` unchanged (all doc updates already done)?

### Measurements

**M1 — Default changed in source**
Command: `grep -c 'unwrap_or.*"around"' src/cli/mod.rs`
Before: 0. Expected: 1. Deviation: 0 = change not applied.

**M2 — `around` arm present in closure**
Command: `grep -c '"around"' src/cli/mod.rs`
Before: 0. Expected: ≥3. Deviation: <3 = partial implementation.

**M3 — New tests pass**
Command: `cargo nextest run --test projects_scope_around_test 2>&1 | tail -3`
Before: test file does not exist / compile error. Expected: `3 passed`. Deviation: any failure.

**M4 — Full suite clean**
Command: `w3 .test level::3 2>&1 | grep "^Summary:"`
Expected: all crates passed, 0 failures. Deviation: any failure.

### Invariants

- [ ] I1 — test suite: `w3 .test level::3` → 0 failures, 0 warnings

### Anti-faking checks

**AF1 — Old default absent**
Check: `grep -n 'unwrap_or.*"under"' src/cli/mod.rs`
Expected: zero results in `projects_routine` context. Why: confirms the default was actually changed, not just a new variable added.

**AF2 — `around` arm is genuinely bidirectional**
Check: `grep -A 15 '"around"' src/cli/mod.rs | grep -c "is_under\|is_relevant"`
Expected: ≥2. Why: confirms the arm combines both directions rather than only one.

**AF3 — Sibling exclusion preserved**
Check: Test IT-N (the 3-project scenario) includes a sibling project assertion that checks the sibling is NOT present in `scope::around` output.
Expected: assertion `!s.contains("sibling_project_name")` present in the test.

## Outcomes

[Empty — populated upon task completion]
