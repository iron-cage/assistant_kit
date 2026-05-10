# Fix `account::delete()` — remove snapshot files alongside credentials

## Execution State

- **Executor Type:** any
- **Actor:** claude-sonnet-4-6
- **Claimed At:** 2026-05-10
- **Status:** ✅ (Complete)
- **Validated By:** claude-sonnet-4-6
- **Validation Date:** 2026-05-10

## Goal

Fix `account::delete()` in `claude_profile_core/src/account.rs` so that deleting a named account removes all three files created by `save()`: the credentials file and both snapshots (`.claude.json`, `.settings.json`). (Motivated: `save()` creates 3 files; `delete()` currently removes only 1, leaving orphaned snapshot files that accumulate silently after each deletion; Observable: after `clp .account.delete name::ACCOUNT`, all three `{name}.credentials.json`, `{name}.claude.json`, and `{name}.settings.json` are absent from the credential store; Scoped: `account::delete()` in `claude_profile_core/src/account.rs` only — best-effort removal of the two snapshot files after the mandatory credentials removal; Testable: `ad_delete_also_removes_snapshots` test passes confirming all three files are absent after delete.)

The snapshot removals are best-effort: if either snapshot file is absent, the delete succeeds silently. The credentials file removal is mandatory — its absence is a `NotFound` error (existing behavior unchanged).

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile_core/src/account.rs` § `delete()` — add best-effort removal of `{name}.claude.json` and `{name}.settings.json` after the mandatory credentials removal
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile_core/tests/` — add test `ad_delete_also_removes_snapshots` verifying all three files absent after delete; add test `ad_delete_succeeds_when_snapshots_absent` verifying delete still exits 0 when snapshot files were never created

## Out of Scope

- `commands.rs` — no change needed; `account_delete_routine()` delegates to `account::delete()`
- `unilang.commands.yaml` — no change needed; behavior change is internal
- CLI documentation (already updated in `docs/feature/005_account_delete.md` and `tests/docs/cli/command/06_account_delete.md`)

## Requirements

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
- The mandatory credentials file removal at step 3 of `delete()` must remain the canonical failure signal — if it fails with `NotFound` or `PermissionDenied`, the function returns that error before attempting snapshot cleanup
- Snapshot removals are best-effort: use `let _ = std::fs::remove_file(...)` to discard `NotFound` errors silently
- The fix must follow the Fix documentation format: `Fix(issue-snapshot-orphan)`, Root cause, Pitfall — in the doc comment
- No changes to public API surface of `delete()`

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note `code_design.rulebook.md` TDD workflow.
2. **Read feature doc** — Read `docs/feature/005_account_delete.md` § Design and AC-05 as source of truth.
3. **Read current implementation** — Read `claude_profile_core/src/account.rs` lines 290-303 (`delete()`) and lines 143-159 (`save()`) to understand the full file-set to clean up.
4. **Write failing test** — In `claude_profile_core/tests/`, add `ad_delete_also_removes_snapshots`: create account with all 3 files, call `delete()`, assert all 3 absent. Confirm RED.
5. **Write guard test** — Add `ad_delete_succeeds_when_snapshots_absent`: create account with only credentials file (no snapshots), call `delete()`, assert exit 0. Confirm RED (or GREEN already).
6. **Implement fix** — In `delete()`, after `std::fs::remove_file(target)?;`, add:
   ```rust
   // Fix(issue-snapshot-orphan):
   // Root cause: save() creates 3 files ({name}.credentials.json, {name}.claude.json,
   //   {name}.settings.json) but delete() only removed credentials — the two snapshot
   //   files accumulated as orphans after every deletion.
   // Pitfall: Snapshot removal must be best-effort (let _ = ...) because accounts
   //   saved before snapshot support was added have no snapshot files.
   let _ = std::fs::remove_file( credential_store.join( format!( "{name}.claude.json" ) ) );
   let _ = std::fs::remove_file( credential_store.join( format!( "{name}.settings.json" ) ) );
   ```
7. **Validate** — Run `w3 .test level::3` inside Docker. All tests must pass.
8. **Walk Validation Checklist** — check every item. Every answer must be YES.

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|----------------|-------------------|-------------------|
| `account::delete("old@archive.com", store)` | All 3 files exist | Returns Ok(()); all 3 files absent |
| `account::delete("old@archive.com", store)` | Only `.credentials.json` exists (no snapshots) | Returns Ok(()); credentials absent; no error for missing snapshots |
| `account::delete("ghost@example.com", store)` | No `.credentials.json` exists | Returns Err(NotFound); unchanged existing behavior |
| `account::delete("active@acme.com", store)` | Account is active | Returns Err(PermissionDenied); unchanged existing behavior |

## Acceptance Criteria

- After `account::delete("old@archive.com", store)` with all 3 files present: `.credentials.json`, `.claude.json`, `.settings.json` all absent from store
- After `account::delete("old@archive.com", store)` when only `.credentials.json` exists: succeeds, no error for absent snapshots
- Existing NotFound and PermissionDenied guard behaviors are unchanged
- `w3 .test level::3` passes with 0 failures and 0 clippy warnings

## Validation

### Checklist

Desired answer for every question is YES.

**Snapshot Cleanup**
- [x] After delete, is `{name}.credentials.json` absent?
- [x] After delete, is `{name}.claude.json` absent (when it existed)?
- [x] After delete, is `{name}.settings.json` absent (when it existed)?
- [x] When snapshot files were never created, does delete still return Ok(())?

**Regression Guard**
- [x] Does attempting to delete a nonexistent account still exit with NotFound?
- [x] Does attempting to delete the active account still exit with PermissionDenied?
- [x] Are other accounts' files untouched?

**Fix Documentation**
- [x] Is the `Fix(issue-snapshot-orphan)` comment in `delete()` with Root cause and Pitfall?

### Measurements

**M1 — Snapshot files absent after delete**
Command: `ls {store}/{name}.*.json 2>&1; echo "exit:$?"`
Before: 3 files listed, exit 0. After: no such file or directory, exit non-0. Deviation: any listed file.

**M2 — Best-effort: no error when snapshots absent**
Command: `account::delete()` called with only credentials file → `Result::is_ok()` → true.

### Invariants

- [x] I1 — full test suite: `w3 .test level::3` → 0 failures, 0 clippy warnings

### Anti-faking checks

**AF1 — Best-effort removals in source**
Check: `grep -c "remove_file.*claude.json\|remove_file.*settings.json" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile_core/src/account.rs`
Expected: 2. Why: confirms both snapshot removals were added, not just documented.

**AF2 — Fix comment present**
Check: `grep -c "snapshot-orphan" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile_core/src/account.rs`
Expected: 1. Why: confirms Fix documentation was added.

## Outcomes

- Extended `account::delete()` in `claude_profile_core/src/account.rs` with two best-effort `let _ = std::fs::remove_file(...)` calls after the mandatory credentials removal, targeting `{name}.claude.json` and `{name}.settings.json`.
- Added `Fix(issue-snapshot-orphan)` doc comment in `delete()` with Root cause and Pitfall fields.
- Created `claude_profile_core/tests/account_test.rs` with 5-section fix documentation and two tests: `ad_delete_also_removes_snapshots` (all 3 files present → all 3 absent after delete) and `ad_delete_succeeds_when_snapshots_absent` (credentials only → delete succeeds with no error).
- Registered `account_test.rs` in `claude_profile_core/tests/readme.md` Responsibility Table.
- Full test suite: 14/14 crates GREEN, 0 clippy warnings — `w3 .test level::3` inside Docker.
