# FT — Feature 037: Accounts/Usage Parameter Set Unification

### Scope

- **Purpose**: Test cases for the unified 32-parameter interface shared by `.accounts` and `.usage`, including `assign::` and `unclaim::` mutation params, `force::` G8 bypass, `cols::` default sets, Owner column, command absorption, and standalone command removal.
- **Source**: `docs/feature/037_accounts_usage_param_unification.md`
- **Covers**: AC-01 through AC-21

### Test Cases

| FT | AC | Scenario | Source fn |
|----|----|----------|-----------|
| FT-01 | AC-01 | `.accounts` accepts all 32 unified params; unknown param exits 1 | `ft01_accounts_accepts_32_params` |
| FT-02 | AC-02 | `.usage` accepts all 32 unified params; unknown param exits 1 | `f37_ft02_usage_accepts_32_params` |
| FT-03 | AC-03 | `.accounts` defaults: `refresh::0`, `touch::0`, `sort::name`, `cols::` = identity set; no HTTP fetch or subprocess without explicit flags | `ft03_accounts_default_profile` |
| FT-04 | AC-04 | `.usage` defaults: `refresh::1`, `touch::1`, `sort::renew`, `cols::` = quota set with Owner column | `f37_ft04_usage_default_profile` |
| FT-05 | AC-05 | `.accounts unclaim::1 name::X` exits 0; writes `owner: ""`; credentials and active marker unchanged | `it01_unclaim_clears_owner (account_mutations_test.rs)` |
| FT-06 | AC-06 | `.accounts unclaim::1 name::X` exits 1 with ownership violation when G8 fails; gate runs before `dry::1` | `ft16_unclaim_g8_gate (account_mutations_test.rs)` |
| FT-07 | AC-07 | `.accounts unclaim::1` (no `name::`) applies unclaim to all filtered accounts; each evaluated against G8 | `ft07_accounts_unclaim_batch` |
| FT-08 | AC-08 | `.accounts assign::1 name::X` writes marker file; `{name}.json`, credentials, `~/.claude.json` unchanged | `aa01_current_machine_marker_written (account_assign_test.rs)` |
| FT-09 | AC-09 | `.accounts assign::1 name::X for::bob@laptop` writes `_active_laptop_bob`; sanitization identical to former `.account.assign` | `aa02_remote_machine_marker_written (account_assign_test.rs)` |
| FT-10 | AC-10 | `.accounts assign::1` (no `name::`) emits live usage block with machine identity and copy-paste examples; exits 0 | `aa04_no_name_emits_usage_block (account_assign_test.rs)` |
| FT-13 | AC-13 | `.accounts` rejects all 15 legacy field toggles (`active::`, `current::`, `sub::`, `tier::`, `expires::`, `email::`, `display_name::`, `host::`, `role::`, `billing::`, `model::`, `uuid::`, `capabilities::`, `org_uuid::`, `org_name::`); each exits 1 directing to `cols::` | `ft13_accounts_legacy_toggles_rejected` |
| FT-14 | AC-14 | `.accounts cols::+host,-tier` adds host column and removes tier from identity default set | `ft14_accounts_cols_modifier` |
| FT-15 | AC-15 | `.accounts refresh::1` fetches live quota; `.accounts touch::1` activates idle sessions — same algorithm as `.usage` | `lim_it_ft15_accounts_refresh_live (accounts_test.rs)` |
| FT-16 | AC-16 | `.usage unclaim::1 name::X` clears owner field — identical result to `.accounts unclaim::1 name::X` | `f37_ft16_usage_unclaim_mirrors_accounts` |
| FT-17 | AC-17 | `.usage assign::1 name::X` writes marker — identical result to `.accounts assign::1 name::X` | `f37_ft17_usage_assign_mirrors_accounts` |
| FT-18 | AC-18 | `.accounts dry::1 unclaim::1 name::X` prints `[dry-run] would unclaim X`; exits 0; no files modified; G8 gate runs | `ft17_unclaim_dry_run (account_mutations_test.rs)` |
| FT-19 | AC-19 | Owner column visible by default on `.accounts` and `.usage`; shows owner from `{name}.json`; `cols::-owner` hides it | `ft19_owner_column_default_visible` |
| FT-20 | AC-20 | `.accounts unclaim::1 name::X force::1` bypasses G8; clears owner even when caller ≠ stored owner; exits 0 | `ft20_accounts_unclaim_force_bypasses_g8` |
| FT-21 | AC-21 | `.accounts force::1` without `unclaim::1`, and `.accounts force::1 assign::1 name::X`, silently ignore `force::1` — no error | `ft21_force_no_effect_without_unclaim` |

### Notes

- FT-01 and FT-02 are structural registration tests: run `.accounts` and `.usage` with each of the 32 params set to a valid value and verify exit 0; then try an unknown param and verify exit 1 with error message.
- FT-03 verifies `.accounts` default behavior without `refresh::1` or `touch::1`: no network calls, no subprocesses. Use `trace::1` to assert no `[trace] fetch` or `[trace] touch` lines appear.
- FT-04 verifies `.usage` default behavior includes Owner column in output and that `sort::renew` ordering is applied without explicit `sort::` param.
- FT-05 is an integration test via `./verb/test` — identical to the former FT-02 in `36_account_ownership.md` but via the new `accounts unclaim::1` interface.
- FT-06 verifies G8 gate: non-owner caller on `.accounts unclaim::1` exits 1 before `dry::1` is checked.
- FT-07 is an integration test: set up two accounts (alice owned by current, bob owned by other). `.accounts unclaim::1` with no `name::` applies unclaim to alice (G8 passes, owner cleared); emits `"skip bob: owned by other@remote"` for bob and continues. Exit 0 always (best-effort batch — per-account G8 violations produce skip messages, not failures).
- FT-08 verifies that only the marker file is written — mtime of `{name}.credentials.json`, `{name}.json`, and `~/.claude.json` are all unchanged after `.accounts assign::1 name::X`.
- FT-09 verifies `for::` sanitization: `for::bob@my-laptop` → marker `_active_my-laptop_bob` (dashes and dots preserved, other specials → `_`).
- FT-11 and FT-12 are integration tests via `./verb/test` — verify exit 1 and exact error message text.
- FT-13 uses one sub-case per legacy toggle — 15 invocations; each exits 1 with a message mentioning `cols::`.
- FT-14 is a render test verifying column set modification: identity default is Account, Owner, Active, Current, Sub, Tier, Expires, Email. After `cols::+host,-tier`: Tier removed, Host added.
- FT-15 is an integration test: `.accounts refresh::1` must produce live quota output matching what `.usage` produces for the same accounts.
- FT-18 verifies G8 gate still runs in dry mode: (a) owned by caller → dry-run line printed, exits 0; (b) owned by other → exits 1, no dry-run line.
- FT-19 verifies Owner column: set up alice with `owner: "testuser@testmachine"`, bob with `owner: ""`. `.accounts` text output: alice row shows `testuser@testmachine` in Owner column, bob shows `—`. `.accounts cols::-owner` output: no Owner column header.
- FT-20 verifies G8 bypass via force: same non-owned setup as FT-06; with `force::1` added, exits 0 and `alice.json` has `"owner": ""`.
- FT-21 verifies force is a no-op without unclaim: `.accounts force::1` (no unclaim, no assign) runs normally; `.accounts force::1 assign::1 name::alice` writes marker normally. No error in either case.

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
- **Source fn:** `f37_ft02_usage_accepts_32_params`
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
- **Source fn:** `f37_ft04_usage_default_profile`
- **Source:** [037_accounts_usage_param_unification.md AC-04](../../../docs/feature/037_accounts_usage_param_unification.md)

---

### FT-05: `.accounts unclaim::1 name::X` clears owner; credentials and marker unchanged

- **Given:** Account `alice` has `alice.json` with `"owner": "testuser@testmachine"`. Current identity = `"testuser@testmachine"` (G8 passes). Record mtime of `alice.credentials.json` and active marker.
- **When:** `clp .accounts unclaim::1 name::alice` is executed.
- **Then:** Exits 0. `alice.json` contains `"owner": ""`. mtime of `alice.credentials.json` unchanged. Active marker unchanged. Output identical to former `clp .account.unclaim name::alice`.
- **Exit:** 0
- **Source fn:** `it01_unclaim_clears_owner` (`account_mutations_test.rs`)
- **Source:** [037_accounts_usage_param_unification.md AC-05](../../../docs/feature/037_accounts_usage_param_unification.md)

---

### FT-06: `.accounts unclaim::1 name::X` exits 1 with G8 violation; gate before dry-run

- **Given (case A):** Account `alice` has `alice.json` with `"owner": "other@remote"`. Current identity ≠ `"other@remote"`.
- **When (case A):** `clp .accounts unclaim::1 name::alice` executed.
- **Then (case A):** Exits 1. stdout contains `"ownership violation: this account is owned by other@remote"`. `alice.json` unchanged.
- **Given (case B):** Same non-owned setup.
- **When (case B):** `clp .accounts unclaim::1 name::alice dry::1` executed.
- **Then (case B):** Exits 1 with ownership violation. G8 gate runs BEFORE `dry::1` check — no dry-run line printed.
- **Exit:** 1 (both cases)
- **Source fn:** `ft16_unclaim_g8_gate` (`account_mutations_test.rs`)
- **Source:** [037_accounts_usage_param_unification.md AC-06](../../../docs/feature/037_accounts_usage_param_unification.md)

---

### FT-07: `.accounts unclaim::1` batch — applies to all filtered accounts; G8 per-account

- **Given:** Accounts `acct-a` (owned by current identity) and `acct-b` (owned by `other@remote`). Current identity ≠ `other@remote`.
- **When:** `clp .accounts unclaim::1` (no `name::`) is executed.
- **Then:** `acct-a` unclaimed (G8 passes) — `acct-a.json` has `"owner": ""`. `acct-b` unchanged — G8 skip for `acct-b` reported on stdout (`"skip acct-b: owned by other@remote"`). Overall exit code is 0 (best-effort; per-account skips are logged, not failures).
- **Exit:** 0 (best-effort — G8 violations skipped; skip message logged to stdout)
- **Source fn:** `ft07_accounts_unclaim_batch`
- **Source:** [037_accounts_usage_param_unification.md AC-07](../../../docs/feature/037_accounts_usage_param_unification.md)

---

### FT-08: `.accounts assign::1 name::X` writes marker only; no other files modified

- **Given:** Account `alice` exists. Record mtime of `alice.credentials.json`, `alice.json`, `~/.claude.json`, and any existing marker file.
- **When:** `clp .accounts assign::1 name::alice` is executed.
- **Then:** Exits 0. Marker file `_active_{machine}_{user}` in credential store contains `alice`. mtime of `alice.credentials.json` unchanged. mtime of `alice.json` unchanged. mtime of `~/.claude.json` unchanged.
- **Exit:** 0
- **Source fn:** `aa01_current_machine_marker_written` (`account_assign_test.rs`)
- **Source:** [037_accounts_usage_param_unification.md AC-08](../../../docs/feature/037_accounts_usage_param_unification.md)

---

### FT-09: `.accounts assign::1 name::X for::bob@laptop` writes marker with sanitized path

- **Given:** Account `alice` exists. `for::` target is `bob@my-laptop`.
- **When:** `clp .accounts assign::1 name::alice for::bob@my-laptop` is executed.
- **Then:** Exits 0. Marker file `_active_my-laptop_bob` in credential store contains `alice`. Sanitization rule: alphanumeric, `-`, `.` preserved; all other chars → `_`.
- **Exit:** 0
- **Source fn:** `aa02_remote_machine_marker_written` (`account_assign_test.rs`)
- **Source:** [037_accounts_usage_param_unification.md AC-09](../../../docs/feature/037_accounts_usage_param_unification.md)

---

### FT-10: `.accounts assign::1` (no `name::`) emits live usage block; exits 0

- **Given:** Current machine identity resolves to `testuser@testmachine`. Active account for current machine is `alice`.
- **When:** `clp .accounts assign::1` (no `name::`) is executed.
- **Then:** Exits 0. stdout contains current machine identity (`testuser@testmachine`), active account name (`alice`), and copy-paste examples (`clp .accounts assign::1 name::alice`, `clp .accounts assign::1 name::alice for::testuser@testmachine`, `clp .accounts assign::1 name::alice for::testuser@testmachine dry::1`). No marker file written.
- **Exit:** 0
- **Source fn:** `aa04_no_name_emits_usage_block` (`account_assign_test.rs`)
- **Source:** [037_accounts_usage_param_unification.md AC-10](../../../docs/feature/037_accounts_usage_param_unification.md)

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

### FT-16: `.usage unclaim::1 name::X` clears owner field — identical to `.accounts unclaim::1 name::X`

- **Given:** Account `alice` with `"owner": "testuser@testmachine"`. Current identity = `"testuser@testmachine"` (G8 passes).
- **When:** `clp .usage unclaim::1 name::alice` is executed.
- **Then:** Exits 0. `alice.json` contains `"owner": ""`. Behavior identical to FT-05 (`.accounts unclaim::1 name::alice`).
- **Exit:** 0
- **Source fn:** `f37_ft16_usage_unclaim_mirrors_accounts` (`usage_feature_test.rs`)
- **Source:** [037_accounts_usage_param_unification.md AC-16](../../../docs/feature/037_accounts_usage_param_unification.md)

---

### FT-17: `.usage assign::1 name::X` writes marker — identical to `.accounts assign::1 name::X`

- **Given:** Account `alice` exists.
- **When:** `clp .usage assign::1 name::alice` is executed.
- **Then:** Exits 0. Marker file `_active_{machine}_{user}` in credential store contains `alice`. Behavior identical to FT-08 (`.accounts assign::1 name::alice`).
- **Exit:** 0
- **Source fn:** `f37_ft17_usage_assign_mirrors_accounts` (`usage_feature_test.rs`)
- **Source:** [037_accounts_usage_param_unification.md AC-17](../../../docs/feature/037_accounts_usage_param_unification.md)

---

### FT-18: `.accounts dry::1 unclaim::1 name::X` — dry-run previews; G8 still runs

- **Given (case A):** Account `alice` with `"owner": "testuser@testmachine"`. Current identity = `"testuser@testmachine"` (G8 passes).
- **When (case A):** `clp .accounts dry::1 unclaim::1 name::alice` executed.
- **Then (case A):** Exits 0. stdout contains `[dry-run] would unclaim alice`. `alice.json` still contains `"owner": "testuser@testmachine"` — unchanged.
- **Given (case B):** Account `bob` with `"owner": "other@remote"`. Current identity ≠ `"other@remote"` (G8 fails).
- **When (case B):** `clp .accounts dry::1 unclaim::1 name::bob` executed.
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

### FT-20: `.accounts unclaim::1 name::X force::1` bypasses G8 and clears owner

- **Given:** Account `alice` with `"owner": "other@remote"`. Current identity ≠ `"other@remote"` (G8 would fail without force).
- **When:** `clp .accounts unclaim::1 name::alice force::1` is executed.
- **Then:** Exits 0. G8 gate bypassed — no ownership violation exit 1. `alice.json` contains `"owner": ""`. stdout contains `unclaimed alice`.
- **Exit:** 0
- **Source fn:** `ft20_accounts_unclaim_force_bypasses_g8`
- **Source:** [037_accounts_usage_param_unification.md AC-20](../../../docs/feature/037_accounts_usage_param_unification.md)

---

### FT-21: `force::1` without `unclaim::1` or with `assign::1` is silently ignored

- **Given:** Credential store with account `alice`.
- **When (case A):** `clp .accounts force::1` (no unclaim, no assign, no mutation) executed.
- **Then (case A):** Exits 0. `force::1` has no effect — `.accounts` executes normally (lists accounts). No error.
- **When (case B):** `clp .accounts force::1 assign::1 name::alice` executed.
- **Then (case B):** Exits 0. Marker file written normally (same as without `force::1`). `force::1` has no effect on assign path. No error.
- **Exit:** 0 (both cases)
- **Source fn:** `ft21_force_no_effect_without_unclaim`
- **Source:** [037_accounts_usage_param_unification.md AC-21](../../../docs/feature/037_accounts_usage_param_unification.md)
