# Feature: Accounts/Usage Parameter Set Unification

### Scope

- **Purpose**: Make `.accounts` and `.usage` symmetric commands sharing 32 parameters with different defaults, absorbing `.account.unclaim` and `.account.assign` as mutation parameters, and introducing `force::` to bypass ownership enforcement on unclaim.
- **Responsibility**: Documents the unified 32-parameter set, default differentiation, `cols::` replacement of 15 field toggles, `unclaim::` and `assign::`/`for::` mutation params, `force::` bypass param, and removal of `.account.unclaim` and `.account.assign` as standalone commands.
- **In Scope**: 32-param unified parameter set; `.accounts` default profile (local/identity view: `refresh::0`, `touch::0`, `sort::name`); `.usage` default profile (live/quota view: `refresh::1`, `touch::1`, `sort::renew`); `cols::` syntax on `.accounts` replacing 15 individual field toggles (`active::`, `current::`, `sub::`, `tier::`, `expires::`, `email::`, `display_name::`, `host::`, `role::`, `billing::`, `model::`, `uuid::`, `capabilities::`, `org_uuid::`, `org_name::`); `unclaim::` mutation param; `assign::`/`for::` mutation params; `force::` bypass param (bypasses G8 ownership check when used with `unclaim::1`); removal of standalone `.account.unclaim` (Command 17) and `.account.assign` (Command 16); command count reduction 18→16.
- **Out of Scope**: New quota fetching features; changes to `.account.save`, `.account.use`, `.account.delete`, `.account.limits`, `.account.relogin`, `.account.rotate`, `.account.renewal`, `.account.inspect`, `.model`, or any non-account command; G1–G4 read-side gates (behavior unchanged — `force::` does not bypass them); adding `force::` to `.account.use`/`.account.delete`/`.account.relogin` (G5–G7 bypass → Feature 036 implementation).

### Design

**Symmetric commands.** `.accounts` and `.usage` become two views of the same underlying account data, sharing an identical 32-parameter interface. The only difference is defaults — `.accounts` is a local/identity view (no fetching, no touching, sorted by name), while `.usage` is a live/quota view (fetch + touch enabled, sorted by renewal).

**Unified parameter set (32 params):**

| Category | Parameters | `.accounts` default | `.usage` default |
|----------|-----------|---------------------|------------------|
| Selection | `name::`, `count::`, `offset::`, `only_active::`, `only_next::` | —, 0, 0, 0, 0 | —, 0, 0, 0, 0 |
| Data source | `refresh::`, `touch::`, `imodel::`, `effort::`, `live::`, `interval::`, `jitter::` | **0**, **0**, auto, auto, 0, 30, 0 | **1**, **1**, auto, auto, 0, 30, 0 |
| Sort & filter | `sort::`, `desc::`, `prefer::`, `next::`, `min_5h::`, `min_7d::`, `only_valid::`, `exclude_exhausted::` | **name**, 0, any, renew, 0, 0, 0, 0 | **renew**, 0, any, renew, 0, 0, 0, 0 |
| Columns | `cols::` | **identity set** | **quota set** |
| Output | `format::`, `get::`, `abs::`, `no_color::`, `trace::` | text, —, 0, 0, 0 | text, —, 0, 0, 0 |
| Mutations | `dry::`, `unclaim::`, `assign::`, `for::`, `set_model::`, `force::` | 0, 0, 0, current, —, 0 | 0, 0, 0, current, —, 0 |

**Default column sets.** `cols::` replaces the 15 individual field toggles on `.accounts`. Both default sets include the `Owner` column — showing the `owner` field from `{name}.json` (`USER@MACHINE` identity or `—` when unowned). This overrides the Feature 036 out-of-scope exclusion for owner display.

The identity set (`.accounts` default) includes: Account, Owner, Active, Current, Sub, Tier, Expires, Email. The quota set (`.usage` default) includes: Status, Account, Owner, 5h Left, 5h Reset, 7d Left, 7d(Son), 7d Reset, Expires, ~Renews, → Next. Both commands support `cols::+col_id` / `cols::-col_id` modifiers to add/remove columns from the default set. `cols::-owner` hides the owner column.

**Command absorption — `.account.unclaim` → `unclaim::` param.**
`clp .accounts unclaim::1 name::X` clears the `owner` field in `{name}.json` — identical behavior to the former `clp .account.unclaim name::X`. G8 ownership gate is evaluated before write. Batch support: when `name::` is omitted, `unclaim::1` applies to all accounts matching the current filter predicates. `dry::1` previews without writing.

**Command absorption — `.account.assign` → `assign::` + `for::` params.**
`clp .accounts assign::1 name::X` writes the per-machine active-account marker — identical to the former `clp .account.assign name::X`. `for::USER@MACHINE` targets a specific machine's marker (default: current machine). When `name::` is absent and `assign::1`, emits the live usage block (current machine identity + copy-paste examples). `dry::1` previews without writing.

**Standalone command removal.** `.account.unclaim` and `.account.assign` are removed as standalone commands. `clp .account.unclaim ...` and `clp .account.assign ...` exit 1 with an error directing to the new syntax. Command count: 18→16 (16 registered → 14 registered; `.`/`.help` hardcoded, not registered).

**`force::` ownership bypass.** `force::1` bypasses the G8 ownership check when used with `unclaim::1`. It allows any machine/user identity to release ownership of an account, regardless of whether `current_identity()` matches the stored owner. `force::1` without `unclaim::1` has no effect (silently ignored). `force::1` with `assign::1` has no effect (assign has no ownership gate). `force::1` does NOT bypass `dry::1` — dry-run preview still applies when both are set.

**`.usage` mutation params.** `.usage` gains the same mutation params (`dry::`, `unclaim::`, `assign::`, `for::`, `set_model::`, `force::`) since both commands share the full parameter set. `set_model::` was already on `.usage`.

### Acceptance Criteria

- **AC-01**: `.accounts` accepts all 32 parameters from the unified set. Unknown parameters exit 1.
- **AC-02**: `.usage` accepts all 32 parameters from the unified set. Unknown parameters exit 1.
- **AC-03**: `.accounts` defaults: `refresh::0`, `touch::0`, `sort::name`, `cols::` = identity set (Account, Owner, Active, Current, Sub, Tier, Expires, Email). No HTTP fetch or subprocess spawn when invoked without explicit `refresh::1` or `touch::1`.
- **AC-04**: `.usage` defaults: `refresh::1`, `touch::1`, `sort::renew`, `cols::` = quota set (Status, Account, Owner, 5h Left, 5h Reset, 7d Left, 7d(Son), 7d Reset, Expires, ~Renews, → Next). Owner column added to default; all other behavior unchanged.
- **AC-05**: `.accounts unclaim::1 name::X` exits 0 and writes `owner: ""` to `{name}.json` when G8 passes. Credentials NOT touched. Active marker NOT changed. Identical observable behavior to the former `.account.unclaim name::X`.
- **AC-06**: `.accounts unclaim::1 name::X` when `!is_owned(account)` exits 1 with `"ownership violation: this account is owned by {owner}"`. G8 gate evaluates before `dry::1` check.
- **AC-07**: `.accounts unclaim::1` (no `name::`) applies unclaim to every account matching current filter predicates (`only_active::`, `only_valid::`, etc.). Each account independently evaluated against G8.
- **AC-08**: `.accounts assign::1 name::X` writes `{credential_store}/_active_{machine}_{user}` = X. Exits 0. `~/.claude/.credentials.json`, `~/.claude.json`, `{name}.json` NOT touched.
- **AC-09**: `.accounts assign::1 name::X for::bob@laptop` writes `{credential_store}/_active_laptop_bob` = X. Sanitization rules identical to former `.account.assign`.
- **AC-10**: `.accounts assign::1` (no `name::`) emits live usage block with current machine identity, active account, and copy-paste examples. Exits 0.
- **AC-11**: `clp .account.unclaim name::X` exits 1 with error message: `"unknown command '.account.unclaim' — use '.accounts unclaim::1 name::X' instead"`.
- **AC-12**: `clp .account.assign name::X` exits 1 with error message: `"unknown command '.account.assign' — use '.accounts assign::1 name::X' instead"`.
- **AC-13**: `.accounts` no longer accepts the 15 individual field toggles (`active::`, `current::`, `sub::`, `tier::`, `expires::`, `email::`, `display_name::`, `host::`, `role::`, `billing::`, `model::`, `uuid::`, `capabilities::`, `org_uuid::`, `org_name::`). These parameters exit 1 with an error directing to `cols::` syntax.
- **AC-14**: `.accounts cols::+host,-tier` adds host column and removes tier column from the identity default set. Syntax and behavior identical to `.usage cols::` (Feature 029).
- **AC-15**: `.accounts refresh::1` fetches live quota data for all accounts — same algorithm as `.usage`. `.accounts touch::1` activates idle session windows — same algorithm as `.usage`.
- **AC-16**: `.usage unclaim::1 name::X` clears owner field — identical to `.accounts unclaim::1 name::X`.
- **AC-17**: `.usage assign::1 name::X` writes marker — identical to `.accounts assign::1 name::X`.
- **AC-18**: `.accounts dry::1 unclaim::1 name::X` prints `[dry-run] would unclaim X` and exits 0. No files modified. G8 gate still runs.
- **AC-19**: Owner column visible by default on both `.accounts` and `.usage`. Shows `owner` field from `{name}.json` — `USER@MACHINE` when owned, `—` when unowned (empty or absent). `cols::-owner` hides it. Overrides Feature 036 out-of-scope exclusion for owner display column.
- **AC-20**: `.accounts unclaim::1 name::X force::1` clears the owner field even when `current_identity() ≠ stored_owner` — G8 gate is bypassed. Exits 0. Output: `unclaimed X`. G8 enforcement (exit 1 with ownership violation message) is suppressed only when `force::1` is present.
- **AC-21**: `.accounts force::1` without `unclaim::1`, or `.accounts force::1 assign::1 name::X`, silently ignores `force::1` — no error, no change to behavior. `force::` is a mutation-scoped bypass with no standalone meaning.

### Features

| File | Relationship |
|------|--------------|
| [003_account_list.md](003_account_list.md) | `.accounts` baseline — account enumeration and per-account block rendering |
| [009_token_usage.md](009_token_usage.md) | `.usage` baseline — quota fetch algorithm |
| [032_account_assign.md](032_account_assign.md) | `.account.assign` behavior absorbed as `assign::` + `for::` params |
| [036_account_ownership.md](036_account_ownership.md) | `.account.unclaim` behavior absorbed as `unclaim::` param; G8 gate preserved |
| [028_usage_row_filtering.md](028_usage_row_filtering.md) | Row filtering params (`count::`, `offset::`, `only_active::`, etc.) added to `.accounts` |
| [020_usage_sort_strategies.md](020_usage_sort_strategies.md) | Sort strategies (`sort::`, `desc::`, `prefer::`) added to `.accounts` |
| [023_next_account_strategies.md](023_next_account_strategies.md) | Next recommendation (`next::`) added to `.accounts` |
| [024_session_touch.md](024_session_touch.md) | Touch (`touch::`) available on `.accounts` with default `0` |
| [017_token_refresh.md](017_token_refresh.md) | Refresh (`refresh::`) available on `.accounts` with default `0` |
| [029_account_host_metadata.md](029_account_host_metadata.md) | `cols::` syntax shared between `.accounts` and `.usage` |
| [034_explicit_session_model_override.md](034_explicit_session_model_override.md) | `set_model::` available on both `.accounts` and `.usage` |

### Commands

| File | Relationship |
|------|--------------|
| [cli/command/001_account.md](../cli/command/001_account.md) | `.accounts` — Command 3; `.account.assign` (Command 16) and `.account.unclaim` (Command 17) removed |
| [cli/command/006_usage.md](../cli/command/006_usage.md) | `.usage` — Command 9; gains mutation params |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/056_unclaim.md](../cli/param/056_unclaim.md) | `unclaim::` — mutation param; clears owner field (re-activated from REMOVED) |
| [cli/param/057_assign.md](../cli/param/057_assign.md) | `assign::` — mutation param; writes per-machine active-account marker |
| [cli/param/058_force.md](../cli/param/058_force.md) | `force::` — bypass G8 ownership check when used with `unclaim::1` |

### Sources

| File | Relationship |
|------|--------------|
| `src/registry.rs` | Command registration — unified param set for `.accounts` and `.usage` |
| `src/commands/accounts.rs` | `.accounts` handler — absorbs unclaim and assign logic |
| `src/commands/account_ops.rs` | `account_unclaim_routine()` moves to accounts handler |
| `src/commands/account_assign.rs` | `account_assign_routine()` moves to accounts handler; file deleted |
| `src/usage/api.rs` | `.usage` handler (`usage_routine()`) — gains mutation param dispatch |

### Tests

| File | Relationship |
|------|--------------|
| `tests/cli/accounts_test.rs` | Integration tests for `.accounts` — absorbs assign and unclaim test cases |
| `tests/cli/account_assign_test.rs` | Tests migrated to `accounts_test.rs`; file deleted |
| `tests/cli/usage_test.rs` | Integration tests for `.usage` — gains mutation param tests |
