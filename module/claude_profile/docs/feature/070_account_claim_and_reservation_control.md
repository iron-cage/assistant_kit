# Feature: Account Claim And Reservation Control

### Scope

- **Purpose**: Give a caller two independent, non-ownership account-management flags — `claim_lock` ("not allowed to be taken, but still usable for quota/refresh/touch") and `reserve` ("usable, but deprioritized for automatic rotation unless nothing else is left") — closing a gap where `owner`/`assignee` alone cannot express either semantic.
- **Responsibility**: Documents the `claim_lock` and `reserve` fields in `{name}.json`; the `lock::`/`reserve::` mutation params on `.accounts`/`.usage`; Gate 9 (unconditional eligibility exclusion for `claim_lock`, inside `find_first_eligible()`); G9 (force::1-bypassable explicit-command gate for `claim_lock`, on `.account.use` and `.accounts assignee::` target-side); the `reserve` leading sort key in `find_next_for_strategy()`; and the consolidated properties table covering every attribute that currently governs account selection behavior.
- **In Scope**: `claim_lock: bool` and `reserve: bool` fields in `{name}.json`; `lock::`/`reserve::` mutation params (comma-list batch, `dry::1` preview, ungated writes); Gate 9 inside `find_first_eligible()` (unconditional, no `force::1` bypass — mirrors Gate 3's scope); G9 on `.account.use` direct target and `.accounts assignee::` target-side (force::1-bypassable — mirrors G5–G8's pattern); `reserve` as a leading sort key prepended to all three strategies (name/renew/renews) in `find_next_for_strategy()`; default-visible lock display — 🔒 account-name suffix in `.usage` text/TSV (`(locked)` under `no_color::1`/`format::plain`), `"claim_lock"` in `.usage format::json`, presence-driven `Lock: yes` text line + 🔒 Account-cell suffix + `"claim_lock"`/`"reserve"` JSON fields in `.accounts`.
- **Out of Scope**: Ownership-gating of `lock::`/`reserve::` writes themselves — no concrete need identified; any caller may set either flag on any account regardless of `owner`, mirroring `assignee::`'s ungated write path rather than `owner::`'s G8-guarded one (deferred until a real requirement emerges). `claim_lock` gating of `.account.delete` (G6), `.account.relogin` (G7), or `.accounts owner::` (G8) — deliberately excluded; `claim_lock` covers only claim-type operations (becoming the active account), not destructive or ownership-transfer operations, which remain governed solely by the existing ownership gates ([036_account_ownership.md](036_account_ownership.md)). Any change to `owner` or `assignee` semantics — both are untouched by this feature.

### Design

**Why owner/assignee aren't enough:** `owner` answers "who may mutate this account's credentials" and `assignee`/`is_active` answers "which account is currently the designated active one." Neither can express "keep this account fully readable and refreshable, but never let it become the active account" (an owned, unoccupied account is still fully eligible for auto-switch) — nor "prefer other accounts during rotation, but fall back to this one when nothing else qualifies" (there is no sort-only signal separate from the hard eligibility gates). `claim_lock` and `reserve` fill these two gaps as new, independent `{name}.json` fields, orthogonal to `owner`/`assignee`.

**Full picture — every attribute controlling account selection behavior:**

| Property | Type | Storage | Purpose | Set via | Governs |
|---|---|---|---|---|---|
| `owner` | `string` | `{name}.json` | Declares which `USER@MACHINE` may mutate this account's credentials | `.accounts owner::` | G5–G8 (mutation gates); G1/G1b/G2/G4 (non-owned → cache-only read path); Gate 8 (eligibility, `force::1`-bypassable) |
| assignee (marker) | `string`, transient file | `_active_{machine}_{user}` | Declares which account is the "active" one for a given machine+user | `.accounts assignee::` | Feeds computed `is_active`; **new:** G9 target-side check |
| `is_current` | `bool` (computed) | n/a — derived at read time | Is this account the live session's account on THIS machine right now | n/a | Gate 1 (eligibility, unconditional) |
| `is_active` | `bool` (computed) | n/a — derived from marker | Is this account the assignee-marked active one (any machine) | n/a | Gate 2 (eligibility, unconditional) |
| `is_occupied_elsewhere` | `bool` (computed) | n/a — derived (`is_active && !is_current`) | Is this account active on a DIFFERENT machine | n/a | Gate 3 (eligibility, unconditional, no `force::1` bypass); G1b (read path) |
| **`claim_lock`** *(new)* | `bool` | `{name}.json` | Self-imposed "not allowed to be taken" flag; read/quota ops unaffected | `.accounts lock::` | Gate 9 (eligibility, unconditional, no `force::1` bypass); G9 (explicit-command, `force::1`-bypassable) |
| **`reserve`** *(new)* | `bool` | `{name}.json` | Self-imposed "deprioritize for rotation unless nothing else is left" flag | `.accounts reserve::` | Leading sort key in `find_next_for_strategy()` — soft, not a gate |

**`claim_lock` — two enforcement points, one field:** "Not allowed to be taken" spans both the *automatic* selection path and the *explicit* named-target path, so one field is enforced at two call-site families with different bypass semantics:

- **Gate 9 (eligibility, algorithm/004)** — lives inside `find_first_eligible()`, the shared filter behind footer recommendation, `.usage rotate::1`, and auto-switch candidate selection ([feature/038_usage_strategy_rotate.md](038_usage_strategy_rotate.md) — "the recommended account and the switched-to account are always the same account," since both call `find_next_for_strategy()`). Gate 9 fires unconditionally, at the same tier as Gate 3 (Occupied) — never bypassed by `force::1`. Rationale: `claim_lock` is a self-imposed absolute exclusion, not a relative "who may act" concern like ownership; an unattended `rotate::1 force::1` cron invocation must never be able to silently defeat a lock the caller deliberately set.
- **G9 (explicit-command, state_machine/004)** — applied to `.account.use` (direct named target) and `.accounts assignee::` (target-side `name::`). `force::1`-bypassable, mirroring G5–G8's existing pattern for deliberate named overrides. `assignee::` is gated too, not just `.account.use`, because it is a second, independent path to "becoming the designated active account" via the marker file — leaving it ungated would let a locked account be made active anyway, defeating Gate 9's purpose for auto-switch (auto-switch consults `is_active`, which `assignee::` controls directly).

**`reserve` — sort key, not a gate:** Prepended as a leading key to all three strategies inside `sort_indices()`'s comparator (`src/usage/sort.rs`), which `find_next_for_strategy()`'s Step 2 delegates to: `(reserve, <strategy key>)`. Non-reserved accounts (`reserve=false`) always sort before reserved ones; ordering within each group is unchanged. No gate is added — a reserved account still passes Gates 1–9 like any other candidate, and the existing "walk from position 0, first eligible wins, else `None`" logic is untouched. Because reserved accounts sort last, that walk naturally yields "only picked when nothing else eligible" without any conditional logic beyond the sort key itself. See [algorithm/007_sort_strategies.md](../algorithm/007_sort_strategies.md).

**Read/quota operations are never gated by either flag:** G1 (fetch), G1b (occupied-elsewhere fetch), G2 (refresh predicate), G4 (touch) are untouched by `claim_lock` or `reserve` — both flags govern only *becoming the active account*, never *being read*. This is what makes `claim_lock` different from `owner`: a non-owned account already falls back to cache-only reads (G1), but a claim-locked account keeps fetching live data normally.

**Ungated writes:** Unlike `owner::` (G8-guarded), `lock::`/`reserve::` writes are not gated by ownership — any caller may set either flag on any account, mirroring `assignee::`'s ungated write path. `force::1` therefore has no effect on the `lock::`/`reserve::` write itself; it only affects whether G9 honors an existing `claim_lock` at `.account.use`/`assignee::` time.

**Batch and dry-run:** Both params follow `owner::`'s comma-list batch pattern (`name::X,Y,Z`; absent `name::` applies to the current filtered set) and support `dry::1` preview. Because neither write is gated, batch operations always succeed for every matched account — there is no per-account rejection path to report.

**Lock visibility — default-visible, name-cell suffix:** A lock that gates selection must be visible in the default views, not behind an opt-in column — otherwise a locked account looks identical to a free one until a switch mysteriously refuses it. `.usage` text and TSV append ` 🔒` to the account-name cell via `with_lock_marker()` (`src/usage/format.rs`); `.accounts` text emits a presence-driven `Lock:    yes` field line (like `Tags:` — no `Lock: no` noise on unlocked accounts) and the table appends the same 🔒 to the Account cell. Suffix rather than prefix or a flag-column glyph: the name cell keeps `starts_with`-style machine matching intact for TSV consumers (same convention as the cache-fallback reason suffix), and the flag column stays single-purpose — a locked *current* account shows both `✓` and 🔒 instead of one masking the other. Under `no_color::1`/`format::plain` the marker renders as `(locked)`. JSON carries the raw booleans instead of the display marker: `"claim_lock"` in `.usage format::json` (both Ok and Err rows) and `"claim_lock"`/`"reserve"` in `.accounts format::json` — these were the only `Account` fields previously absent from the `.accounts` object.

### Acceptance Criteria

- **AC-01**: `clp .accounts lock::1 name::X` writes `claim_lock: true` to `X.json`; exits 0. No credential files modified.
- **AC-02**: `clp .accounts lock::0 name::X` writes `claim_lock: false` to `X.json`; exits 0.
- **AC-03**: `clp .accounts lock::1` (no `name::`) batch-writes `claim_lock: true` to every account in the current filtered set; exits 0.
- **AC-04**: `clp .accounts lock::1 name::X,Y,Z` batch-writes `claim_lock: true` to `X`, `Y`, `Z` only; exits 0.
- **AC-05**: An account with `claim_lock: true` never appears as the footer "Next (strategy):" recommendation — `find_next_for_strategy()` skips it via Gate 9 regardless of `gate_ownership` or `force::1`.
- **AC-06**: `clp .usage rotate::1` never switches to an account with `claim_lock: true` — same shared `find_next_for_strategy()` call as AC-05, confirming the footer recommendation and the rotate target are always the same account (feature/038 invariant) even in the presence of a lock.
- **AC-07**: `clp .usage rotate::1 force::1` still never switches to an account with `claim_lock: true` — Gate 9 has no `force::1` bypass, unlike Gate 8 (Foreign-owned).
- **AC-08**: `clp .account.use name::X` when `X.claim_lock == true` exits 1 with a claim-lock violation message; `switch_account()` is not called.
- **AC-09**: `clp .account.use name::X force::1` when `X.claim_lock == true` bypasses G9, proceeds to `switch_account()`, exits 0.
- **AC-10**: `clp .accounts assignee::USER@MACHINE name::X` when `X.claim_lock == true` exits 1 with a claim-lock violation message; the `_active_{machine}_{user}` marker is not written.
- **AC-11**: `clp .accounts assignee::USER@MACHINE name::X force::1` when `X.claim_lock == true` bypasses G9, writes the marker, exits 0.
- **AC-12**: An account with `claim_lock: true` still returns live (non-cached) quota data via `.usage` — G1/G1b/G2/G4 are unaffected by `claim_lock`; refresh and touch subprocess behavior is unchanged from an unlocked, owned account.
- **AC-13**: `clp .accounts reserve::1 name::X` writes `reserve: true` to `X.json`; exits 0.
- **AC-14**: In `find_next_for_strategy()`, given two otherwise-equal-priority eligible accounts A (`reserve: false`) and B (`reserve: true`), A is always selected before B regardless of strategy (name/renew/renews).
- **AC-15**: An account with `reserve: true` IS selected by `find_next_for_strategy()` when it is the only remaining eligible candidate (all non-reserved accounts are gated out or exhausted) — reserve deprioritizes, it does not exclude.
- **AC-16**: `clp .accounts lock::1 name::X dry::1` / `clp .accounts reserve::1 name::X dry::1` preview the write (`[dry-run]` message) without modifying `X.json`.
- **AC-17**: `clp .accounts lock::1 name::X` and `clp .accounts reserve::1 name::X` succeed regardless of `X`'s `owner` field — neither write is gated by ownership (no G8-style check).
- **AC-18**: An account with `claim_lock: true` is visibly marked in every default `.usage` view with no extra params: text and `format::tsv` append ` 🔒` to the account-name cell (suffix — name-prefix matching stays intact); `no_color::1`/`format::plain` render it as `(locked)` with no 🔒 leak; `format::json` includes `"claim_lock": true`/`false` on every row (Ok and Err alike). Unlocked accounts are byte-identical to pre-Feature-070 output.
- **AC-19**: `clp .accounts` marks locked accounts in all three formats: default text emits a presence-driven `Lock:    yes` line only for locked accounts (no `Lock: no` on others); `format::table` appends ` 🔒` to the Account cell; `format::json` includes `"claim_lock"` and `"reserve"` booleans on every account object.

### Bugs

| ID | Summary | Status |
|----|---------|--------|
| *(none)* | | |

### Features

| File | Relationship |
|------|--------------|
| [020_usage_sort_strategies.md](020_usage_sort_strategies.md) | Existing strategy sort this feature prepends a leading `reserve` key to |
| [036_account_ownership.md](036_account_ownership.md) | Ownership model (`owner`/`is_owned`, G1–G8) — contrast: `claim_lock`/`reserve` are orthogonal, non-ownership fields |
| [037_accounts_usage_param_unification.md](037_accounts_usage_param_unification.md) | Unified param set precedent — `lock::`/`reserve::` follow the same `.accounts`/`.usage` availability as `owner::`/`assignee::` |
| [038_usage_strategy_rotate.md](038_usage_strategy_rotate.md) | `rotate::1` shares `find_next_for_strategy()` with the footer recommendation — the invariant Gate 9 must preserve |
| [063_explicit_ownership_claim.md](063_explicit_ownership_claim.md) | `owner::` — structural precedent for a mutation param with comma-list batch and `dry::1` |
| [065_assignee_param_redesign.md](065_assignee_param_redesign.md) | `assignee::` — structural precedent for an ungated write path (`force::` silently without effect) |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/067_lock.md](../cli/param/067_lock.md) | `lock::` — sets/clears `claim_lock` |
| [cli/param/068_reserve.md](../cli/param/068_reserve.md) | `reserve::` — sets/clears `reserve` |
| [cli/param/001_name.md](../cli/param/001_name.md) | `name::` — single target or comma-list batch |
| [cli/param/004_dry.md](../cli/param/004_dry.md) | `dry::` — preview without writing |
| [cli/param/058_force.md](../cli/param/058_force.md) | `force::` — bypasses G9 only; never bypasses Gate 9; no effect on `lock::`/`reserve::` writes |
| [cli/param/062_owner.md](../cli/param/062_owner.md) | `owner::` — independent field; contrast for the ungated-write design choice |
| [cli/param/047_no_color.md](../cli/param/047_no_color.md) | `no_color::` — renders the 🔒 name-cell marker as `(locked)` |

### Commands

| File | Relationship |
|------|--------------|
| [cli/command/001_account.md](../cli/command/001_account.md) | `.accounts` — primary host for `lock::`/`reserve::` mutation |

### Algorithm Docs

| File | Relationship |
|------|--------------|
| [algorithm/004_eligibility_gates.md](../algorithm/004_eligibility_gates.md) | Gate 9 — unconditional `claim_lock` exclusion inside `find_first_eligible()` |
| [algorithm/005_next_account_selection.md](../algorithm/005_next_account_selection.md) | Positive selection — `reserve` leading key participates in Step 2 sort |
| [algorithm/007_sort_strategies.md](../algorithm/007_sort_strategies.md) | `reserve` leading sort key across all three strategies |

### Schema

| File | Relationship |
|------|--------------|
| [schema/002_account_json.md](../schema/002_account_json.md) | `claim_lock` and `reserve` fields in `{name}.json` |

### State Machines

| File | Relationship |
|------|--------------|
| [state_machine/004_ownership_lifecycle.md](../state_machine/004_ownership_lifecycle.md) | G9 — `force::1`-bypassable explicit-command gate on `.account.use`/`assignee::` target |

### Sources

| File | Relationship |
|------|--------------|
| `src/usage/types.rs` | `AccountQuota` — new `claim_lock: bool`, `reserve: bool` fields, populated alongside existing `is_owned` |
| `src/usage/sort_next.rs` | `find_first_eligible()` — new Gate 9 check; `find_next_for_strategy()` — delegates to `sort_indices()` (`sort.rs`) for the `reserve` leading sort key ahead of the existing strategy comparator |
| `src/usage/sort.rs` | `sort_indices()` — new `reserve` leading sort key in the comparator for all three strategies; the actual implementation `find_next_for_strategy()` delegates to |
| `claude_profile_core/src/account/` | `Account` struct — new `claim_lock`/`reserve` fields; new `write_claim_lock()`/`write_reserve()` functions mirroring `write_owner()` (fields are written directly, not threaded through `save()`'s parameter list) |
| `src/commands/accounts.rs` | New `lock::`/`reserve::` mutation dispatch (comma-list batch, `dry::1`), alongside existing `owner::`/`assignee::` dispatch; new G9 check in the `assignee::` target-side path |
| `src/commands/account_ops.rs` | `account_use_routine()` — new G9 `claim_lock` check alongside the existing G5 ownership check |
| `src/usage/format.rs` | `with_lock_marker()` — appends the ` 🔒` suffix to an account-name cell when `claim_lock` is set |
| `src/usage/render.rs` | Text renderer — name-cell 🔒 via `with_lock_marker()` (Ok and Err rows); `apply_no_color()` maps 🔒 → `(locked)` |
| `src/usage/render_tsv.rs` | TSV renderer — name-cell 🔒 via `with_lock_marker()` (suffix keeps name-prefix matching intact) |
| `src/usage/render_json.rs` | `.usage` JSON — `"claim_lock"` boolean on every row (Ok and Err arms) |
| `src/commands/accounts_render.rs` | `.accounts` renderers — presence-driven `Lock:    yes` text line; table Account-cell 🔒 suffix; `"claim_lock"`/`"reserve"` JSON fields |

### Tests

| File | Relationship |
|------|--------------|
| `tests/cli/account_claim_lock_reserve_test.rs` | t01–t17 — lock::/reserve:: set/clear/batch, Gate 9 (footer + rotate, force-unbypassable), G9 (.account.use + assignee:: on both .accounts/.usage, force-bypassable), reserve sort/fallback, ungated write, read-side non-effect, dry-run preview; t18 — AC-18 `.usage` lock marker across text/TSV/no_color/JSON; t19 — AC-19 `.accounts` lock visibility across text/table/JSON |
| `tests/usage/format_tests.rs` | `test_with_lock_marker_suffix_only_when_locked` — 🔒 suffix appended only when `claim_lock` is set; unlocked name byte-identical |
