# Feature: Accounts/Usage Parameter Set Unification

> **Partial supersession (Feature 064 — shipped; Feature 065 — shipped):** The `assign::1`, `for::`, and `unclaim::1` mutation params introduced by this feature have been removed and replaced:
> - `assign::1` + `for::USER@MACHINE` → `active::USER@MACHINE name::X` (single param, Feature 064)
> - `unclaim::1` → `owner::0` sentinel on the existing `owner::` param (Feature 064)
> - `active::USER@MACHINE` → `assignee::USER@MACHINE name::X` (renamed param + `assignee::0` current-machine sentinel, Feature 065; `active::` is now a REMOVED_TOGGLE)
>
> The unified 32-param set is now 28 active params (4 removed). REMOVED_TOGGLE stubs for `assign`, `for`, `unclaim`, `active` emit migration messages. All other ACs below apply via the updated command surface — ACs referencing `active::USER@MACHINE` now apply via `assignee::USER@MACHINE`. See [064_active_marker_and_owner_redesign.md](064_active_marker_and_owner_redesign.md) and [065_assignee_param_redesign.md](065_assignee_param_redesign.md).

### Scope

- **Purpose**: Make `.accounts` and `.usage` symmetric commands sharing parameters with different defaults, absorbing `.account.unclaim` and `.account.assign` as mutation parameters, and introducing `force::` to bypass ownership enforcement.
- **Responsibility**: Documents the unified parameter set, default differentiation, `cols::` replacement of 15 field toggles, `assignee::` mutation param (marker assign/unassign; `active::` REMOVED in Feature 065), `owner::` mutation param (ownership set/release via `owner::0` sentinel), `force::` bypass param, and removal of `.account.unclaim` and `.account.assign` as standalone commands.
- **In Scope**: Unified parameter set; `.accounts` default profile (local/identity view: `refresh::0`, `touch::0`, `sort::name`); `.usage` default profile (live/quota view: `refresh::1`, `touch::1`, `sort::renew`); `cols::` syntax on `.accounts` replacing 15 individual field toggles (`active::`, `current::`, `sub::`, `tier::`, `expires::`, `email::`, `display_name::`, `host::`, `role::`, `billing::`, `model::`, `uuid::`, `capabilities::`, `org_uuid::`, `org_name::`); `assignee::USER@MACHINE` mutation param for marker assign/unassign (Feature 065 — renames `active::` from Feature 064; `active::` is now a REMOVED_TOGGLE); `owner::` mutation param with `owner::0` sentinel for release and `owner::USER@MACHINE` for set (Feature 064 — `owner::0` replaces former `unclaim::1`); `force::` bypass param (bypasses G8 ownership check when used with `owner::0` or `owner::USER@MACHINE`); `.account.unclaim` (Command 17) and `.account.assign` (Command 16) fully removed (deregistered — not in command listing).
- **Out of Scope**: New quota fetching features; changes to `.account.save`, `.account.use`, `.account.delete`, `.account.limits`, `.account.relogin`, `.account.rotate`, `.account.renewal`, `.account.inspect`, `.model`, or any non-account command; G1–G4 read-side gates (behavior unchanged — `force::` does not bypass them); adding `force::` to `.account.use`/`.account.delete`/`.account.relogin` (G5–G7 bypass → Feature 036 implementation).

### Design

**Symmetric commands.** `.accounts` and `.usage` become two views of the same underlying account data, sharing an identical parameter interface. The only difference is defaults — `.accounts` is a local/identity view (no fetching, no touching, sorted by name), while `.usage` is a live/quota view (fetch + touch enabled, sorted by renewal).

**Unified parameter set:**

| Category | Parameters | `.accounts` default | `.usage` default |
|----------|-----------|---------------------|------------------|
| Selection | `name::`, `count::`, `offset::`, `only_active::`, `only_next::` | —, 0, 0, 0, 0 | —, 0, 0, 0, 0 |
| Data source | `refresh::`, `touch::`, `imodel::`, `effort::`, `live::`, `interval::`, `jitter::` | **0**, **0**, auto, auto, 0, 30, 0 | **1**, **1**, auto, auto, 0, 30, 0 |
| Sort & filter | `sort::`, `desc::`, `prefer::`, `min_5h::`, `min_7d::`, `only_valid::`, `exclude_exhausted::` | **name**, 0, any, 0, 0, 0, 0 | **renew**, 0, any, 0, 0, 0, 0 |
| Columns | `cols::` | **identity set** | **quota set** |
| Output | `format::`, `get::`, `abs::`, `no_color::`, `trace::` | text, —, 0, 0, 0 | text, —, 0, 0, 0 |
| Mutations | `dry::`, `assignee::`, `owner::`, `set_model::`, `force::` | 0, —, —, —, 0 | 0, —, —, —, 0 |

**Default column sets.** `cols::` replaces the 15 individual field toggles on `.accounts`. Both default sets include the `Owner` column — showing the `owner` field from `{name}.json` (`USER@MACHINE` identity or `—` when unowned). This overrides the Feature 036 out-of-scope exclusion for owner display.

The identity set (`.accounts` default) includes: Account, Owner, Active, Current, Sub, Tier, Expires, Email. The quota set (`.usage` default) includes: Status, Account, Owner, 5h Left, 5h Reset, 7d Left, 7d(Son), 7d Reset, Expires, ~Renews, → Next. Both commands support `cols::+col_id` / `cols::-col_id` modifiers to add/remove columns from the default set. `cols::-owner` hides the owner column.

**Command absorption — `.account.unclaim` → `owner::0` (via `unclaim::` removal).** The original Feature 037 introduced `unclaim::1` to absorb `.account.unclaim`. Feature 064 removed `unclaim::1` and replaced it with the `owner::0` sentinel on the existing `owner::` param. `clp .accounts owner::0 name::X` clears the `owner` field in `{name}.json` — identical behavior to the former `clp .accounts unclaim::1 name::X`. G8 ownership gate is evaluated before write. Batch support: when `name::` is omitted, `owner::0` applies to all accounts matching the current filter predicates.

**Command absorption — `.account.assign` → `assignee::USER@MACHINE` (via `assign::` + `for::` removal, then rename).** The original Feature 037 introduced `assign::1` + `for::` for marker assignment. Feature 064 removed both and replaced them with `active::USER@MACHINE name::X`. Feature 065 renamed `active::` → `assignee::` and added `assignee::0` as the current-machine sentinel. `clp .accounts assignee::user1@w003 name::X` writes `_active_w003_user1` = X. When `name::` is absent, `assignee::USER@MACHINE` clears/unassigns the marker for that identity.

**Standalone command removal.** `.account.unclaim` and `.account.assign` are fully removed — deregistered from the command registry. Calling either command produces a generic "unknown command" error. Total visible command count drops from 16 to 14. Assign and unclaim operations use `.accounts assignee::USER@MACHINE` and `.accounts owner::0` exclusively.

**`force::` ownership bypass.** `force::1` bypasses the G8 ownership check when used with `owner::0` or `owner::USER@MACHINE`. It allows any machine/user identity to release or reassign ownership of an account, regardless of whether `current_identity()` matches the stored owner. `force::1` without `owner::` has no effect. `force::1` with `assignee::` has no effect (active marker has no ownership gate). `force::1` does NOT bypass `dry::1` — dry-run preview still applies when both are set.

**`.usage` mutation params.** `.usage` gains the same mutation params (`dry::`, `assignee::`, `owner::`, `set_model::`, `force::`) since both commands share the full parameter set.

### Acceptance Criteria

- **AC-01**: `.accounts` accepts all parameters from the unified set. Unknown parameters exit 1.
- **AC-02**: `.usage` accepts all parameters from the unified set. Unknown parameters exit 1.
- **AC-03**: `.accounts` defaults: `refresh::0`, `touch::0`, `sort::name`, `cols::` = identity set (Account, Owner, Active, Current, Sub, Tier, Expires, Email). No HTTP fetch or subprocess spawn when invoked without explicit `refresh::1` or `touch::1`.
- **AC-04**: `.usage` defaults: `refresh::1`, `touch::1`, `sort::renew`, `cols::` = quota set (Status, Account, Owner, 5h Left, 5h Reset, 7d Left, 7d(Son), 7d Reset, Expires, ~Renews, → Next). Owner column added to default; all other behavior unchanged.
- **AC-05**: `.accounts owner::0 name::X` exits 0 and writes `owner: ""` to `{name}.json` when G8 passes. Credentials NOT touched. Active marker NOT changed. Identical observable behavior to the former `.account.unclaim name::X`.
- **AC-06**: `.accounts owner::0 name::X` when `!is_owned(account)` exits 1 with `"ownership violation: this account is owned by {owner}"`. G8 gate evaluates before `dry::1` check.
- **AC-07**: `.accounts owner::0` (no `name::`) applies ownership release to every account matching current filter predicates (`only_active::`, `only_valid::`, etc.). Each account independently evaluated against G8; non-owned accounts skipped with `"skip"` message (not exit 1).
- **AC-08**: `.accounts assignee::user1@w003 name::X` writes `{credential_store}/_active_w003_user1` = X. Exits 0. `~/.claude/.credentials.json`, `~/.claude.json`, `{name}.json` NOT touched. (Formerly `active::user1@w003 name::X` — Feature 064; `active::` is now REMOVED_TOGGLE — Feature 065.)
- **AC-09**: `.accounts assignee::bob@laptop name::X` writes `{credential_store}/_active_laptop_bob` = X. Sanitization rules identical to former `.account.assign` `for::` param. (Formerly `active::bob@laptop name::X` — Feature 064.)
- **AC-10**: `.accounts assignee::user1@w003` (no `name::`) clears `{credential_store}/_active_w003_user1`. Exits 0. No credentials or `{name}.json` touched. (Formerly `active::user1@w003` — Feature 064.)
- **AC-11**: `.account.unclaim` is fully removed — not registered; calling it produces a generic "unknown command" error.
- **AC-12**: `.account.assign` is fully removed — not registered; calling it produces a generic "unknown command" error.
- **AC-13**: `.accounts` no longer accepts the 15 individual field toggles (`active::`, `current::`, `sub::`, `tier::`, `expires::`, `email::`, `display_name::`, `host::`, `role::`, `billing::`, `model::`, `uuid::`, `capabilities::`, `org_uuid::`, `org_name::`) as `Kind::Bool` field-presence params. These params exit 1 with an error directing to `cols::` syntax. (`active::` was subsequently repurposed as a `Kind::String` mutation param in Feature 064, and is now a REMOVED_TOGGLE since Feature 065 — use `assignee::USER@MACHINE name::X`.)
- **AC-14**: `.accounts cols::+host,-tier` adds host column and removes tier column from the identity default set. Syntax and behavior identical to `.usage cols::` (Feature 029).
- **AC-15**: `.accounts refresh::1` fetches live quota data for all accounts — same algorithm as `.usage`. `.accounts touch::1` activates idle session windows — same algorithm as `.usage`.
- **AC-16**: `.usage owner::0 name::X` clears owner field — identical to `.accounts owner::0 name::X`.
- **AC-17**: `.usage assignee::user1@w003 name::X` writes marker — identical to `.accounts assignee::user1@w003 name::X`. (Formerly `active::user1@w003 name::X` — Feature 064.)
- **AC-18**: `.accounts dry::1 owner::0 name::X` prints `[dry-run] would clear owner of X` and exits 0. No files modified. G8 gate still runs.
- **AC-19**: Owner column visible by default on both `.accounts` and `.usage`. Shows `owner` field from `{name}.json` — `USER@MACHINE` when owned, `—` when unowned (empty or absent). `cols::-owner` hides it.
- **AC-20**: `.accounts owner::0 name::X force::1` clears the owner field even when `current_identity() ≠ stored_owner` — G8 gate is bypassed. Exits 0. Output: `unclaimed X`. G8 enforcement (exit 1 with ownership violation message) is suppressed only when `force::1` is present.
- **AC-21**: `.accounts force::1` without `owner::`, or `.accounts force::1 assignee::user@host name::X`, silently ignores `force::1` — no error, no change to behavior. `force::` is a mutation-scoped bypass with no standalone meaning.
- **AC-22**: `.accounts assign::1` exits 1 with migration message: "REMOVED — use `assignee::USER@MACHINE name::X`". `.accounts unclaim::1` exits 1 with migration message: "REMOVED — use `owner::0 name::X`". `.accounts for::user@host` exits 1 with migration message. `.accounts active::user@host` exits 1 with REMOVED_TOGGLE migration message pointing to `assignee::` (Feature 065).
- **AC-23**: `.accounts owner::user1@w003 name::X,Y,Z` sets ownership for X, Y, and Z in one invocation. Each account evaluated against G8 independently.
- **AC-24**: `.accounts owner::0 name::X,Y,Z` clears ownership for X, Y, and Z. Each account evaluated against G8 independently.

### Features

| File | Relationship |
|------|--------------|
| [003_account_list.md](003_account_list.md) | `.accounts` baseline — account enumeration and per-account block rendering |
| [009_token_usage.md](009_token_usage.md) | `.usage` baseline — quota fetch algorithm |
| [032_account_assign.md](032_account_assign.md) | `.account.assign` behavior originally absorbed as `assign::` + `for::`; now via `assignee::USER@MACHINE` (Feature 065) |
| [036_account_ownership.md](036_account_ownership.md) | `.account.unclaim` behavior absorbed as `owner::0` (former `unclaim::` removed in Feature 064); G8 gate preserved |
| [028_usage_row_filtering.md](028_usage_row_filtering.md) | Row filtering params (`count::`, `offset::`, `only_active::`, etc.) added to `.accounts` |
| [020_usage_sort_strategies.md](020_usage_sort_strategies.md) | Sort strategies (`sort::`, `desc::`, `prefer::`) and footer recommendation added to `.accounts` |
| [024_session_touch.md](024_session_touch.md) | Touch (`touch::`) available on `.accounts` with default `0` |
| [017_token_refresh.md](017_token_refresh.md) | Refresh (`refresh::`) available on `.accounts` with default `0` |
| [029_account_host_metadata.md](029_account_host_metadata.md) | `cols::` syntax shared between `.accounts` and `.usage` |
| [034_explicit_session_model_override.md](034_explicit_session_model_override.md) | `set_model::` available on both `.accounts` and `.usage` |
| [064_active_marker_and_owner_redesign.md](064_active_marker_and_owner_redesign.md) | Feature 064 — `assign::1`/`for::` removed → `active::USER@MACHINE`; `unclaim::1` removed → `owner::0` |
| [065_assignee_param_redesign.md](065_assignee_param_redesign.md) | Feature 065 — `active::` REMOVED → `assignee::USER@MACHINE`; `assignee::0` current-machine sentinel |

### Commands

| File | Relationship |
|------|--------------|
| [cli/command/001_account.md](../cli/command/001_account.md) | `.accounts` — Command 3; `.account.assign` (Command 16) and `.account.unclaim` (Command 17) removed |
| [cli/command/006_usage.md](../cli/command/006_usage.md) | `.usage` — Command 9; gains mutation params |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/063_assignee.md](../cli/param/063_assignee.md) | `assignee::` — current marker assign/unassign param; `assignee::0` = current machine sentinel (Feature 065; replaced `active::` which is now REMOVED) |
| [cli/param/053_for.md](../cli/param/053_for.md) | `for::` — REMOVED (Feature 064); absorbed into `active::` value |
| [cli/param/056_unclaim.md](../cli/param/056_unclaim.md) | `unclaim::` — REMOVED (Feature 064); replaced by `owner::0` |
| [cli/param/057_assign.md](../cli/param/057_assign.md) | `assign::` — REMOVED (Feature 064); replaced by `assignee::USER@MACHINE name::X` (via `active::` which was further REMOVED in Feature 065) |
| [cli/param/058_force.md](../cli/param/058_force.md) | `force::` — bypass G8 ownership check when used with `owner::0` or `owner::USER@MACHINE` |
| [cli/param/062_owner.md](../cli/param/062_owner.md) | `owner::` — `owner::0` sentinel releases ownership; `owner::USER@MACHINE` sets it; batch via comma-list |

### Sources

| File | Relationship |
|------|--------------|
| `src/registry.rs` | Command registration — unified param set for `.accounts` and `.usage`; `assign`, `for`, `unclaim` as REMOVED_TOGGLE stubs; `.account.unclaim` (Command 17) and `.account.assign` (Command 16) fully deregistered |
| `src/commands/accounts.rs` | `.accounts` handler — `assignee::` and `owner::` logic inline; G8 gate for `owner::0` and `owner::USER@MACHINE` paths; `.account.unclaim` and `.account.assign` produce generic "unknown command" error |
| `src/usage/api.rs` | `.usage` handler — gains same mutation param dispatch |

### Tests

| File | Relationship |
|------|--------------|
| `tests/cli/accounts_ft_test.rs` | Integration tests for `.accounts` — absorbs assign and unclaim test cases |
| `tests/cli/account_assign_test.rs` | Integration tests for `.accounts assignee::USER@MACHINE` — verifies marker-only write behavior per AC-08/AC-09/AC-10 |
| `tests/cli/usage_feature_test.rs` | Integration tests for `.usage` — gains mutation param tests |
