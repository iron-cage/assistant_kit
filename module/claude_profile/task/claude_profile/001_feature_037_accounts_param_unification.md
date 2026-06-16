# Feature 037: Absorb `.account.assign` and `.account.unclaim` into `.accounts` with `force::` Bypass

## Execution State

- **Executor Type:** ai
- **Actor:** null
- **Claimed At:** null
- **Reopen Count:** 0
- **State:** 🎯 (Verified)
- **Closes:** null
- **Blocked Reason:** null
- **Validated By:** null
- **Validation Date:** null

## Goal

**Why (null hypothesis answered):** Skipping this task leaves two standalone commands
(`.account.assign`, `.account.unclaim`) whose single-responsibility behaviors are more naturally
expressed as mutation parameters on `.accounts`/`.usage`. Keeping the status quo prevents batch
unclaim, blocks cross-machine ownership bypass, and fragments the CLI surface. Absorbing them
reduces the command count from 18 to 16, enables batch unclaim via filter predicates, adds
`force::` as an ownership escape hatch (required for admin scenarios when the owning machine is
unavailable), and achieves parameter symmetry between `.accounts` and `.usage`. These are
concrete, committed requirements — this task cannot be skipped without losing confirmed
functionality.

Implement the 32-parameter unified interface on `.accounts` and `.usage`, absorbing
`account_assign_routine()` and `account_unclaim_routine()` into the accounts handler,
adding `force::` bypass for G8 (unclaim), removing the standalone `.account.assign` and
`.account.unclaim` commands (replaced by exit-1 redirectors), and reducing the registered
command count from 18 to 16.

Task is complete when ALL of the following hold:
1. `clp .accounts unclaim::1 name::X` clears the `owner` field in `{name}.json`; exits 0
2. `clp .accounts unclaim::1 name::X force::1` clears owner even when `current_identity() ≠ stored_owner`; exits 0
3. `clp .accounts unclaim::1` (no `name::`) given 2 test accounts `acct-a` and `acct-b` both owned by `current_identity()`: exits 0; stdout contains lines `unclaimed acct-a` and `unclaimed acct-b`
4. `clp .accounts assign::1 name::X` writes the marker file `_active_{machine}_{user}` (where `{machine}` and `{user}` are the sanitized current hostname and `$USER`) in the credential store directory with content `X`; exits 0; stdout contains `Assigned X`
5. `clp .accounts assign::1 name::X for::bob@laptop` writes the marker file `_active_laptop_bob` in the credential store directory with content `X`; exits 0
6. `clp .account.unclaim name::X` exits 1; stdout contains the exact string `"unknown command '.account.unclaim' — use '.accounts unclaim::1 name::X' instead"`
7. `clp .account.assign name::X` exits 1; stdout contains the exact string `"unknown command '.account.assign' — use '.accounts assign::1 name::X' instead"`
8. Each of the 15 old field toggles (`active::1`, `current::1`, `sub::1`, `tier::1`, `expires::1`, `email::1`, `display_name::1`, `host::1`, `role::1`, `billing::1`, `model::1`, `uuid::1`, `capabilities::1`, `org_uuid::1`, `org_name::1`) exits 1 when passed to `.accounts`; stdout contains the string `cols::`
9. `clp .accounts` default text output contains an `Owner:` label with value `—` for each unowned account and `USER@MACHINE` for owned accounts
10. `clp .usage format::table` default text output contains `Owner` as a column header string
11. `clp .accounts unclaim::1 name::X` when X already has `owner: ""` exits 0 idempotently; stdout contains `unclaimed X` (double-unclaim is safe)
12. `RUSTFLAGS="-D warnings" cargo nextest run --all-features` exits 0 with no test failures and no compilation warnings
13. `test ! -f tests/cli/account_assign_test.rs` exits 0 (file deleted); `tests/cli/accounts_test.rs` exists and contains test functions with `assign` and `unclaim` in their names
14. `test ! -f src/commands/account_assign.rs` exits 0 (file deleted)
15. `grep -c "fn account_unclaim_routine" src/commands/account_ops.rs` returns `0` (function removed)
16. `clp .usage unclaim::1 name::X` clears the `owner` field in the account's `.json` file in the credential store; exits 0; stdout contains `unclaimed X`
17. `clp .usage assign::1 name::X` writes the marker file `_active_{machine}_{user}` in the credential store directory with content `X`; exits 0; stdout contains `Assigned X`
18. `clp .accounts unclaim::1 name::X dry::1` when G8 passes: exits 0; stdout contains `[dry-run] would unclaim X`; `{name}.json` owner field unchanged
19. `clp .accounts assign::1` (no `name::`) exits 0; stdout contains both the current `USER@MACHINE` identity string and the marker filename `_active_{machine}_{user}`
20. `clp .accounts assign::1 name::X dry::1` exits 0; stdout contains `[dry-run] would assign X`; the marker file `_active_{machine}_{user}` is NOT written
21. `clp .accounts unclaim::1` (no `name::`) given accounts `acct-a` owned by current identity and `acct-b` owned by a different identity: exits 1 (overall); stdout contains `unclaimed acct-a`; stdout contains `ownership violation` for `acct-b`
22. `clp .accounts cols::+host,-tier` exits 0; output includes a host column; output does not include a tier column
23. `clp .accounts unclaim::1 name::X` when `current_identity() ≠ stored_owner` and `force::` is absent: exits 1; stdout contains `ownership violation`

## In Scope

**A-1 — Registry changes:**
- Deregister `.account.assign` (Command 16) from `src/registry.rs`; register it as a redirector that exits 1 with the canonical migration error message
- Deregister `.account.unclaim` (Command 17) from `src/registry.rs`; register it as a redirector that exits 1 with the canonical migration error message
- Register `assign::`, `unclaim::`, `force::` params on `.accounts` and `.usage`
- Register the 15 former field-toggle params (`active::`, `current::`, `sub::`, `tier::`, `expires::`, `email::`, `display_name::`, `host::`, `role::`, `billing::`, `model::`, `uuid::`, `capabilities::`, `org_uuid::`, `org_name::`) on `.accounts` as explicit error-redirector params that exit 1 with a message **on stdout** containing the string `cols::` (the same redirector pattern used for the removed standalone commands); they must not be silently ignored or cause a generic unknown-param error
- Register `cols::` with default identity column set on `.accounts`

**A-2 — `.accounts` handler (`src/commands/accounts.rs`):**
- Absorb `account_assign_routine()` logic from `src/commands/account_assign.rs` as the `assign::1` dispatch branch
- Absorb `account_unclaim_routine()` logic from `src/commands/account_ops.rs` as the `unclaim::1` dispatch branch
- Add `force::` param parsing; bypass G8 ownership check when `force::1 && unclaim::1`
- Replace 15 field-toggle dispatch with `cols::` column visibility logic (identity default set: Account, Owner, Active, Current, Sub, Tier, Expires, Email)
- Add Owner column rendering from `{name}.json` owner field (`—` when empty/absent)
- Batch unclaim: when `unclaim::1` and `name::` absent, apply to all filter-matching accounts; each account is independently evaluated; if any account fails G8, the overall command exits 1; accounts that pass G8 are still unclaimed and their `unclaimed {name}` lines still printed
- No-name assign: when `assign::1` and `name::` absent, emit a live usage block showing current `USER@MACHINE` identity, the active account marker filename (`_active_{machine}_{user}`), and copy-paste examples; exit 0 without writing any file

**A-3 — `.usage` handler (`src/usage/api.rs`):**
- Add `assign::`, `unclaim::`, `for::`, `force::`, `dry::` param dispatch (same logic as accounts handler; dispatch first before quota fetch)
- Add Owner column to quota set default output (Status, Account, Owner, 5h Left, …)

**A-4 — Source file cleanup:**
- Delete `src/commands/account_assign.rs`; remove its `mod` declaration from `src/commands/mod.rs`
- Remove `account_unclaim_routine()` from `src/commands/account_ops.rs`; remove its export from `src/commands/mod.rs`

**A-5 — Test migration:**
- Create `tests/cli/accounts_test.rs` (or extend existing) with all assign and unclaim test cases
- Migrate `tests/cli/account_assign_test.rs` test cases (FT-01–FT-15, IT-01–IT-11) to `accounts_test.rs` under updated names reflecting `.accounts assign::1` syntax — these are port-then-fix tests (updated syntax compiles against the new registry from step A-1; red phase for migration tests occurs at compile time before A-2 handler is complete)
- Add new red-first tests for: `unclaim::` mutation, `force::` bypass, batch unclaim, Owner column, `cols::` syntax rejection of old toggles, and redirector exit-1 messages (written in step 1, before implementation)
- Delete `tests/cli/account_assign_test.rs` after migration
- Update `tests/cli/account_mutations_test.rs` AU section: replace `.account.unclaim` cases with `.accounts unclaim::1` syntax where applicable
- **Param spec docs** (`docs/cli/param/057_assign.md`, `docs/cli/param/058_force.md`) are already created as part of the doc update pass and exist on disk; no param doc creation needed during implementation

## Out of Scope

- Adding `force::` to `.account.use`, `.account.delete`, `.account.relogin` (G5–G7 bypass) — covered by Task 002
- Changes to `.account.save` ownership stamping behavior
- Changes to G1–G4 (fetch, refresh, touch suppression for non-owned accounts)
- New quota fetching features or changes to fetch algorithm
- `cols::` syntax design (already documented in Feature 029 / param 033); this task implements it on `.accounts`

## Work Procedure

1. **Red (new tests)** — create `tests/cli/accounts_test.rs` if absent; write failing tests for:
   - (a) [criterion 1] `unclaim::1 name::X` clears `owner` field in json to `""`; exits 0; stdout `unclaimed X`
   - (b) [criterion 2] `unclaim::1 name::X force::1` clears owner when current identity ≠ stored owner; exits 0
   - (c) [criterion 3] `unclaim::1` no-name batch, 2 accounts (`acct-a`, `acct-b`) both owned by current identity; exits 0; stdout both `unclaimed acct-a` and `unclaimed acct-b`
   - (d) [criterion 4] `assign::1 name::X` writes `_active_{machine}_{user}` in credential store; exits 0; stdout `Assigned X`
   - (e) [criterion 5] `assign::1 name::X for::bob@laptop` writes `_active_laptop_bob`; exits 0
   - (f) [criterion 20] `assign::1 name::X dry::1` prints `[dry-run] would assign X`; exits 0; no file written
   - (g) [criterion 6] `.account.unclaim name::X` exits 1; stdout exact redirector string
   - (h) [criterion 7] `.account.assign name::X` exits 1; stdout exact redirector string
   - (i) [criterion 8] all 15 old toggles (`active::1`, `current::1`, `sub::1`, `tier::1`, `expires::1`, `email::1`, `display_name::1`, `host::1`, `role::1`, `billing::1`, `model::1`, `uuid::1`, `capabilities::1`, `org_uuid::1`, `org_name::1`) each exit 1; stdout contains `cols::`
   - (j) [criterion 9] default output contains `Owner:` label; `—` for unowned accounts
   - (k) [criterion 11] double-unclaim (X already unowned) exits 0; stdout `unclaimed X`
   - (l) [criterion 16] `.usage unclaim::1 name::X` clears owner field; exits 0; stdout `unclaimed X`
   - (m) [criterion 17] `.usage assign::1 name::X` writes marker; exits 0; stdout `Assigned X`
   - (n) [criterion 18] `unclaim::1 name::X dry::1` exits 0; stdout `[dry-run] would unclaim X`; file unchanged
   - (o) [criterion 19] `assign::1` no-name exits 0; stdout contains `USER@MACHINE` and marker filename
   - (p) [criterion 21] batch unclaim mixed: `acct-a` owned by current, `acct-b` by other; exits 1; stdout both `unclaimed acct-a` and `ownership violation` for `acct-b`
   - (q) [criterion 22] `cols::+host,-tier` exits 0; host column present; tier column absent
   - (r) [criterion 23] `unclaim::1 name::X` when identity ≠ owner and no force:: exits 1; stdout `ownership violation`
2. **Red (migration tests)** — port all test functions from `tests/cli/account_assign_test.rs` to `accounts_test.rs` with updated syntax (`.accounts assign::1 name::X` instead of `.account.assign name::X`); these fail to compile until step 3 — that compile failure is the red state for migrated tests
3. **Registry** — update `src/registry.rs`: remove `.account.assign` and `.account.unclaim` as real commands; register both as redirectors (closures that exit 1 with the canonical migration message); add `assign:: bool default(0)`, `unclaim:: bool default(0)`, `force:: bool default(0)` params on `.accounts` and `.usage`; register the 15 former field-toggle params on `.accounts` as explicit error-redirectors (exit 1 with message containing `cols::`); register `cols::` with identity default set on `.accounts`
4. **Data structures** — before modifying handlers: (a) add `owner: String` field to `AccountQuota` in `src/usage/types.rs`; run `grep -rn "AccountQuota {" src/usage/fetch.rs` to find production construction sites (the struct is built in `fetch.rs`); at each production site, populate `owner` with `crate::account::read_owner(&credential_store, &name)`; for test-only fixture literals inside `#[cfg(test)]` blocks in `fetch.rs`, use `owner: String::new()` (no credential store available in unit-test context); (b) add `owner: bool` to `ColsVisibility` in `src/usage/types.rs`; the struct is only constructed via `ColsVisibility::default_set()` — update `fn default_set()` to set `owner: true` in both the identity default set (`.accounts`) and quota default set (`.usage`); update all render functions (`render_text()`, `render_tsv()`, `render_plain()`, `render_json()`) in `src/usage/render.rs` to include the Owner column/field when `cols.owner` is true
5. **Accounts handler** — update `src/commands/accounts.rs` in the main dispatch function: (a) parse `assign`, `unclaim`, `force`, `dry` bools from `cmd`; (b) `assign::1` branch: read `name`, `for`, `dry` params; reproduce the marker-write logic from `account_assign_routine()` in `src/commands/account_assign.rs` (derive `marker` filename from `for::` or current `USER`/hostname; validate account exists in store; if `dry` return `[dry-run] would assign {name} for {target}  →  {marker}`; else `std::fs::write(credential_store.join(&marker), name.as_bytes())`); (c) `unclaim::1` branch: iterate matched accounts; for each: call `crate::account::read_owner(&credential_store, &name)` then check `crate::account::is_owned(&owner)` [both confirmed to exist in `crate::account` — verified via `src/commands/account_ops.rs` lines 406–407] — skip gate if `force`; if gate fails exit 1 with `ownership violation: this account is owned by {owner}`; else call `crate::account::write_owner(&name, &credential_store, "")` [confirmed at `src/commands/account_ops.rs` line 420] and print `unclaimed {name}`; if any account fails the gate, the overall exit code is 1; (d) `cols::` rendering: replace 15 field-toggle if-arms with column set lookup from `cols` param (Owner column data already in `AccountQuota.owner` from step 4); (e) batch unclaim when `name` absent
6. **Usage handler** — update `src/usage/api.rs` `usage_routine()`: at function entry before quota fetch, check `assign`, `unclaim`, `force`, `dry` bools; if mutation path detected, dispatch to same logic as accounts handler and return early; Owner column is already in `render_text()` from step 4b
7. **Cleanup** — delete `src/commands/account_assign.rs`; remove `mod account_assign;` and `use crate::commands::account_assign::*;` from `src/commands/mod.rs`; remove `fn account_unclaim_routine` from `src/commands/account_ops.rs`; remove its re-export from `mod.rs`
8. **Test migration complete** — delete `tests/cli/account_assign_test.rs`; confirm all red tests from step 1 are now green; verify no orphaned test functions remain from deleted files
9. **Green** — run `./verb/test` (project container test runner; equivalent to `RUSTFLAGS="-D warnings" cargo nextest run --all-features`); fix compilation errors and test failures until all pass

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior | Criterion |
|----------------|-------------------|-------------------|-----------|
| `.accounts unclaim::1 name::alice` — current identity == owner | G8 gate passes | Exits 0; `owner: ""` in alice.json; stdout `unclaimed alice` | 1 |
| `.accounts unclaim::1 name::alice force::1` — current identity ≠ owner | G8 bypassed by force:: | Exits 0; `owner: ""` in alice.json; stdout `unclaimed alice` | 2 |
| `.accounts unclaim::1` — no name::, 2 accounts `acct-a`/`acct-b` both owned by current identity | Batch unclaim all-pass | Exits 0; stdout `unclaimed acct-a` and `unclaimed acct-b` | 3 |
| `.accounts assign::1 name::alice` | Marker write — current machine | Exits 0; `_active_{machine}_{user}` = alice in credential store | 4 |
| `.accounts assign::1 name::alice for::bob@laptop` | Marker write — remote machine | Exits 0; `_active_laptop_bob` = alice in credential store | 5 |
| `clp .account.unclaim name::alice` | Removed command redirector | Exits 1; stdout `"unknown command '.account.unclaim' — use '.accounts unclaim::1 name::alice' instead"` | 6 |
| `clp .account.assign name::alice` | Removed command redirector | Exits 1; stdout `"unknown command '.account.assign' — use '.accounts assign::1 name::alice' instead"` | 7 |
| `.accounts active::1` | Old field toggle rejected | Exits 1; stdout contains `cols::` | 8 |
| `.accounts` — default output | Owner column present | Exits 0; stdout contains `Owner:` label; `—` for unowned accounts | 9 |
| `.usage format::table` — default output | Owner column in .usage | Exits 0; stdout contains `Owner` as column header | 10 |
| `.accounts unclaim::1 name::alice` — alice already unowned | Idempotent double-unclaim | Exits 0; stdout `unclaimed alice`; alice.json `owner` remains `""` | 11 |
| `.accounts unclaim::1 name::alice` — current identity ≠ owner, no force:: | G8 gate blocks | Exits 1; stdout contains `ownership violation` | 23 |
| `.accounts unclaim::1 name::alice dry::1` | Dry-run with G8 passing | Exits 0; stdout `[dry-run] would unclaim alice`; alice.json unchanged | 18 |
| `.usage unclaim::1 name::alice` | Mutation on .usage | Exits 0; owner field cleared; stdout `unclaimed alice` | 16 |
| `.usage assign::1 name::alice` | Mutation on .usage | Exits 0; `_active_{machine}_{user}` = alice in credential store | 17 |
| `.accounts assign::1` — no name:: | Live usage block | Exits 0; stdout contains `USER@MACHINE` and `_active_{machine}_{user}` | 19 |
| `.accounts assign::1 name::alice dry::1` | Assign dry-run | Exits 0; stdout contains `[dry-run] would assign alice`; marker file NOT written | 20 |
| `.accounts unclaim::1` — `acct-a` owned by current, `acct-b` owned by other | Batch mixed ownership | Exits 1 overall; stdout `unclaimed acct-a`; stdout `ownership violation` for `acct-b` | 21 |
| `.accounts cols::+host,-tier` | cols:: column modifier | Exits 0; host column present; tier column absent | 22 |

## Related Documentation

- `docs/feature/037_accounts_usage_param_unification.md` — 21 ACs defining complete scope
- `docs/feature/032_account_assign.md` — assign behavior being absorbed
- `docs/feature/036_account_ownership.md` — G8 gate and force:: bypass design
- `docs/feature/003_account_list.md` — `.accounts` baseline
- `docs/feature/009_token_usage.md` — `.usage` baseline
- `docs/feature/025_per_machine_active_marker.md` — marker file format
- `docs/feature/029_account_host_metadata.md` — cols:: syntax
- `docs/cli/param/056_unclaim.md` — unclaim:: param (re-activated)
- `docs/cli/param/057_assign.md` — assign:: param (new)
- `docs/cli/param/058_force.md` — force:: param (new)
- `docs/cli/command/001_account.md` — command specification
- `tests/docs/cli/command/03_accounts.md` — accounts command test spec
- `tests/docs/cli/command/16_account_assign.md` — assign test spec (cases migrate here)
- `tests/docs/cli/command/18_account_unclaim.md` — unclaim test spec (cases migrate here)
- `tests/docs/feature/32_account_assign.md` — assign FT spec
- `tests/docs/feature/36_account_ownership.md` — ownership FT spec

## History

- **2026-06-16** `CREATED` — Implement Feature 037: absorb .account.assign and .account.unclaim into .accounts with 32-param unified set and force:: bypass.

## Verification Record

- **Date:** 2026-06-16
- **Validators:** 4 independent Agent subagents (adversarial mandate)
- **Dimensions checked:** Scope Coherence, MOST Goal Quality, Value/YAGNI, Implementation Readiness
- **Result:** All 4 PASS
- **Notes:** Reached PASS after iterative refinement across 10 MAAV rounds. Key additions that enabled PASS: (1) criterion 23 for G8-blocks negative case; (2) explicit `fn default_set()` guidance for ColsVisibility instead of struct-literal grep; (3) `AccountQuota` test-fixture handling in step 4; (4) inline [criterion N] annotations on all step-1 sub-items; (5) Null Hypothesis explicit in Why paragraph.
