# Relocate credential store to `$PRO/.persistent/claude/credential/`

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** ✅ (Complete)

## Goal

Move the account credential snapshot store from `~/.claude/accounts/` to `$PRO/.persistent/claude/credential/` (with `$HOME/.persistent/claude/credential/` fallback), eliminating the hard-coded HOME-relative path in favor of the `$PRO`/`$HOME` resolution chain already established by `PersistPaths` (Motivated: credential snapshots are machine-migratable committed data — the `.persistent/` storage tier is the correct home, and `$PRO` points to the workspace root that survives machine migration; Observable: `clp .account.save name::alice@acme.com` creates the file under `$PRO/.persistent/claude/credential/`, `~/.claude/accounts/` is never created; Scoped: `ClaudePaths::accounts_dir()` removed from `paths.rs`, account functions in `account.rs` changed to accept `credential_store: &Path`, `PersistPaths::credential_store()` added to `persist.rs`, `commands.rs` updated to resolve and pass the credential store, all test fixtures updated; Testable: `w3 .test level::3` passes, grep confirms no `accounts_dir` symbol in code, `$PRO/.persistent/claude/credential/alice@acme.com.credentials.json` created by save tests).

## In Scope

### Source Changes

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile_core/src/paths.rs` — remove `accounts_dir()` method from `ClaudePaths`
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile_core/src/account.rs` — change all account functions to accept `credential_store: &Path` instead of calling `paths.accounts_dir()`; update doc comments from `~/.claude/accounts/` to `{credential_store}`
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/src/persist.rs` — add `PersistPaths::credential_store()` method returning `{root}/.persistent/claude/credential/`; change `base()` to use `.persistent/claude_profile/` (add missing dot prefix)
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/src/commands.rs` — update all `require_nonempty_string_arg`/account operation call sites to resolve `PersistPaths::credential_store()` and pass it as `credential_store` parameter
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/src/lib.rs` — update module doc example paths

### Test Changes

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/tests/account_tests.rs` — update all `paths.accounts_dir()` calls and fixture directories to use credential store paths; update `save_creates_accounts_dir_when_missing` to assert `{credential_store}` is created
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/tests/cli/persist_test.rs` — update `base()` assertions from `persistent/claude_profile/` to `.persistent/claude_profile/`; add tests `p16`–`p18` for `credential_store()` method
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/tests/cli/account_mutations_test.rs` — update expected file paths
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/tests/cli/account_list_status_test.rs` — update expected file paths
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/tests/cli/account_status_name_test.rs` — update expected file paths
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/tests/paths_tests.rs` — remove `accounts_dir()` test; verify remaining path methods

## Out of Scope

- Documentation updates (completed: docs/feature/ 001-005, 007, 010-012; docs/cli/ all files; docs/invariant/003)
- Email-based account names (Task 110)
- POSIX flag removal (Task 109)
- Changes to `claude_storage_core`, `claude_runner_core`, or any other crate
- Migration of existing credential snapshots from `~/.claude/accounts/` to new location (user responsibility)

## Requirements

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
- Code style: 2-space indentation, custom formatting; never use `cargo fmt`
- Tests in `tests/` directory of the crate; no inline `#[cfg(test)]` modules
- No backward compatibility: `accounts_dir()` is deleted, not deprecated
- `PersistPaths::credential_store()` must use the same `$PRO`-is-dir guard as `PersistPaths::new()` — never use `exists()` where `is_dir()` is required

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note `code_style.rulebook.md` formatting constraints and `test_organization.rulebook.md` test placement rules.
2. **Read account.rs fully** — understand all callers of `paths.accounts_dir()` (functions: `list`, `save`, `switch_account`, `delete`, `status_by_name`).
3. **Read paths.rs fully** — identify `accounts_dir()` implementation and all references.
4. **Read persist.rs fully** — understand current `base()` implementation to mirror its pattern for `credential_store()`.
5. **Read commands.rs fully** — identify how account operation call sites pass `paths` today.
6. **Write failing tests** — Add `p16`–`p18` in `persist_test.rs` asserting `credential_store()` returns correct paths under `$PRO` and `$HOME`. Add test asserting `save` creates files under `{credential_store}`, not `~/.claude/accounts/`. Confirm these fail before implementation.
7. **Update `persist.rs`** — Add `PersistPaths::credential_store()` returning `{root}/.persistent/claude/credential/`. Fix `base()` to use `.persistent/claude_profile/` (add dot). Ensure `resolve_root()` logic is shared (not duplicated).
8. **Update `account.rs`** — Change function signatures for `list`, `save`, `switch_account`, `delete`, `status_by_name` to accept `credential_store: &Path` instead of using `paths.accounts_dir()`. Update all internal usages and doc comments.
9. **Remove `accounts_dir()` from `paths.rs`** — Delete the method completely.
10. **Update `commands.rs`** — At each account operation call site, resolve `PersistPaths::credential_store()?` and pass it as `credential_store` argument.
11. **Update tests** — Fix all test fixtures: replace `paths.accounts_dir()` with resolved credential store paths. Update `persist_test.rs` expected values.
12. **Validate** — Run `w3 .test level::3` inside Docker (`run/docker .test`). All tests must pass.
13. **Walk Validation Checklist** — check every item. Every answer must be YES.

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|----------------|-------------------|-------------------|
| `$PRO` set to existing dir | `credential_store()` | Returns `$PRO/.persistent/claude/credential/` |
| `$PRO` set to a file path | `credential_store()` | Falls back to `$HOME/.persistent/claude/credential/` |
| `$PRO` unset | `credential_store()` | Returns `$HOME/.persistent/claude/credential/` |
| `account::save("alice@acme.com", store, paths)` | `save()` | Creates `{store}/alice@acme.com.credentials.json` |
| `account::list(store)` | `list()` | Enumerates `{store}/*.credentials.json` |
| `account::switch_account("alice@home.com", store, paths)` | `switch_account()` | Reads `{store}/alice@home.com.credentials.json` |
| `account::delete("alice@oldco.com", store)` | `delete()` | Removes `{store}/alice@oldco.com.credentials.json` |
| `clp .account.save name::alice@acme.com` | end-to-end | Creates file under `$PRO/.persistent/claude/credential/` |
| `accounts_dir()` called | `ClaudePaths` | Compile error — method removed |

## Acceptance Criteria

- `PersistPaths::credential_store()` returns `$PRO/.persistent/claude/credential/` when `$PRO` is a dir
- `PersistPaths::base()` returns `$PRO/.persistent/claude_profile/` (dot prefix fixed)
- `ClaudePaths::accounts_dir()` does not exist (deleted)
- `account::save` stores files under `{credential_store}`, not `~/.claude/accounts/`
- All tests in `tests/account_tests.rs` pass with new paths
- All tests in `tests/cli/persist_test.rs` pass with updated expected values
- `grep -rn "accounts_dir" src/ tests/` returns 0 matches

## Validation

### Checklist

Desired answer for every question is YES.

**Path resolution**
- [x] Does `PersistPaths::credential_store()` return `$PRO/.persistent/claude/credential/` when `$PRO` is set and is a dir?
- [x] Does `credential_store()` fall back to `$HOME/.persistent/claude/credential/` when `$PRO` is unset?
- [x] Does `PersistPaths::base()` now return `{root}/.persistent/claude_profile/` (with dot)?

**`ClaudePaths` cleanup**
- [x] Is `accounts_dir()` removed from `ClaudePaths`?
- [x] Does the code compile without `accounts_dir()` referenced anywhere?

**Account operations**
- [x] Do `list`, `save`, `switch_account`, `delete`, `status_by_name` accept `credential_store: &Path`?
- [x] Does `commands.rs` resolve `PersistPaths::credential_store()` at each call site?

**Test updates**
- [x] Do `persist_test.rs` assertions match new `.persistent/` dot-prefix paths?
- [x] Do all account test fixtures use the credential store path?

**Out of Scope confirmation**
- [x] Are email-based validation changes (Task 110) NOT included?
- [x] Are POSIX flag removals (Task 109) NOT included?

### Measurements

**M1 — New `credential_store()` tests pass**
Command: `cargo nextest run --test cli/persist_test 2>&1 | tail -3`
Before: p16-p18 fail. Expected: `test result: ok. X passed`. Deviation: any FAILED line.

**M2 — No `accounts_dir` references remain**
Command: `grep -rn "accounts_dir" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile_core/src/ /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/src/`
Expected: 0 lines. Deviation: any match.

**M3 — Full suite passes**
Command: `w3 .test level::3` (run via `run/docker .test`)
Expected: 0 failures. Deviation: any failing test.

### Invariants

- [ ] I1 — full test suite: `w3 .test level::3` → 0 failures

### Anti-faking checks

**AF1 — `accounts_dir` removed**
Check: `grep -rn "accounts_dir" /home/user1/pro/lib/wip_core/claude_tools/dev/module/`
Expected: 0 lines.
Why: confirms `accounts_dir()` was deleted, not just unused.

**AF2 — credential_store method added**
Check: `grep -n "credential_store" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/src/persist.rs`
Expected: at least 2 lines (method definition + return value).
Why: confirms the actual method exists, not a stub.

**AF3 — dot prefix in persist.rs base path**
Check: `grep -n '\.persistent' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/src/persist.rs`
Expected: lines showing `.persistent` (with dot).
Why: confirms the base path was fixed, not just credential_store added.

**AF4 — credential store used in account test fixtures**
Check: `grep -n "persistent/claude/credential" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/tests/account_tests.rs`
Expected: at least 1 line showing the new path in test fixtures.
Why: confirms tests were updated, not just the production code.

## Outcomes

**Completed 2026-05-04.** All acceptance criteria met; `w3 .test level::3` passes across 14/14 crates (56/56 jobs, 0 failures).

### Changes delivered

**`claude_profile_core/src/paths.rs`** — `accounts_dir()` removed completely from `ClaudePaths`.

**`claude_profile_core/src/account.rs`** — `list`, `save`, `switch_account`, `delete`, `auto_rotate`, `status_by_name` all accept `credential_store: &Path`; zero internal `accounts_dir()` calls remain.

**`claude_profile/src/persist.rs`** — `PersistPaths::credential_store()` added (returns `{root}/.persistent/claude/credential/`); `base()` dot-prefix fixed (`persistent/` → `.persistent/`).

**`claude_profile/src/commands.rs`** — `require_credential_store()` helper added; every account operation call site passes the resolved credential store path.

**All test fixtures updated** — `tests/account_tests.rs`, `tests/cli/persist_test.rs` (p04, p16-p18), `tests/cli/account_mutations_test.rs`, `tests/cli/account_list_status_test.rs`, `tests/cli/account_status_name_test.rs`, `tests/cli/token_paths_test.rs`, `tests/cli/account_limits_test.rs`, `tests/cli/cross_cutting_test.rs`, `tests/cli/helpers.rs`, `tests/paths_tests.rs`.

**`claude_version/tests/integration/helpers.rs`** — `write_account` updated to write under `.persistent/claude/credential/`.

**`tests/manual/readme.md`** — step 7 and IT-4 updated to use email-format account names (consistent with TSK-110).

### M2 verification (AF1)

`grep -rn "accounts_dir" module/` returns only matches inside `-plan/` (hyphenated temp files, not committed).
