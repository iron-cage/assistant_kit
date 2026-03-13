# TSK-065: TDD implementation of FR-16 — `.account.status name::` parameter

## Goal

Implement FR-16 following strict TDD (Red → Green → Refactor):
extend `account_status_routine` in `src/commands.rs` to accept an optional `name::`
parameter, and register `name::` in `unilang.commands.yaml`.

## Motivation

FR-16 (defined in TSK-063 / spec.md) allows any saved account to be inspected without
switching to it. At `v::1`+, the active account additionally shows email and organization
from `~/.claude/.claude.json`.

## In Scope

- `module/claude_profile/unilang.commands.yaml`:
  - Add `name::` argument (optional, default `""`) to `.account.status` command
- `module/claude_profile/src/commands.rs`:
  - Extend `account_status_routine` per Algorithm A1 in plan 002
  - Add private helper `token_status_from_ms` per Algorithm A2
- `module/claude_profile/tests/integration/`:
  - New file `account_status_name_test.rs` covering the full test matrix
  - Register in `tests/integration/readme.md`

## Out of Scope

- Changes to `claude_common` (no `ClaudePaths::claude_json()` method needed)
- `scopes[]` or `loginMethod` fields (deferred per plan 002 Appendix)
- CLI documentation updates (covered in TSK-066)

## Test Matrix (RED phase)

| Scenario | `name::` | Active | Expected |
|----------|----------|--------|----------|
| No name, active set | (omitted) | `work` | Shows `work` status (backward compat) |
| No name, no active | (omitted) | (none) | Exit error: "no active account set" |
| name = active account | `name::work` | `work` | Shows `work` status |
| name = non-active account | `name::personal` | `work` | Personal's own expiry; N/A email |
| name = nonexistent | `name::ghost` | `work` | Exit error: account not found |
| name = empty string | `name::` | `work` | Exit error: name must not be empty |
| v::0 with name:: | `name::personal v::0` | `work` | Two bare lines: name, token state |
| v::1 with name:: (active) | `name::work v::1` | `work` | Sub, tier, email, org shown |
| v::1 with name:: (non-active) | `name::personal v::1` | `work` | Sub, tier, N/A email/org |
| v::2 with name:: (non-active) | `name::personal v::2` | `work` | Full metadata, N/A email/org, personal expiry |
| format::json with name:: | `name::work format::json` | `work` | JSON with `account` and `token` fields |

## Work Procedure

**Phase RED — write failing tests:**

1. Create `tests/integration/account_status_name_test.rs`
   - Each test acquires `ENV_MUTEX`, creates temp `HOME`, writes fixture files
   - Invokes CLI binary, asserts exit code and output
2. Add row to `tests/integration/readme.md`
3. Confirm tests fail: `w3 .test level::1` shows failures in new file

**Phase GREEN — implement:**

4. `unilang.commands.yaml`: add `name::` argument to `.account.status` block
   (optional, kind: String, default: "", before `verbosity` argument)
5. `src/commands.rs`: extend `account_status_routine` per Algorithm A1:
   - Extract `name::` arg (Value::String); treat empty as absent
   - If name provided: validate via `crate::account::validate_name()`; construct
     `accounts/{name}.credentials.json` path; return error if not found
   - Compute `active_name` from `_active` marker
   - For non-active named account: compute token status via `token_status_from_ms()`
     (reads `expiresAt` from account file, not live credentials)
   - For active account: use existing `crate::token::status_with_threshold()`
   - At v::1+: read sub/tier from credential content
   - At v::1+, active only: read email/org from `~/.claude/.claude.json`
     (graceful N/A if file absent)
6. Add private `fn token_status_from_ms(expires_at_ms: u64) -> crate::token::TokenStatus`
   per Algorithm A2

**Phase REFACTOR:**

7. Apply 2-space indents throughout (no cargo fmt)
8. Keep `account_status_routine` under 80 lines; extract helpers if needed
9. Run `cargo clippy --all-targets --all-features -- -D warnings` → zero warnings

**Verify:**

10. Run `w3 .test level::3` — all tests pass, zero warnings

## Validation List

Desired answer for every question is YES.

- [ ] Does `unilang.commands.yaml` `.account.status` block have `name::` with `optional: true`?
- [ ] Does `account_status_routine` handle missing `name::` identically to current behavior?
- [ ] Does a non-active named account show its OWN `expiresAt`, not the active account's?
- [ ] Does email/org show `N/A` for non-active named account?
- [ ] Does email/org read from `~/.claude/.claude.json` for active account?
- [ ] Does `name::` = nonexistent account → error with "not found" message?
- [ ] Does `name::` = empty string → error?
- [ ] Do all 10 existing IT-1..IT-10 tests still pass?
- [ ] Is the test matrix fully covered by new tests in `account_status_name_test.rs`?
- [ ] Does `w3 .test level::3` pass with zero failures and zero warnings?

## Validation Procedure

### Measurements

**M1 — All prior IT tests still pass**
```bash
cd /home/user1/pro/lib/wip_core/claude_tools/dev && RUSTFLAGS="-D warnings" cargo nextest run --all-features -p claude_profile -E "test(IT)" 2>&1 | grep -E "PASS|FAIL|passed|failed"
```
Before: all IT-1..IT-10 pass. Expected: same, unchanged.

**M2 — New name:: tests pass**
```bash
cd /home/user1/pro/lib/wip_core/claude_tools/dev && RUSTFLAGS="-D warnings" cargo nextest run --all-features -p claude_profile -E "test(account_status_name)" 2>&1 | tail -5
```
Before: file does not exist. Expected: all new tests pass.

**M3 — Full level 3 clean**
```bash
cd /home/user1/pro/lib/wip_core/claude_tools/dev && w3 .test level::3 2>&1 | tail -20
```
Expected: nextest ✓, doc tests ✓, clippy ✓ — zero failures, zero warnings.

**M4 — name:: registered in YAML**
```bash
grep -A 40 '"\.account\.status"' module/claude_profile/unilang.commands.yaml | grep -c "name"
```
Before: 0. Expected: ≥1.

### Anti-faking checks

**AF1 — Non-active account shows own expiry, not active's**
Test: two accounts with different `expiresAt` values. Query the non-active one.
Assert output shows expiry derived from non-active account's `expiresAt`, not live credentials.

**AF2 — Backward compat: no-name error message preserved**
```bash
grep "no active account set" module/claude_profile/src/commands.rs
```
Expected: message still present, unchanged.

## Outcomes

**Completed:** 2026-03-31
**Result:** Done — implemented FR-16 via TDD with 10 passing integration tests; added `token_status_from_ms`, `status_active`, `status_named` helpers in `commands.rs`; registered `name::` in `src/lib.rs` and `unilang.commands.yaml`; all 20 IT tests pass at level 3.
**Files changed:** `module/claude_profile/src/commands.rs`, `module/claude_profile/src/lib.rs`, `module/claude_profile/unilang.commands.yaml`, `module/claude_profile/tests/cli_integration_test.rs`, `module/claude_profile/tests/integration/account_status_name_test.rs`, `module/claude_profile/tests/integration/helpers.rs`, `module/claude_profile/tests/integration/readme.md`, `module/claude_profile/spec.md` (conformance ✅)
