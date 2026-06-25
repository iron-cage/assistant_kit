# FT — Feature 037: Accounts/Usage Parameter Set Unification

### Scope

- **Purpose**: Test cases for the unified parameter interface shared by `.accounts` and `.usage`, including `assignee::USER@MACHINE` and `owner::0`/`owner::USER@MACHINE` mutation params (Feature 065/064; formerly `assign::1`/`unclaim::1`), REMOVED_TOGGLE stubs for `assign::`, `unclaim::`, `for::`, `active::`, `force::` G8 bypass, `cols::` default sets, Owner column, and standalone command removal.
- **Source**: `docs/feature/037_accounts_usage_param_unification.md`
- **Covers**: AC-01 through AC-24

### Test Cases

| FT | AC | Scenario | Source fn |
|----|----|----------|-----------|
| FT-01 | AC-01 | `.accounts` accepts all 32 unified params; unknown param exits 1 | `ft01_accounts_accepts_32_params` |
| FT-02 | AC-02 | `.usage` accepts all 32 unified params; unknown param exits 1 | `f37_ft02_usage_accepts_32_params` (`usage_feature_test.rs`) |
| FT-03 | AC-03 | `.accounts` defaults: `refresh::0`, `touch::0`, `sort::name`, `cols::` = identity set; no HTTP fetch or subprocess without explicit flags | `ft03_accounts_default_profile` |
| FT-04 | AC-04 | `.usage` defaults: `refresh::1`, `touch::1`, `sort::renew`, `cols::` = quota set with Owner column | `f37_ft04_usage_default_profile` (`usage_feature_test.rs`) |
| FT-05 | AC-05 | `.accounts owner::0 name::X` exits 0; writes `owner: ""`; credentials and active marker unchanged (Feature 064; formerly `unclaim::1 name::X`) | `it01_unclaim_clears_owner (account_mutations_test.rs)` |
| FT-06 | AC-06 | `.accounts owner::0 name::X` exits 1 with ownership violation when G8 fails; gate runs before `dry::1` (Feature 064; formerly `unclaim::1`) | `ft16_unclaim_g8_gate (account_mutations_test.rs)` |
| FT-07 | AC-07 | `.accounts owner::0` (no `name::`) applies ownership release to all filtered accounts; each evaluated against G8; non-owned skipped (Feature 064; formerly `unclaim::1` batch) | `ft07_accounts_unclaim_batch` |
| FT-08 | AC-08 | `.accounts assignee::user1@w003 name::X` writes marker file; `{name}.json`, credentials, `~/.claude.json` unchanged (Feature 065; formerly `assign::1 name::X`) | `ft01_assignee_assign_writes_current_machine_marker (account_assign_test.rs)` |
| FT-09 | AC-09 | `.accounts assignee::bob@laptop name::X` writes `_active_laptop_bob`; sanitization identical to former `.account.assign` (Feature 065; formerly `assign::1 name::X for::bob@laptop`) | `ft01b_assignee_assign_writes_remote_marker (account_assign_test.rs)` |
| FT-10 | AC-10 | `.accounts assignee::user1@w003` (no `name::`) clears `_active_w003_user1`; exits 0; no credentials or `{name}.json` touched (Feature 065; replaces former `assign::1` no-name usage block) | `ft02_assignee_unassign_clears_marker (account_assign_test.rs)` |
| FT-11 | AC-11 | `.account.unclaim name::alice` exits 1 with targeted `owner::0` migration hint — registered as redirect stub (Feature 037) | `ft11_account_unclaim_fully_deregistered` (`accounts_test.rs`) |
| FT-12 | AC-12 | `.account.assign name::alice` exits 1 with targeted `assignee::` migration hint — registered as redirect stub (Feature 037) | `ft12_account_assign_fully_deregistered` (`accounts_test.rs`) |
| FT-13 | AC-13 | `.accounts` rejects all 15 legacy field toggles (`active::`, `current::`, `sub::`, `tier::`, `expires::`, `email::`, `display_name::`, `host::`, `role::`, `billing::`, `model::`, `uuid::`, `capabilities::`, `org_uuid::`, `org_name::`); each exits 1 directing to `cols::` | `ft13_accounts_legacy_toggles_rejected` |
| FT-14 | AC-14 | `.accounts cols::+host,-tier` adds host column and removes tier from identity default set | `ft14_accounts_cols_modifier` |
| FT-15 | AC-15 | `.accounts refresh::1` fetches live quota; `.accounts touch::1` activates idle sessions — same algorithm as `.usage` | `lim_it_ft15_accounts_refresh_live (accounts_test.rs)` |
| FT-16 | AC-16 | `.usage owner::0 name::X` clears owner field — identical result to `.accounts owner::0 name::X` (Feature 064; formerly `usage unclaim::1 name::X`) | `f37_ft16_usage_unclaim_mirrors_accounts` (`usage_feature_test.rs`) |
| FT-17 | AC-17 | `.usage assignee::user1@w003 name::X` writes marker — identical result to `.accounts assignee::user1@w003 name::X` (Feature 065; formerly `active::USER@MACHINE name::X` — Feature 064) | `f37_ft17_usage_assign_mirrors_accounts` (`usage_feature_test.rs`) |
| FT-18 | AC-18 | `.accounts dry::1 owner::0 name::X` prints `[dry-run] would clear owner of X`; exits 0; no files modified; G8 gate runs (Feature 064; formerly `dry::1 unclaim::1`) | `ft17_unclaim_dry_run (account_mutations_test.rs)` |
| FT-19 | AC-19 | Owner column visible by default on `.accounts` and `.usage`; shows owner from `{name}.json`; `cols::-owner` hides it | `ft19_owner_column_default_visible` |
| FT-20 | AC-20 | `.accounts owner::0 name::X force::1` bypasses G8; clears owner even when caller ≠ stored owner; exits 0 (Feature 064; formerly `unclaim::1 force::1`) | `ft20_accounts_unclaim_force_bypasses_g8` |
| FT-21 | AC-21 | `.accounts force::1` without `owner::`, and `.accounts force::1 assignee::user1@w003 name::X`, silently ignore `force::1` — no error (Feature 065; formerly with `assign::1`/`active::`) | `ft21_force_no_effect_without_unclaim` |
| FT-22 | AC-22 | `.accounts assign::1` exits 1 with migration message "REMOVED — use `assignee::USER@MACHINE name::X`"; `.accounts unclaim::1` exits 1 "REMOVED — use `owner::0 name::X`"; `.accounts for::user@host` exits 1 with migration message; `.accounts active::user@host` exits 1 with REMOVED_TOGGLE pointing to `assignee::` (Feature 065) | `ft22_removed_toggle_stubs` |
| FT-23 | AC-23 | `.accounts owner::user1@w003 name::X,Y,Z` sets ownership for X, Y, Z in one invocation; each G8-evaluated independently | `ft23_owner_batch_set` |
| FT-24 | AC-24 | `.accounts owner::0 name::X,Y,Z` clears ownership for X, Y, Z; each G8-evaluated independently | `ft24_owner_batch_clear` |

### Notes

- FT-01 and FT-02 are structural registration tests: run `.accounts` and `.usage` with each of the 32 params set to a valid value and verify exit 0; then try an unknown param and verify exit 1 with error message.
- FT-03 verifies `.accounts` default behavior without `refresh::1` or `touch::1`: no network calls, no subprocesses. Use `trace::1` to assert no `[trace] fetch` or `[trace] touch` lines appear.
- FT-04 verifies `.usage` default behavior includes Owner column in output and that `sort::renew` ordering is applied without explicit `sort::` param.
- FT-05 is an integration test via `./verb/test` — identical to the former FT-02 in `36_account_ownership.md` but via `accounts owner::0 name::alice` (Feature 064; formerly `unclaim::1`).
- FT-06 verifies G8 gate: non-owner caller on `.accounts owner::0 name::X` exits 1 before `dry::1` is checked (Feature 064).
- FT-07 is an integration test: set up two accounts (alice owned by current, bob owned by other). `.accounts owner::0` with no `name::` applies ownership release to alice (G8 passes, owner cleared); emits `"skip bob: owned by other@remote"` for bob and continues. Exit 0 always (best-effort batch — per-account G8 violations produce skip messages, not failures). (Feature 064; formerly `unclaim::1` batch.)
- FT-08 verifies that only the marker file is written — mtime of `{name}.credentials.json`, `{name}.json`, and `~/.claude.json` are all unchanged after `.accounts assignee::user1@w003 name::X` (Feature 065; formerly `active::user1@w003 name::X` — Feature 064; formerly `assign::1 name::X`).
- FT-09 verifies `assignee::` sanitization: `assignee::bob@my-laptop` → marker `_active_my-laptop_bob` (dashes and dots preserved, other specials → `_`). (Feature 065; formerly `active::bob@my-laptop` — Feature 064; formerly `assign::1 name::X for::bob@my-laptop`.)
- FT-11 and FT-12 are integration tests via `./verb/test` — verify exit 1 and that stderr contains the targeted migration hint. These commands are registered as redirect stubs (Feature 037): `.account.unclaim` exits 1 with `"owner::0"` hint; `.account.assign` exits 1 with `"assignee::"` hint. NOT generic "unknown command" errors.
- FT-13 uses one sub-case per legacy toggle — 15 invocations; each exits 1 with a message mentioning `cols::`.
- FT-14 is a render test verifying column set modification: identity default is Account, Owner, Active, Current, Sub, Tier, Expires, Email. After `cols::+host,-tier`: Tier removed, Host added.
- FT-15 is an integration test: `.accounts refresh::1` must produce live quota output matching what `.usage` produces for the same accounts.
- FT-18 verifies G8 gate still runs in dry mode: (a) owned by caller → `[dry-run] would clear owner of X` printed, exits 0; (b) owned by other → exits 1, no dry-run line. Uses `owner::0 name::X dry::1` (Feature 064; formerly `dry::1 unclaim::1`).
- FT-19 verifies Owner column: set up alice with `owner: "testuser@testmachine"`, bob with `owner: ""`. `.accounts` text output: alice row shows `testuser@testmachine` in Owner column, bob shows `—`. `.accounts cols::-owner` output: no Owner column header.
- FT-20 verifies G8 bypass via force: same non-owned setup as FT-06; with `force::1` added to `owner::0 name::X`, exits 0 and `alice.json` has `"owner": ""`. (Feature 064; formerly `unclaim::1 force::1`.)
- FT-21 verifies force is a no-op without `owner::`: `.accounts force::1` (no mutation) runs normally; `.accounts force::1 assignee::user1@w003 name::alice` writes marker normally. No error in either case. (Feature 065; formerly `active::` — Feature 064; formerly with `assign::1`.)
- FT-22 verifies REMOVED_TOGGLE stubs: `assign::1`, `unclaim::1`, `for::user@host`, and `active::user@host` on `.accounts` each exit 1 with migration messages pointing to `assignee::USER@MACHINE name::X`, `owner::0 name::X` respectively. (Feature 065 adds `active::` stub; Feature 064 added `assign::1`/`for::`/`unclaim::1` stubs.)
- FT-23 verifies batch set via comma-list `name::X,Y,Z` with `owner::USER@MACHINE`; each G8-evaluated independently; all succeed when caller owns or accounts are unowned.
- FT-24 verifies batch clear via comma-list `name::X,Y,Z` with `owner::0`; each G8-evaluated independently; accounts owned by others skipped with `"skip"` message (batch-clear mode).

---

### FT-01: `.accounts` accepts all 32 unified parameters; unknown param exits 1

- **Given:** Credential store with at least one account.
- **When:** `.accounts` called with each of the 32 unified params (see Feature 037 Design table) set to a valid value.
- **Then:** Exits 0 for all 32 param invocations. No "unknown parameter" error.
- **When:** `.accounts unknown_param::1` called.
- **Then:** Exits 1 with error message referencing the unknown parameter.
- **Exit:** 0 (32 valid cases), 1 (unknown param)
- **Source fn:** `ft01_accounts_accepts_32_params`
- **Source:** [037_accounts_usage_param_unification.md AC-01](../../../docs/feature/037_accounts_usage_param_unification.md)

---

### FT-02: `.usage` accepts all 32 unified parameters; unknown param exits 1

- **Given:** Credential store with at least one account.
- **When:** `.usage` called with each of the 32 unified params set to a valid value.
- **Then:** Exits 0 for all 32 param invocations.
- **When:** `.usage unknown_param::1` called.
- **Then:** Exits 1 with error message referencing the unknown parameter.
- **Exit:** 0 (32 valid cases), 1 (unknown param)
- **Source fn:** `f37_ft02_usage_accepts_32_params` (`usage_feature_test.rs`)
- **Source:** [037_accounts_usage_param_unification.md AC-02](../../../docs/feature/037_accounts_usage_param_unification.md)

---

### FT-03: `.accounts` defaults — no HTTP fetch or subprocess; identity column set; sort by name

- **Given:** Credential store with accounts including at least one non-expired account. No explicit `refresh::` or `touch::` param.
- **When:** `clp .accounts trace::1` is executed.
- **Then:** Exits 0. No `[trace] fetch` or `[trace] touch` lines in output. Output columns include Account, Owner, Active, Current, Sub, Tier, Expires, Email (identity set). Rows sorted alphabetically by account name. No HTTP call performed.
- **Exit:** 0
- **Source fn:** `ft03_accounts_default_profile`
- **Source:** [037_accounts_usage_param_unification.md AC-03](../../../docs/feature/037_accounts_usage_param_unification.md)

---

### FT-04: `.usage` defaults — live fetch, quota column set with Owner, sort by renew

- **Given:** Credential store with at least one owned account with known quota data.
- **When:** `clp .usage trace::1` is executed.
- **Then:** Exits 0. `[trace] fetch` lines appear (live fetch active). Output columns include Status, Account, Owner, 5h Left, 5h Reset, 7d Left, 7d(Son), 7d Reset, Expires, ~Renews, → Next (quota set). Owner column shows owner identity. Rows sorted by `~Renews` (soonest first).
- **Exit:** 0
- **Source fn:** `f37_ft04_usage_default_profile` (`usage_feature_test.rs`)
- **Source:** [037_accounts_usage_param_unification.md AC-04](../../../docs/feature/037_accounts_usage_param_unification.md)

---

### FT-05: `.accounts owner::0 name::X` clears owner; credentials and marker unchanged (Feature 064)

- **Given:** Account `alice` has `alice.json` with `"owner": "testuser@testmachine"`. Current identity = `"testuser@testmachine"` (G8 passes). Record mtime of `alice.credentials.json` and active marker.
- **When:** `clp .accounts owner::0 name::alice` is executed. (Formerly `unclaim::1 name::alice` — Feature 064.)
- **Then:** Exits 0. `alice.json` contains `"owner": ""`. mtime of `alice.credentials.json` unchanged. Active marker unchanged. Output identical to former `clp .account.unclaim name::alice`.
- **Exit:** 0
- **Source fn:** `it01_unclaim_clears_owner` (`account_mutations_test.rs`)
- **Source:** [037_accounts_usage_param_unification.md AC-05](../../../docs/feature/037_accounts_usage_param_unification.md)

---

### FT-06: `.accounts owner::0 name::X` exits 1 with G8 violation; gate before dry-run (Feature 064)

- **Given (case A):** Account `alice` has `alice.json` with `"owner": "other@remote"`. Current identity ≠ `"other@remote"`.
- **When (case A):** `clp .accounts owner::0 name::alice` executed. (Formerly `unclaim::1 name::alice` — Feature 064.)
- **Then (case A):** Exits 1. stdout contains `"ownership violation: this account is owned by other@remote"`. `alice.json` unchanged.
- **Given (case B):** Same non-owned setup.
- **When (case B):** `clp .accounts owner::0 name::alice dry::1` executed.
- **Then (case B):** Exits 1 with ownership violation. G8 gate runs BEFORE `dry::1` check — no dry-run line printed.
- **Exit:** 1 (both cases)
- **Source fn:** `ft16_unclaim_g8_gate` (`account_mutations_test.rs`)
- **Source:** [037_accounts_usage_param_unification.md AC-06](../../../docs/feature/037_accounts_usage_param_unification.md)

---

### FT-07: `.accounts owner::0` batch — applies to all filtered accounts; G8 per-account (Feature 064)

- **Given:** Accounts `acct-a` (owned by current identity) and `acct-b` (owned by `other@remote`). Current identity ≠ `other@remote`.
- **When:** `clp .accounts owner::0` (no `name::`) is executed. (Formerly `unclaim::1` batch — Feature 064.)
- **Then:** `acct-a` ownership released (G8 passes) — `acct-a.json` has `"owner": ""`. `acct-b` unchanged — G8 skip for `acct-b` reported on stdout (`"skip acct-b: owned by other@remote"`). Overall exit code is 0 (best-effort; per-account skips are logged, not failures).
- **Exit:** 0 (best-effort — G8 violations skipped; skip message logged to stdout)
- **Source fn:** `ft07_accounts_unclaim_batch`
- **Source:** [037_accounts_usage_param_unification.md AC-07](../../../docs/feature/037_accounts_usage_param_unification.md)

---

### FT-08: `.accounts active::user1@w003 name::X` writes marker only; no other files modified (Feature 064)

- **Given:** Account `alice` exists. Record mtime of `alice.credentials.json`, `alice.json`, `~/.claude.json`, and any existing marker file.
- **When:** `clp .accounts active::user1@w003 name::alice` is executed. (Formerly `assign::1 name::alice` — Feature 064.)
- **Then:** Exits 0. Marker file `_active_w003_user1` in credential store contains `alice`. mtime of `alice.credentials.json` unchanged. mtime of `alice.json` unchanged. mtime of `~/.claude.json` unchanged.
- **Exit:** 0
- **Source fn:** `aa01_current_machine_marker_written` (`account_assign_test.rs`)
- **Source:** [037_accounts_usage_param_unification.md AC-08](../../../docs/feature/037_accounts_usage_param_unification.md)

---

### FT-09: `.accounts active::bob@my-laptop name::X` writes marker with sanitized path (Feature 064)

- **Given:** Account `alice` exists.
- **When:** `clp .accounts active::bob@my-laptop name::alice` is executed. (Formerly `assign::1 name::alice for::bob@my-laptop` — Feature 064.)
- **Then:** Exits 0. Marker file `_active_my-laptop_bob` in credential store contains `alice`. Sanitization rule: alphanumeric, `-`, `.` preserved; all other chars → `_`.
- **Exit:** 0
- **Source fn:** `aa02_remote_machine_marker_written` (`account_assign_test.rs`)
- **Source:** [037_accounts_usage_param_unification.md AC-09](../../../docs/feature/037_accounts_usage_param_unification.md)

---

### FT-10: `.accounts active::user1@w003` (no `name::`) clears marker for that identity (Feature 064)

- **Given:** Credential store with marker `_active_w003_user1` = `alice`. Current identity resolves to `testuser@testmachine`.
- **When:** `clp .accounts active::user1@w003` (no `name::`) is executed. (Feature 064 — no-name now unassigns; the former `assign::1` no-name emitted a usage block, no longer applicable.)
- **Then:** Exits 0. Marker file `_active_w003_user1` is cleared/removed from credential store. `alice.json` unchanged. `alice.credentials.json` unchanged.
- **Exit:** 0
- **Source fn:** `aa04_no_name_emits_usage_block` (`account_assign_test.rs`)
- **Source:** [037_accounts_usage_param_unification.md AC-10](../../../docs/feature/037_accounts_usage_param_unification.md)

---

### FT-11: `.account.unclaim` exits 1 with targeted migration hint to `owner::0`

- **Given:** Any environment.
- **When:** `clp .account.unclaim name::alice` is executed.
- **Then:** Exits 1. Error output contains `"owner::0"` migration hint (added by redirect stub, Feature 037). No `alice.json` modification.
- **Exit:** 1
- **Source fn:** `ft11_account_unclaim_fully_deregistered` (`accounts_test.rs`)
- **Source:** [037_accounts_usage_param_unification.md AC-11](../../../docs/feature/037_accounts_usage_param_unification.md)

---

### FT-12: `.account.assign` exits 1 with targeted migration hint to `assignee::`

- **Given:** Any environment.
- **When:** `clp .account.assign name::alice` is executed.
- **Then:** Exits 1. Error output contains `"assignee::"` migration hint (added by redirect stub, Feature 065). No marker file written.
- **Exit:** 1
- **Source fn:** `ft12_account_assign_fully_deregistered` (`accounts_test.rs`)
- **Source:** [037_accounts_usage_param_unification.md AC-12](../../../docs/feature/037_accounts_usage_param_unification.md)

---

### FT-13: `.accounts` rejects all 15 legacy field toggle parameters

- **Given:** Credential store with at least one account.
- **When:** `.accounts {toggle}::1` called for each of: `active`, `current`, `sub`, `tier`, `expires`, `email`, `display_name`, `host`, `role`, `billing`, `model`, `uuid`, `capabilities`, `org_uuid`, `org_name`.
- **Then:** Each invocation exits 1. Error message references `cols::` syntax.
- **Exit:** 1 (all 15 cases)
- **Source fn:** `ft13_accounts_legacy_toggles_rejected`
- **Source:** [037_accounts_usage_param_unification.md AC-13](../../../docs/feature/037_accounts_usage_param_unification.md)

---

### FT-14: `.accounts cols::+host,-tier` adds host column and removes tier from identity default set

- **Given:** Credential store with at least one account having `host::` metadata set.
- **When:** `clp .accounts cols::+host,-tier` is executed.
- **Then:** Exits 0. Output column headers include Host. Output column headers do NOT include Tier. All other identity-set columns (Account, Owner, Active, Current, Sub, Expires, Email) present.
- **Exit:** 0
- **Source fn:** `ft14_accounts_cols_modifier`
- **Source:** [037_accounts_usage_param_unification.md AC-14](../../../docs/feature/037_accounts_usage_param_unification.md)

---

### FT-15: `.accounts refresh::1` and `.accounts touch::1` use same algorithm as `.usage`

- **Given:** Credential store with at least one owned account. Network accessible.
- **When (case A):** `clp .accounts refresh::1 trace::1` is executed; separately, `clp .usage trace::1` is executed.
- **Then (case A):** Both produce `[trace] fetch` lines; quota columns in `.accounts refresh::1` output match values in `.usage` output for same accounts.
- **When (case B):** `clp .accounts touch::1 trace::1` is executed.
- **Then (case B):** `[trace] touch` lines appear; same accounts touched as `.usage touch::1` would touch.
- **Exit:** 0
- **Source fn:** `lim_it_ft15_accounts_refresh_live (accounts_test.rs)`
- **Source:** [037_accounts_usage_param_unification.md AC-15](../../../docs/feature/037_accounts_usage_param_unification.md)

---

### FT-16: `.usage owner::0 name::X` clears owner field — identical to `.accounts owner::0 name::X` (Feature 064)

- **Given:** Account `alice` with `"owner": "testuser@testmachine"`. Current identity = `"testuser@testmachine"` (G8 passes).
- **When:** `clp .usage owner::0 name::alice` is executed. (Formerly `unclaim::1 name::alice` — Feature 064.)
- **Then:** Exits 0. `alice.json` contains `"owner": ""`. Behavior identical to FT-05 (`.accounts owner::0 name::alice`).
- **Exit:** 0
- **Source fn:** `f37_ft16_usage_unclaim_mirrors_accounts` (`usage_feature_test.rs`)
- **Source:** [037_accounts_usage_param_unification.md AC-16](../../../docs/feature/037_accounts_usage_param_unification.md)

---

### FT-17: `.usage assignee::user1@w003 name::X` writes marker — identical to `.accounts assignee::user1@w003 name::X` (Feature 065)

- **Given:** Account `alice` exists.
- **When:** `clp .usage assignee::user1@w003 name::alice` is executed. (Feature 065; formerly `active::user1@w003 name::alice` — Feature 064; formerly `assign::1 name::alice`.)
- **Then:** Exits 0. Marker file `_active_w003_user1` in credential store contains `alice`. Behavior identical to FT-08 (`.accounts assignee::user1@w003 name::alice`).
- **Exit:** 0
- **Source fn:** `f37_ft17_usage_assign_mirrors_accounts` (`usage_feature_test.rs`)
- **Source:** [037_accounts_usage_param_unification.md AC-17](../../../docs/feature/037_accounts_usage_param_unification.md)

---

### FT-18: `.accounts dry::1 owner::0 name::X` — dry-run previews; G8 still runs (Feature 064)

- **Given (case A):** Account `alice` with `"owner": "testuser@testmachine"`. Current identity = `"testuser@testmachine"` (G8 passes).
- **When (case A):** `clp .accounts dry::1 owner::0 name::alice` executed. (Formerly `dry::1 unclaim::1 name::alice` — Feature 064.)
- **Then (case A):** Exits 0. stdout contains `[dry-run] would clear owner of alice`. `alice.json` still contains `"owner": "testuser@testmachine"` — unchanged.
- **Given (case B):** Account `bob` with `"owner": "other@remote"`. Current identity ≠ `"other@remote"` (G8 fails).
- **When (case B):** `clp .accounts dry::1 owner::0 name::bob` executed.
- **Then (case B):** Exits 1. G8 ownership violation — no `[dry-run]` line printed. `bob.json` unchanged.
- **Exit:** 0 (case A), 1 (case B)
- **Source fn:** `ft17_unclaim_dry_run` (`account_mutations_test.rs`)
- **Source:** [037_accounts_usage_param_unification.md AC-18](../../../docs/feature/037_accounts_usage_param_unification.md)

---

### FT-19: Owner column visible by default on both `.accounts` and `.usage`; `cols::-owner` hides it

- **Given:** Accounts: `alice` with `"owner": "testuser@testmachine"`, `bob` with `"owner": ""`.
- **When (case A):** `clp .accounts` (no cols modifier) executed.
- **Then (case A):** Owner column header present. `alice` row: Owner = `testuser@testmachine`. `bob` row: Owner = `—`.
- **When (case B):** `clp .usage` (no cols modifier) executed.
- **Then (case B):** Owner column header present. Same owner values as case A.
- **When (case C):** `clp .accounts cols::-owner` executed.
- **Then (case C):** No Owner column header in output.
- **Exit:** 0
- **Source fn:** `ft19_owner_column_default_visible`
- **Source:** [037_accounts_usage_param_unification.md AC-19](../../../docs/feature/037_accounts_usage_param_unification.md)

---

### FT-20: `.accounts owner::0 name::X force::1` bypasses G8 and clears owner (Feature 064)

- **Given:** Account `alice` with `"owner": "other@remote"`. Current identity ≠ `"other@remote"` (G8 would fail without force).
- **When:** `clp .accounts owner::0 name::alice force::1` is executed. (Formerly `unclaim::1 name::alice force::1` — Feature 064.)
- **Then:** Exits 0. G8 gate bypassed — no ownership violation exit 1. `alice.json` contains `"owner": ""`. stdout contains `unclaimed alice`.
- **Exit:** 0
- **Source fn:** `ft20_accounts_unclaim_force_bypasses_g8`
- **Source:** [037_accounts_usage_param_unification.md AC-20](../../../docs/feature/037_accounts_usage_param_unification.md)

---

### FT-21: `force::1` without `owner::` or with `active::` is silently ignored (Feature 064)

- **Given:** Credential store with account `alice`.
- **When (case A):** `clp .accounts force::1` (no `owner::`, no mutation) executed.
- **Then (case A):** Exits 0. `force::1` has no effect — `.accounts` executes normally (lists accounts). No error.
- **When (case B):** `clp .accounts force::1 active::user1@w003 name::alice` executed. (Formerly `force::1 assign::1 name::alice` — Feature 064.)
- **Then (case B):** Exits 0. Marker file written normally (same as without `force::1`). `force::1` has no effect on `active::` path. No error.
- **Exit:** 0 (both cases)
- **Source fn:** `ft21_force_no_effect_without_unclaim`
- **Source:** [037_accounts_usage_param_unification.md AC-21](../../../docs/feature/037_accounts_usage_param_unification.md)

---

### FT-22: REMOVED_TOGGLE stubs — `assign::1`, `unclaim::1`, `for::` exit 1 with migration messages (Feature 064)

- **Given:** Any environment with at least one account.
- **When (case A):** `clp .accounts assign::1 name::alice` is executed.
- **Then (case A):** Exits 1. Error message: "REMOVED — use `active::USER@MACHINE name::X`". No marker file written.
- **When (case B):** `clp .accounts unclaim::1 name::alice` is executed.
- **Then (case B):** Exits 1. Error message: "REMOVED — use `owner::0 name::X`". No file written.
- **When (case C):** `clp .accounts for::user@host name::alice` is executed.
- **Then (case C):** Exits 1 with REMOVED migration message. No file written.
- **Exit:** 1 (all cases)
- **Source fn:** `ft22_removed_toggle_stubs`
- **Source:** [037_accounts_usage_param_unification.md AC-22](../../../docs/feature/037_accounts_usage_param_unification.md)

---

### FT-23: `.accounts owner::user1@w003 name::X,Y,Z` — batch ownership set (Feature 064)

- **Given:** Accounts `alice`, `bob`, `carol` all unowned (or owned by current identity). Current identity = `"testuser@testmachine"` (G8 passes for all).
- **When:** `clp .accounts owner::user1@w003 name::alice,bob,carol` is executed.
- **Then:** Exits 0. `alice.json`, `bob.json`, `carol.json` each contain `"owner": "user1@w003"`. Each evaluated against G8 independently.
- **Exit:** 0
- **Source fn:** `ft23_owner_batch_set`
- **Source:** [037_accounts_usage_param_unification.md AC-23](../../../docs/feature/037_accounts_usage_param_unification.md)

---

### FT-24: `.accounts owner::0 name::X,Y,Z` — batch ownership clear (Feature 064)

- **Given:** Accounts `alice` (owned by current identity), `bob` (owned by current identity), `carol` (owned by `other@remote`). Current identity ≠ `"other@remote"`.
- **When:** `clp .accounts owner::0 name::alice,bob,carol` is executed.
- **Then:** Exits 0 (best-effort). `alice.json` and `bob.json` each have `"owner": ""` (G8 passes). `carol.json` unchanged — G8 violation for `carol` reported as skip message on stdout. Overall exit 0.
- **Exit:** 0 (best-effort; per-account G8 violations in comma-list mode = skip + continue)
- **Source fn:** `ft24_owner_batch_clear`
- **Source:** [037_accounts_usage_param_unification.md AC-24](../../../docs/feature/037_accounts_usage_param_unification.md)
