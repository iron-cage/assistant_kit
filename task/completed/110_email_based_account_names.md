# Change account names to email-based identifiers in clp CLI

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** ✅ (Complete)

## Goal

Replace arbitrary string account names with email addresses as the account identifier in `clp`, so each credential snapshot is self-documenting and maps directly to the Claude account's authenticated identity (Motivated: eliminates arbitrary labels like "work"/"personal" that have no relationship to the actual credential; Observable: `validate_name("notanemail")` returns error, `validate_name("alice@acme.com")` returns Ok, `clp .account.save name::notanemail` exits 1; Scoped: `validate_name()` in `claude_profile_core/src/account.rs`, module doc examples, `print_usage()` in `lib.rs`, tests; Testable: `w3 .test level::3` passes, grep confirms email-format validation logic exists in `account.rs`).

Currently, `validate_name()` in `claude_profile_core/src/account.rs` accepts any non-empty string free of filesystem-forbidden characters (`/\:*?"<>|` and null bytes). After this change, it requires a valid email address: non-empty, containing `@` with non-empty local part and domain. Files are stored as `{credential_store}/{email}.credentials.json` — `@` is a valid filesystem character on Linux/macOS so no encoding is needed. The `print_usage()` help text must also be updated from `name::STRING` to `name::EMAIL`.

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile_core/src/account.rs` § `validate_name()` — replace filesystem-safe-string check with email format validation
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile_core/src/account.rs` — update module doc examples (`save("work")` → `save("alice@acme.com")` etc.)
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/src/lib.rs` § `print_usage()` — change `name::STRING` to `name::EMAIL` in the Commands block
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/src/lib.rs` — update module doc examples using `"work"`, `"personal"`, `"old"`
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/tests/account_tests.rs` — update all account name strings to email format; add test asserting `validate_name("notanemail")` fails with email error
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/tests/cli/account_mutations_test.rs` — update account name strings to email format
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/tests/cli/account_list_status_test.rs` — update account name strings
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/tests/cli/account_status_name_test.rs` — update account name strings
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/tests/cli/account_limits_test.rs` — update account name strings (if applicable)

## Out of Scope

- Documentation updates (completed by doc_tsk)
- POSIX flag removal (Task 109)
- Changes to `account::list()`, `account::save()`, `account::switch_account()`, `account::delete()` — these call `validate_name()` and need no other change
- URL-encoding of `@` in filenames (not needed on Linux/macOS)

## Requirements

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
- Code style: 2-space indentation, custom formatting; never use `cargo fmt`
- Tests in `tests/` directory of the crate; no inline `#[cfg(test)]` modules
- Error message from `validate_name()` must be actionable — include the term "email address"

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note `code_style.rulebook.md` formatting constraints and `test_organization.rulebook.md` test placement rules.
2. **Read `validate_name()`** — Read `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile_core/src/account.rs` fully to understand current validation logic (around line 296-313) and all callers.
3. **Read all test files** — Read `tests/account_tests.rs`, `tests/cli/account_mutations_test.rs`, `tests/cli/account_list_status_test.rs`, `tests/cli/account_status_name_test.rs` to find all account name strings that need updating.
4. **Write failing test** — Add test in `tests/account_tests.rs` asserting `validate_name("notanemail")` returns an error containing "email address". Confirm it fails before the impl change.
5. **Implement `validate_name()` change** — Replace the filesystem-forbidden-character check with email format validation: non-empty + `@` present + non-empty local part (before `@`) + non-empty domain (after `@`). Error message must include "must be an email address". Keep the non-empty check as the first guard.
6. **Update module doc examples** in `claude_profile_core/src/account.rs` — change `save("work")`, `switch_account("personal")`, `delete("old")` to email examples.
7. **Update module doc examples** in `claude_profile/src/lib.rs` — change `switch_account("personal")`, list output showing `"work"`, etc. to email examples.
8. **Update `print_usage()`** in `claude_profile/src/lib.rs` — change all three `name::STRING` occurrences (`.account.save`, `.account.switch`, `.account.delete` rows) to `name::EMAIL`.
9. **Update all tests** — Sweep `account_tests.rs` and all `cli/` test files; replace every `"work"`, `"personal"`, `"old"`, etc. account name string with email equivalents (`"alice@acme.com"`, `"alice@home.com"`, `"alice@oldco.com"`, etc.). Ensure the positive validation test (`validate_name("alice@acme.com")` → Ok) is present.
10. **Validate** — Run `w3 .test level::3` inside Docker (`run/docker .test`). All tests must pass.
11. **Walk Validation Checklist** — check every item. Every answer must be YES.

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|----------------|-------------------|-------------------|
| `validate_name("alice@acme.com")` | valid email | Returns `Ok(())` |
| `validate_name("alice@home.com")` | valid email with subdomain style | Returns `Ok(())` |
| `validate_name("")` | empty string | Returns `Err("must not be empty")` |
| `validate_name("notanemail")` | no `@` present | Returns `Err("must be an email address")` |
| `validate_name("@nodomain")` | empty local part | Returns `Err("must be an email address")` |
| `validate_name("nolocal@")` | empty domain | Returns `Err("must be an email address")` |
| `validate_name("work")` | formerly valid, now invalid | Returns `Err("must be an email address")` |
| `validate_name("personal")` | formerly valid, now invalid | Returns `Err("must be an email address")` |

## Acceptance Criteria

- `validate_name("alice@acme.com")` returns `Ok(())`
- `validate_name("notanemail")` returns `Err` containing "email address"
- `validate_name("@")` returns `Err` (empty local and domain)
- `validate_name("work")` returns `Err` (formerly accepted, now rejected)
- `print_usage()` output contains `name::EMAIL` (not `name::STRING`) for `.account.save`, `.account.switch`, `.account.delete`
- All tests in `tests/account_tests.rs` pass with email-based account names
- All tests in `tests/cli/account_mutations_test.rs` pass with email-based account names

## Validation

### Checklist

Desired answer for every question is YES.

**Email validation logic**
- [x] Does `validate_name("alice@acme.com")` return `Ok`?
- [x] Does `validate_name("notanemail")` return an error containing "email address"?
- [x] Does `validate_name("")` return an error about empty name?
- [x] Does `validate_name("@")` return an error (empty local part and domain)?
- [x] Does `validate_name("nolocal@")` return an error?
- [x] Does `validate_name("@nodomain")` return an error?

**Help output**
- [x] Does `print_usage()` show `name::EMAIL` instead of `name::STRING`?

**Test updates**
- [x] Are all uses of `"work"` as account names replaced with email strings in test files?
- [x] Are all uses of `"personal"` as account names replaced with email strings in test files?

**Out of Scope confirmation**
- [x] Are `account::list()`, `account::save()`, `account::switch_account()`, `account::delete()` signatures unchanged?
- [x] Is the POSIX flag removal (Task 109) NOT included in this task?

### Measurements

**M1 — New email validation test passes**
Command: `cargo nextest run --test account_tests 2>&1 | tail -3`
Before: new test for `validate_name("notanemail")` fails. Expected: `test result: ok. X passed`. Deviation: any FAILED line.

**M2 — Full suite passes**
Command: `w3 .test level::3` (run via `run/docker .test`)
Before: N/A (baseline is all passing before the change). Expected: 0 failures. Deviation: any failing test.

### Invariants

- [x] I1 — full test suite: `w3 .test level::3` → 0 failures

### Anti-faking checks

**AF1 — Email validation logic present**
Check: `grep -n "find.*'@'\|contains.*@\|email" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile_core/src/account.rs`
Expected: at least 2 lines (the `@` check and the error message containing "email").
Why: confirms the actual email check is in the code, not a trivially always-passing stub.

**AF2 — Old validation removed**
Check: `grep -n "filesystem-forbidden\|matches!.*'/'" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile_core/src/account.rs`
Expected: 0 lines (old character exclusion list gone).
Why: confirms the old validation was replaced, not just supplemented.

**AF3 — print_usage updated**
Check: `grep -n "name::STRING\|name::EMAIL" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/src/lib.rs`
Expected: 0 lines containing `name::STRING`; lines containing `name::EMAIL` present.
Why: confirms the help text was actually updated.

## Outcomes

Completed. All source changes applied and all tests pass under `w3 .test level::3` in Docker.

**Source changes applied:**
- `claude_profile_core/src/account.rs` — `validate_name()` replaced character-exclusion logic with email-format validation: requires `@`, non-empty local part, non-empty domain; error message updated to "must be a valid email address"
- `claude_profile/src/lib.rs` — `print_usage()` updated from `name::STRING` to `name::EMAIL` for `.account.save`, `.account.switch`, `.account.delete`
- All test fixtures in `tests/account_tests.rs` and `tests/cli/account_mutations_test.rs` updated from `"work"`/`"personal"` to `"work@acme.com"`/`"alice@acme.com"` etc.

**Validation:** `w3 .test level::3` — all crates ✅; `validate_name("notanemail")` returns error; `validate_name("alice@acme.com")` returns `Ok`
