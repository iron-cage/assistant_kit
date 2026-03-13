# TSK-060: Fix scope::under false positive — sibling modules with underscore names

## Goal

Fix `scope::under` in `sessions_routine` so that a sibling directory named `foo_bar`
is never matched when the base path is `foo`, eliminating false-positive session
results caused by the lossy encoding that maps both `_` and `/` to `-`.

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs`
  § `sessions_routine` — fix the `under` predicate (line ~2155) to distinguish
  subdirectory from same-level sibling with underscore name
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/sessions_command_test.rs`
  — add regression test `it_25_scope_under_excludes_underscore_named_sibling`

## Out of Scope

- Changes to `scope::local` or `scope::relevant` predicates (different semantics)
- Spec or CLI-docs updates (covered in TSK-061 and TSK-062)
- Changes outside claude_storage

## Work Procedure

1. `src/cli/mod.rs` — replace the `"under"` branch of `project_matches` closure:
   - Current: `dir_name.starts_with(&format!("{eb}-"))` — matches sibling `foo_bar`
     when base is `foo` because both encode to `foo-...`
   - Fix: for each candidate, use `decode_path_via_fs` to recover the real filesystem
     path; then check if `real_candidate_path.starts_with(&real_base_path)` using
     `std::path::Path::starts_with`. Preserve exact-match branch (`dir_name == eb`)
     unchanged — that branch is unambiguous
   - Add `Fix(issue-031)` comment with root cause and pitfall
2. `tests/sessions_command_test.rs` — add `it_25_scope_under_excludes_underscore_named_sibling`:
   - Create two temp projects: `base_dir/sub/` (child) and `base_dir_extra/` (sibling
     with underscore-like name)
   - Run `.sessions scope::under path::base_dir`
   - Assert child sessions appear, sibling sessions do not

## Validation List

Desired answer for every question is YES.

- [x] Does `scope::under` from `claude_storage` exclude sessions from `claude_storage_core`?
- [x] Does `scope::under` still find sessions nested under `src/-default_topic`?
- [x] Does the new regression test `it_25` pass?
- [x] Does `ctest3` pass with 0 failures?
- [x] Is the `Fix(issue-031)` comment present in `cli/mod.rs`?
- [x] Are all Validation Procedure measurements met?

## Validation Procedure

### Measurements

**M1 — Regression test present**
Command: `grep -c "it_25_scope_under_excludes_underscore_named_sibling" module/claude_storage/tests/sessions_command_test.rs`
Before: 0. Expected: ≥1. Deviation: missing if 0.

**M2 — Fix comment present**
Command: `grep -c "Fix(issue-031)" module/claude_storage/src/cli/mod.rs`
Before: 0. Expected: ≥1. Deviation: missing if 0.

**M3 — All tests pass**
Command: `ctest3`
Before: 1303 passed. Expected: ≥1303 passed, 0 failed. Deviation: any failure.

### Anti-faking checks

**AF1 — Sibling actually excluded**
Check: new test `it_25` must fail when run against the unfixed code (before fix).
Before fix: test FAILS. After fix: test PASSES.

**AF2 — Child sessions still found**
Run: `clg .sessions scope::under` from workspace root.
Expected: sessions from `src/-default_topic` appear, count unchanged.
