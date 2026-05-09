# Make `name::` optional on `.account.save` — infer from `emailAddress`

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** 🎯 (Available)

## Goal

Make `name::` optional on `clp .account.save` by reading `emailAddress` from `~/.claude.json` when the argument is absent, exiting 1 with a clear diagnostic when the field is unavailable. (Motivated: the active email is already readable from `~/.claude.json` — forcing the user to re-type it is unnecessary friction; Observable: `clp .account.save` with no arguments succeeds and prints `saved current credentials as 'EMAIL'`; Scoped: `account_save_routine()` in `commands.rs` only — no change to `.account.switch` or `.account.delete`; Testable: `cargo nextest run --test accounts_test account_save` passes all 14 IT cases including new IT-14 and updated IT-10.)

When `name::` is supplied it behaves exactly as before. When absent, the routine reads `~/.claude.json`, extracts `emailAddress` via `parse_string_field`, validates it through `account::validate_name()`, and proceeds. If the field is absent or the file unreadable, exits 1 with `cannot infer account name: emailAddress absent from ~/.claude.json — pass name:: explicitly`.

The counterexample already exists: `read_live_cred_meta()` at `commands.rs:148` reads the same field from the same path using the same `parse_string_field` function. The infrastructure is proven and co-located; only `account_save_routine()` needs to be updated.

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/src/commands.rs` § `account_save_routine()` — replace `require_nonempty_string_arg(&cmd, "name")?` with a two-branch pattern: use explicit `name::` if present and non-empty, else read `emailAddress` from `paths.claude_json_file()` via `parse_string_field`
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/tests/cli/accounts_test.rs` — update test for IT-10 (now tests absent-emailAddress path) and add test for IT-14 (infer success)

## Out of Scope

- Documentation updates (already completed)
- `.account.switch` and `.account.delete` — `name::` remains required on these commands
- `claude_profile_core/src/account.rs` — no changes needed; `validate_name()` and `parse_string_field` already correct
- `unilang.commands.yaml` — arg optionality is enforced at the routine level, not in the schema

## Requirements

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
- `name::` must remain fully functional when provided — the inference branch is only the fallback
- Inference fallback error message must match exactly: `cannot infer account name: emailAddress absent from ~/.claude.json — pass name:: explicitly`
- Use `parse_string_field(&cj, "emailAddress")` — same function and field name as used in `read_live_cred_meta()` at line 148
- The inferred name passes through `account::validate_name()` — if the stored email is invalid (theoretically impossible but defensive), exits 1 with the validation error

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note `code_design.rulebook.md` TDD workflow requirements.
2. **Read feature doc** — Read `docs/feature/002_account_save.md` § Design and § Acceptance Criteria as source of truth for expected behavior (AC-08, AC-09).
3. **Read source counterexample** — Read `src/commands.rs` lines 134-163 (`read_live_cred_meta()`) to confirm `parse_string_field(&cj, "emailAddress")` pattern and `paths.claude_json_file()` usage.
4. **Read current routine** — Read `src/commands.rs` lines 765-810 (`account_save_routine()`) to understand full flow before changing.
5. **Write failing tests** — In `tests/cli/accounts_test.rs`, update the IT-10 test (absent-email path exits 1 with correct message) and add IT-14 test (emailAddress present → saves with inferred name). Confirm RED.
6. **Implement inference** — In `account_save_routine()`, replace `let name = require_nonempty_string_arg(&cmd, "name")?;` with:
   ```rust
   let name = match get_string_arg(&cmd, "name").filter(|s| !s.is_empty()) {
     Some(n) => n,
     None => {
       let paths = require_claude_paths()?;
       let cj    = std::fs::read_to_string(paths.claude_json_file()).unwrap_or_default();
       crate::account::parse_string_field(&cj, "emailAddress")
         .filter(|s| !s.is_empty())
         .ok_or_else(|| make_error(
           "cannot infer account name: emailAddress absent from ~/.claude.json — pass name:: explicitly"
         ))?
     }
   };
   ```
   Ensure `paths` from the inference branch is reused for the rest of the routine (do not call `require_claude_paths()` again below).
7. **Validate** — Run `w3 .test level::3` inside Docker. All tests must pass.
8. **Walk Validation Checklist** — check every item. Every answer must be YES.

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|----------------|-------------------|-------------------|
| `clp .account.save name::alice@acme.com` | Explicit name provided | Saves as `alice@acme.com`; exit 0 — no change from current behavior |
| `clp .account.save` | `~/.claude.json` has `"emailAddress": "alice@acme.com"` | Infers name; saves as `alice@acme.com`; stdout `saved current credentials as 'alice@acme.com'`; exit 0 |
| `clp .account.save` | `~/.claude.json` absent | Exit 1; stderr: `cannot infer account name: emailAddress absent from ~/.claude.json — pass name:: explicitly` |
| `clp .account.save` | `~/.claude.json` present but no `emailAddress` field | Exit 1; same error message as above |
| `clp .account.save name::` | Empty explicit name | Exit 1; existing empty-name validation error unchanged |
| `clp .account.save name::notanemail` | Invalid name | Exit 1; existing `validate_name()` error unchanged |

## Acceptance Criteria

- `clp .account.save` with `emailAddress: "alice@acme.com"` in `~/.claude.json` exits 0 and creates `{credential_store}/alice@acme.com.credentials.json`
- `clp .account.save` when `~/.claude.json` has no `emailAddress` exits 1 with message `cannot infer account name: emailAddress absent from ~/.claude.json — pass name:: explicitly`
- `clp .account.save name::alice@acme.com` (explicit) behaves identically to before — no regression
- IT-10 test in `tests/cli/accounts_test.rs` tests the absent-emailAddress path and passes
- IT-14 test in `tests/cli/accounts_test.rs` tests the successful inference path and passes
- `w3 .test level::3` passes with 0 failures and 0 clippy warnings

## Validation

### Checklist

Desired answer for every question is YES.

**Name Inference**
- [ ] Does `clp .account.save` (no args) with `emailAddress` in `~/.claude.json` exit 0 and create the credential file?
- [ ] Does the stdout read `saved current credentials as 'EMAIL'` where EMAIL is the inferred address?
- [ ] Does `clp .account.save` (no args) when `emailAddress` is absent exit 1?
- [ ] Is the error message exactly `cannot infer account name: emailAddress absent from ~/.claude.json — pass name:: explicitly`?

**Regression Guard**
- [ ] Does `clp .account.save name::alice@acme.com` (explicit) still work with exit 0?
- [ ] Does `clp .account.save name::notanemail` still exit 1 with email-format error?
- [ ] Does `clp .account.save name::` still exit 1 with empty-name error?
- [ ] Are `.account.switch` and `.account.delete` without `name::` still exit 1?

**Out of Scope confirmation**
- [ ] Is `account.rs` unchanged?
- [ ] Is `unilang.commands.yaml` unchanged?

### Measurements

**M1 — Inference success**
Command: `clp .account.save 2>&1; echo "exit:$?"`
Before: `Error: Execution Error: name:: is required` / `exit:1`. Expected: `saved current credentials as 'EMAIL'` / `exit:0`. Deviation: any error or wrong exit code.

**M2 — Inference failure message**
Command: `clp .account.save 2>&1` (with `~/.claude.json` absent)
Before: generic `name:: is required`. Expected: `cannot infer account name: emailAddress absent from ~/.claude.json — pass name:: explicitly`. Deviation: wrong message text.

### Invariants

- [ ] I1 — full test suite: `w3 .test level::3` → 0 failures, 0 clippy warnings

### Anti-faking checks

**AF1 — Inference branch exists in source**
Check: `grep -c "emailAddress" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/src/commands.rs`
Expected: ≥2 (one existing in `read_live_cred_meta`, one new in `account_save_routine`). Why: confirms inference code was added, not just tests.

**AF2 — IT-14 test exists**
Check: `grep -c "IT-14\|infer\|emailAddress" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/tests/cli/accounts_test.rs`
Expected: ≥2. Why: confirms test was written, not just documented.

## Outcomes

