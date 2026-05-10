# Make `.account.save` write the `_active` marker

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** ✅ (Complete)
- **Closes:** null

## Goal

Extend `account::save()` in `claude_profile_core` to write `{credential_store}/_active = {name}` as step 8 of its operation, so that `clp .credentials.status` shows `Account: {name}` immediately after `clp .account.save` without requiring a subsequent `.account.switch`. (Motivated: saving credentials means "I am this account right now" — omitting the `_active` write makes the system inconsistent: the credentials ARE saved, but the system doesn't know who is active; Observable: `_active` file contains the account name after every successful save, and `.credentials.status` shows `Account: {name}` right after save; Scoped: `account::save()` in `claude_profile_core/src/account.rs` + `ClaudePaths::with_home()` in `claude_core/src/paths.rs` for test isolation; Testable: `cargo nextest run as_save_writes_active` passes, and `cred14` passes confirming `.credentials.status` reflects the saved account name.)

`account::save()` already writes credentials + two snapshots (steps 1-7 per `docs/feature/002_account_save.md`). Step 8 is simply: `std::fs::write(credential_store.join("_active"), name)?;`. No new data structures, no email lookups, no changes to any other command.

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_core/src/paths.rs` — add `ClaudePaths::with_home(home: &Path) -> Self` constructor; required for unit test isolation (avoids `HOME` env-var mutation in parallel nextest processes)
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile_core/src/account.rs` § `save()` — add `std::fs::write(credential_store.join("_active"), name)?;` after the snapshot copies; update doc comment
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile_core/tests/account_test.rs` — add `as_save_writes_active_marker`: call `save()` via `ClaudePaths::with_home()`, assert `_active` file contains the account name
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/tests/cli/credentials_test.rs` — add `cred14_save_writes_active_shown_in_credentials_status`: run `.account.save`, then run `.credentials.status`, assert `Account: test@example.com` in output; update test matrix header
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/tests/cli/account_mutations_test.rs` — add `as16_save_writes_active_marker`: run `.account.save`, read `_active` file directly, assert it contains the saved name; update AS test matrix header

## Out of Scope

- `credentials_status_routine()` — no change; it already reads `_active` correctly
- `account::switch_account()` — no change; already writes `_active`
- `.accounts` command — no change
- Adding `email` field to `Account` struct — separate improvement, separate task
- CLI documentation (already updated in `docs/feature/002_account_save.md` and `012_live_credentials_status.md`)

## Requirements

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
- The `_active` write in `save()` must be non-best-effort: if writing `_active` fails, `save()` returns the error — the marker is not optional
- The write must occur AFTER the credential copy succeeds, not before
- Dry-run mode must remain unchanged: `_active` is not written during dry-run (dry-run check is in `account_save_routine`, before `account::save()` is called — no change needed)
- No function may exceed 50 lines; no public items without `///` doc comments

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note `code_design.rulebook.md` TDD workflow and `code_style.rulebook.md` formatting constraints.
2. **Read feature docs** — Read `docs/feature/002_account_save.md` § Design (step 8 + AC-10) and `docs/feature/007_file_topology.md` § Acceptance Criteria (AC-06) as the implementation contract.
3. **Add `with_home()` to `claude_core`** — In `claude_core/src/paths.rs`, add constructor immediately after `new()`:
   ```rust
   /// Create a `ClaudePaths` from an explicit home directory path.
   ///
   /// Used in tests to avoid `HOME` env-var mutation across parallel processes.
   #[ inline ]
   #[ must_use ]
   pub fn with_home( home : &std::path::Path ) -> Self
   {
     Self { base : home.join( ".claude" ) }
   }
   ```
4. **Read source** — Read `claude_profile_core/src/account.rs` `save()` (lines 143-160) to confirm the insertion point after the snapshot copies.
5. **Write failing tests** —
   a. In `claude_profile_core/tests/account_test.rs`, add `as_save_writes_active_marker` using `ClaudePaths::with_home(tmp.path())`: call `account::save("alice@acme.com", store, &paths)`, assert `{store}/_active` exists and contains `"alice@acme.com"`. Update module doc header. Confirm RED.
   b. In `claude_profile/tests/cli/credentials_test.rs`, add `cred14_save_writes_active_shown_in_credentials_status`: call `write_credentials`, then `run_cs_with_env(".account.save name::test@example.com")`, then `run_cs_with_env(".credentials.status")`, assert exit 0 and stdout contains `Account: test@example.com`. Update test matrix header row. Confirm RED on the core test (unit test fails; cred14 will also fail once binary is rebuilt).
   c. In `claude_profile/tests/cli/account_mutations_test.rs`, add `as16_save_writes_active_marker`: call `write_credentials`, run `".account.save name::work@acme.com"`, read `{store}/_active` directly, assert it equals `"work@acme.com"`. Update AS test matrix header row.
6. **Implement** — In `account::save()`, add after the two best-effort snapshot copies and update doc comment:
   ```rust
   // Mark this account as active so credentials_status_routine shows Account: {name}.
   std::fs::write( credential_store.join( "_active" ), name )?;
   ```
7. **Green state** — `w3 .test level::3` inside Docker. All tests must pass with 0 failures and 0 clippy warnings.
8. **Walk Validation Checklist** — check every item. Every answer must be YES.
9. **Update task status** — set ✅ in `task/readme.md`, move file to `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `account::save("alice@acme.com", store, paths)` called with valid credentials | `_active` write in `save()` | `{store}/_active` exists and contains `"alice@acme.com"` |
| T02 | `account::save` called twice with different names | `_active` overwrite | `_active` contains the second name |
| T03 | `.credentials.status` called after `.account.save` | `_active` written by save; read by credentials_status_routine | stdout contains `Account: test@example.com`; exit 0 |
| T04 | `.account.save dry::1` | dry-run path — `save()` not called | `_active` NOT written; output is dry-run message |
| T05 | `.account.save name::work@acme.com` via CLI | CLI invokes `save()`; `as16` reads `_active` directly | `{store}/_active` contains `"work@acme.com"`; exit 0 |

## Acceptance Criteria

- `ClaudePaths::with_home(home)` exists in `claude_core/src/paths.rs` and returns `ClaudePaths` whose `credentials_file()` resolves to `{home}/.claude/.credentials.json`
- `account::save()` in `claude_profile_core/src/account.rs` calls `std::fs::write(credential_store.join("_active"), name)` with `?` propagation, after the credential copy and snapshot copies
- `as_save_writes_active_marker` passes: `_active` file contains the account name after `save()`
- `cred14_save_writes_active_shown_in_credentials_status` passes: `.credentials.status` shows `Account: test@example.com` after `.account.save`
- `as16_save_writes_active_marker` passes: `{store}/_active` contains `"work@acme.com"` after `.account.save name::work@acme.com`
- Dry-run (`dry::1`) does NOT write `_active` (existing `account_save_routine` dry-run path is unchanged)
- `w3 .test level::3` inside Docker passes with 0 failures and 0 clippy warnings

## Validation

### Checklist

Desired answer for every question is YES.

**`ClaudePaths::with_home()` prerequisite**
- [ ] C0 — Does `ClaudePaths::with_home(home)` exist in `claude_core/src/paths.rs`?

**`account::save()` writes `_active`**
- [ ] C1 — Does `save()` call `std::fs::write(credential_store.join("_active"), name)` with `?` propagation?
- [ ] C2 — Is the `_active` write placed AFTER the credential copy and snapshot copies?
- [ ] C3 — Does `as_save_writes_active_marker` pass: `_active` contains the saved account name?
- [ ] C4 — Does `cred14` pass: `.credentials.status` shows `Account: test@example.com` after save?
- [ ] C4b — Does `as16` pass: `{store}/_active` contains `"work@acme.com"` after `.account.save`?

**Dry-run unchanged**
- [ ] C5 — Does `dry::1` NOT write `_active`? (dry-run check is before `save()` call in `account_save_routine`)

**Regression guard**
- [ ] C6 — Do all existing `account_test.rs` tests still pass?
- [ ] C7 — Do all existing `credentials_test.rs` tests (cred01–cred13) still pass?
- [ ] C8 — Does `account::switch_account()` remain unchanged?

**Out of Scope confirmation**
- [ ] C9 — Is `credentials_status_routine()` unchanged?
- [ ] C10 — Is the `Account` struct unchanged (no new fields added in this task)?

### Measurements

- [ ] M1 — `as_save_writes_active` GREEN: `cargo nextest run -p claude_profile_core as_save_writes 2>&1 | tail -3` → `1 passed`; was: test absent.
- [ ] M2 — `cred14` GREEN: `cargo nextest run --test cli_integration_test cred14 2>&1 | tail -3` → `1 passed`; was: test absent.
- [ ] M3 — Full suite GREEN: `w3 .test level::3` → 0 failures; was: same (no regressions).

### Invariants

- [ ] I1 — test suite: `w3 .test level::3` → 0 failures, 0 clippy warnings
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check --all-features` → 0 warnings

### Anti-faking checks

- [ ] AF1 — `_active` write in save: `grep -c 'join.*"_active"' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile_core/src/account.rs` → `≥2` (one in `switch_account`, one new in `save`). Why: confirms the write was added to `save()`, not just to `switch_account`.
- [ ] AF2 — non-best-effort: `grep -A1 'join.*"_active"' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile_core/src/account.rs` → second occurrence ends with `?;` not `let _ =`. Why: confirms `_active` write in `save()` propagates errors rather than silently discarding them.
- [ ] AF3 — test asserts account name: `grep -c 'Account: test@example.com' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/tests/cli/credentials_test.rs` → `≥1`. Why: confirms cred14 asserts the resolved name, not just exit 0.
- [ ] AF4 — `with_home` exists: `grep -c 'fn with_home' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_core/src/paths.rs` → `1`. Why: confirms the constructor was added, enabling isolated unit tests.
- [ ] AF5 — `as16` exists: `grep -c 'as16_save_writes_active_marker' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/tests/cli/account_mutations_test.rs` → `≥1`. Why: confirms the CLI-level symmetry test was added alongside aw07.

## Outcomes

- Added `ClaudePaths::with_home(home: &Path) -> Self` to `claude_core/src/paths.rs` for test isolation.
- Added `std::fs::write(credential_store.join("_active"), name)?;` to `account::save()` in `claude_profile_core/src/account.rs` after both snapshot copies, with `?` propagation (non-best-effort).
- Added unit test `as_save_writes_active_marker` in `claude_profile_core/tests/account_test.rs`.
- Added CLI test `cred14_save_writes_active_shown_in_credentials_status` in `claude_profile/tests/cli/credentials_test.rs`.
- Added CLI test `as16_save_writes_active_marker` in `claude_profile/tests/cli/account_mutations_test.rs`.
- Fixed regression in `claude_profile/tests/account_tests.rs::delete_removes_credential_file` — after `save()` now writes `_active = "alice@oldco.com"`, added overwrite to `"work@acme.com"` so deletion is permitted.
- `w3 .test level::3`: 14/14 crates passed, 0 failures, 0 clippy warnings.
