# Add `force::` Ownership Bypass to `.account.use`, `.account.delete`, `.account.relogin`

## Execution State

- **Executor Type:** ai
- **Actor:** null
- **Claimed At:** null
- **Reopen Count:** 0
- **State:** 🎯 (Verified)
- **Closes:** null
- **Blocked Reason:** null
- **Validated By:** independent Agent subagents (4 dimensions — Scope Coherence PASS, MOST Goal PASS, Value/YAGNI PASS, Implementation Readiness PASS)
- **Validation Date:** 2026-06-16

## Goal

Register `force::` as a `bool` param (default 0) on `.account.use`, `.account.delete`, and
`.account.relogin`. When `force::1`, bypass the respective ownership gate (G5, G6, G7) and
proceed with the mutation even if `current_identity() ≠ stored_owner`.

Task is complete when ALL of the following hold:
1. `clp .account.use name::X force::1` when X is owned by a different identity exits 0 and switches the active account
2. `clp .account.delete name::X force::1` when X is owned by a different identity exits 0 and deletes the account from the credential store
3. `clp .account.relogin name::X force::1` when X is owned by a different identity exits 0 and proceeds with the 6-step relogin procedure
4. `clp .account.use name::X force::1 dry::1` exits 0; G5 bypassed; `[dry-run]` message printed; no credential file changes
5. `clp .account.delete name::X force::1 dry::1` exits 0; G6 bypassed; `[dry-run]` message printed; no files deleted
6. `clp .account.relogin name::X force::1 dry::1` exits 0; G7 bypassed; `[dry-run]` message printed; no relogin executed
7. Without `force::1`, G5–G7 continue to exit 1 with the ownership violation message (no regression)
8. ACs 18–21 in `docs/feature/036_account_ownership.md` produce the documented observable outcomes
9. All existing tests pass with no regressions; `./verb/test` exits 0
10. New test cases for `force::1` on each of the three commands exist in `tests/cli/account_mutations_test.rs`

## In Scope

**A-1 — Registry changes (`src/registry.rs`):**
- Register `force::` param (type: `bool`, default: `0`) on `.account.use`, `.account.delete`, `.account.relogin`

**A-2 — `.account.use` handler (`src/commands/account_ops.rs` `account_use_routine()`):**
- Parse `force::` bool param from `cmd`
- Modify G5 check: `if !is_owned(account) && !force` → exit 1; when `force::1`, skip the exit-1 branch and proceed to `switch_account()`

**A-3 — `.account.delete` handler (`src/commands/account_ops.rs` `account_delete_routine()`):**
- Parse `force::` bool param
- Modify G6 check: same pattern — skip exit-1 when `force::1`

**A-4 — `.account.relogin` handler (`src/commands/account_relogin.rs` `account_relogin_routine()`):**
- Parse `force::` bool param
- Modify G7 check: same pattern — skip exit-1 when `force::1`

**A-5 — Tests (`tests/cli/account_mutations_test.rs`):**
- Add `ft_force_use_bypasses_g5` — `.account.use name::X force::1` when X owned by another; expect exit 0 + account switched
- Add `ft_force_delete_bypasses_g6` — `.account.delete name::X force::1` when X owned by another; expect exit 0 + account deleted
- Add `ft_force_relogin_bypasses_g7` — `.account.relogin name::X force::1` when X owned by another; expect exit 0 + relogin proceeds
- Add `ft_force_use_dry_run` — `.account.use name::X force::1 dry::1`; expect exit 0 + `[dry-run]` + no switch
- Add `ft_force_does_not_bypass_without_flag` — `.account.use name::X` (no force::) when X owned by another; expect exit 1 (regression guard)

## Out of Scope

- `force::` on `.accounts unclaim::1` (G8 bypass) — covered by Task 001
- `force::` on `.account.save`, `.account.rotate`, `.account.renewal`, `.account.limits`, `.account.inspect` (these commands have no ownership gate to bypass)
- Changes to G1–G4 (read-side gates — `force::` does not affect fetch, refresh, or touch suppression)
- Changes to the `.accounts` or `.usage` param set (Task 001 scope)
- Changes to the `is_owned()` predicate or `current_identity()` resolution logic

## Work Procedure

1. **Red** — add failing tests in `tests/cli/account_mutations_test.rs`: `ft_force_use_bypasses_g5`, `ft_force_delete_bypasses_g6`, `ft_force_relogin_bypasses_g7`, `ft_force_use_dry_run`, `ft_force_does_not_bypass_without_flag`
2. **Registry** — add `force::` param registration to `.account.use`, `.account.delete`, `.account.relogin` in `src/registry.rs`
3. **Use handler** — in `account_use_routine()`: parse `force`; modify G5 branch to `if !is_owned(account) && !force { return Err(ownership_violation(...)) }`
4. **Delete handler** — in `account_delete_routine()`: same pattern for G6
5. **Relogin handler** — in `account_relogin_routine()`: same pattern for G7
6. **Green** — run `./verb/test`; fix any failures; confirm force::+dry:: combination works correctly for all three commands
7. **Regression** — confirm without `force::1` the existing G5/G6/G7 ownership violations still exit 1 (no regression)

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|----------------|-------------------|-------------------|
| `.account.use name::X force::1` — X owned by other identity | G5 bypassed | Exits 0; `~/.claude/.credentials.json` updated to X |
| `.account.use name::X force::1 dry::1` — X owned by other identity | G5 bypassed + dry-run | Exits 0; `[dry-run] would switch to X`; credentials unchanged |
| `.account.use name::X` — X owned by other, no force:: | G5 not bypassed | Exits 1; `"ownership violation: this account is owned by {owner}"` |
| `.account.delete name::X force::1` — X owned by other identity | G6 bypassed | Exits 0; `{name}.json` and credential files deleted |
| `.account.delete name::X force::1 dry::1` | G6 bypassed + dry-run | Exits 0; `[dry-run] would delete X`; files unchanged |
| `.account.delete name::X` — X owned by other, no force:: | G6 not bypassed | Exits 1; ownership violation message |
| `.account.relogin name::X force::1` — X owned by other identity | G7 bypassed | Exits 0; 6-step relogin executes |
| `.account.relogin name::X force::1 dry::1` | G7 bypassed + dry-run | Exits 0; `[dry-run]` message; no credential touch |
| `.account.relogin name::X` — X owned by other, no force:: | G7 not bypassed | Exits 1; ownership violation message |
| `.account.use name::X force::1` — X unowned (owner == "") | G5 passes regardless (no ownership gate fires) | Exits 0; force:: has no observable effect |
| `.account.use name::X force::1` — X owned by current identity | G5 passes regardless | Exits 0; force:: has no observable effect |

## Related Documentation

- `docs/feature/036_account_ownership.md` — ACs 18–21 define the force:: bypass contract; G5–G8 gate table
- `docs/cli/param/058_force.md` — force:: param specification
- `tests/docs/cli/command_verb/02_use.md` — use verb test spec
- `tests/docs/cli/command_verb/03_delete.md` — delete verb test spec
- `tests/docs/cli/command_verb/05_relogin.md` — relogin verb test spec
- `tests/docs/feature/36_account_ownership.md` — ownership FT spec

## History

- **2026-06-16** `CREATED` — Add force::1 bypass to .account.use, .account.delete, .account.relogin for G5–G7 ownership gates.

## Verification Record

- **Date:** 2026-06-16
- **Validators:** 2 independent Agent subagents (adversarial mandate)
- **Dimensions checked:** Scope Coherence, MOST Goal Quality, Value/YAGNI, Implementation Readiness
- **Result:** All 4 PASS
- **Notes:** Low-medium phantom risk flagged for `is_owned()` / `ownership_violation()` function name verification before executing Step 3; not a blocking defect — implementer pre-check item only.
