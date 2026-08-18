# Algorithm: Next-Account Eligibility Gates

### Scope

- **Purpose**: Define the eligibility gate filter applied before next-account recommendation and auto-switch.
- **Responsibility**: Documents all 11 eligibility gates, their skip conditions, and the `gate_ownership` context by call site.
- **In Scope**: `find_first_eligible()` gates 1–6, 9, 10, 11; `extra` closure gates 7–8; `gate_ownership` semantics; `is_owned` definition; `claim_lock` unconditional exclusion (Gate 9); selected-provider unconditional exclusion (Gate 10); Identity tag-filter unconditional exclusion (Gate 11, 📋 planned).
- **Out of Scope**: Positive selection after gating (→ algorithm/005); sort strategies, `reserve` leading sort key (→ algorithm/007); explicit-command `claim_lock` gate G9 (→ state_machine/004).

### Abstract

Filter candidates for next-account recommendation and auto-switch. An account is **skipped** when any gate fires. Only accounts passing all gates are eligible.

### Algorithm

#### Entry Points

- `src/usage/sort_next.rs:24-35` — `find_first_eligible()` (gates 1–6, 9, 10; 11 planned)
- `src/usage/sort_next.rs:59` — `extra` closure passed by `find_next_for_strategy()` (gates 7–8)

#### Gate Table

| # | Gate | Skip condition | Source |
|---|------|---------------|--------|
| 1 | Current | `is_current = true` | `sort_next.rs:27` |
| 2 | Active | `is_active = true` | `sort_next.rs:27` |
| 3 | Occupied | `is_occupied_elsewhere = true` | `sort_next.rs:28` |
| 3b | Cancelled | `billing_type = "none"` | `sort_next.rs:29` |
| 4 | Error | `result = Err(...)` | `sort_next.rs:30` |
| 5 | h-exhausted | `five_hour_left( aq ) <= H_EXHAUSTED_THRESHOLD` (rounded left ≤ 15%; audit-h-exhaustion-drift — formerly raw `utilization >= 85.0`) | `sort_next.rs:47` |
| 6 | Expired | `expires_at_ms / 1000 ≤ now_secs` | `sort_next.rs:31` |
| 7 | Weekly-exhausted | `seven_day_left(aq) ≤ WEEKLY_EXHAUSTION_THRESHOLD` | `sort_next.rs:59` (extra) |
| 8 | Foreign-owned | `is_owned = false AND gate_ownership = true` | `sort_next.rs:59` (extra) |
| 9 | Claim-locked | `claim_lock = true` | `sort_next.rs` — inside `find_first_eligible()` (unconditional, not part of `extra`) |
| 10 | Provider-mismatch | `inference_provider != selected_provider` | `sort_next.rs` — inside `find_first_eligible()` (unconditional, not part of `extra`) |
| 11 | Tag-mismatch 📋 | `NOT (tags ⊇ filter.include AND tags ∩ filter.exclude = ∅)` | planned — inside `find_first_eligible()` (unconditional, not part of `extra`); [feature/076](../feature/076_identity_tag_filter.md) |

#### Gate 8 Context — `gate_ownership` varies by call site

| Call site | `gate_ownership` | Effect |
|---|---|---|
| Footer + display recommendation (`render.rs:241`) | `false` (hardcoded) | Ownership never checked — non-owned accounts can appear as "Next" recommendation |
| `only_next::1` row filter (`api.rs:835`) | `rotate && !force` | Non-owned excluded when auto-switch is active and not forced |
| Auto-switch execution (`api.rs:935`) | `!params.force` | Ownership required unless `force::1` |

Note: the `→ Next` column in the table is a **data column** showing the next renewal/reset event time — it is not a per-row recommendation marker. The footer "Next (strategy):" line is the only recommendation output and uses `gate_ownership=false`. This means the footer can recommend a non-owned account that auto-switch (`rotate::1`) would reject — BUG-320 🟢 Verified.

#### Gate 3 vs Gate 8 — `force::1` scope

Gate 3 (`is_occupied_elsewhere → continue`) fires unconditionally inside `find_first_eligible()` — it is not part of the `extra` predicate controlled by `gate_ownership`. Gate 8 (Foreign-owned) is inside the `extra` predicate and is bypassed when `gate_ownership=false` or when `force::1` sets it to `false`.

An occupied-elsewhere account cannot be selected by `find_next_for_strategy()` under any `force::1` or `gate_ownership` combination. A non-owned account can be selected when `gate_ownership=false` (footer recommendation at `render.rs:241`).

#### Gate 9 Context — unconditional, mirrors Gate 3

Gate 9 (Claim-locked) fires unconditionally inside `find_first_eligible()`, exactly like Gate 3 (Occupied) — it is not part of the `extra` predicate and has no `force::1` bypass at the eligibility layer. `claim_lock` is a caller-imposed absolute exclusion (the caller who set it decided this account must never be auto-selected), not a relative "who may act" concern like ownership — an unattended `rotate::1 force::1` cron invocation must never be able to silently defeat a lock the caller deliberately set.

A claim-locked account cannot be selected by `find_next_for_strategy()` under any `force::1` combination — same absolute-exclusion property as Gate 3.

**Not the same as G9 (explicit-command):** the `claim_lock` field also gates `.account.use` and `.accounts assignee::` target-side via a *separate*, `force::1`-bypassable gate — see G9 in [state_machine/004_ownership_lifecycle.md](../state_machine/004_ownership_lifecycle.md). One field, two enforcement points with different bypass semantics: Gate 9 here (unconditional, automatic-selection path) vs. G9 there (bypassable, named-target path). See [feature/070_account_claim_and_reservation_control.md](../feature/070_account_claim_and_reservation_control.md) for the full picture.

#### Gate 10 Context — unconditional, mirrors Gate 9

Gate 10 (Provider-mismatch) fires unconditionally inside `find_first_eligible()`, exactly like Gate 9 (Claim-locked) — it is not part of the `extra` predicate and has no `force::1` bypass at the eligibility layer. The selected provider (`.provider.select`'s global config value, default `anthropic`) is a single static scalar — never a filter, never derived; only a manual `.provider.select id::` write changes it. An account tagged with a different `inference_provider` is categorically ineligible for rotation, not merely deprioritized: silently rotating into a different provider's account would switch billing/auth context without the user's explicit consent. `force::1` bypasses ownership (Gate 8) and other relative "who may act" concerns; it must never bypass a provider mismatch, since provider selection is a "which provider is active" concern, not an ownership concern.

A provider-mismatched account cannot be selected by `find_next_for_strategy()` under any `force::1` combination — same absolute-exclusion property as Gate 3 and Gate 9. See [feature/072_inference_provider_selection.md](../feature/072_inference_provider_selection.md) for the full picture.

#### Gate 11 Context — unconditional, mirrors Gate 10 *(📋 planned — [feature/076](../feature/076_identity_tag_filter.md))*

Gate 11 (Tag-mismatch) fires unconditionally inside `find_first_eligible()`, after Gate 10 — it is not part of the `extra` predicate and has no `force::1` bypass at the eligibility layer. The current Identity's Tag Filter ([type/004](../type/004_tag_filter.md), stored per [schema/009](../schema/009_identity_filter_json.md)) supplies an `include`/`exclude` tag-set pair; an account whose tag set `T` fails `T ⊇ include ∧ T ∩ exclude = ∅` is categorically ineligible for automatic selection. Same doctrine as Gate 10: `force::1` bypasses "who may act" gates (Gate 8 ownership), never "which pool" gates — an operator's declared fleet partition must not be silently violated by an unattended `rotate::1 force::1` invocation. Absent filter file = permit-all: Gate 11 passes every account, preserving pre-feature behavior exactly.

Two properties distinguish Gate 11 from Gate 10:

- **Per-Identity, not global**: Gate 10 compares against one machine-global config scalar; Gate 11 evaluates the *current Identity's* own filter file — two seats on one fleet can carve different pools.
- **Loud exclusion**: when Gate 11 excludes ≥1 account during a selection pass, `.usage` reports `N excluded by tag filter include=[…] exclude=[…]` — a filter that silently empties the pool is this gate's primary operational hazard ([feature/076](../feature/076_identity_tag_filter.md) AC-13).

A tag-mismatched account cannot be selected by `find_next_for_strategy()` under any `force::1` combination — same absolute-exclusion property as Gates 3, 9, and 10. Explicit `.account.use name::X` is never filtered.

#### `is_owned` Semantics

`is_owned = true` when `owner` field is empty OR matches `current_identity()` (`{user}@{hostname}`). `is_owned = false` when a different machine owns the account. Source: `types.rs:193-195`.

### Features

| File | Relationship |
|------|-------------|
| [feature/039_decision_algorithms.md](../feature/039_decision_algorithms.md) | Table 4 (legacy reference) |
| [feature/036_account_ownership.md](../feature/036_account_ownership.md) | `is_owned` field semantics |
| [feature/061_solo_token_conservation.md](../feature/061_solo_token_conservation.md) | Solo gate (before G1 in fetch/refresh/touch) |
| [feature/070_account_claim_and_reservation_control.md](../feature/070_account_claim_and_reservation_control.md) | Gate 9 (`claim_lock`, unconditional) — full properties table |
| [feature/072_inference_provider_selection.md](../feature/072_inference_provider_selection.md) | Gate 10 (`inference_provider` mismatch, unconditional) — full properties table |
| [feature/075_account_tags.md](../feature/075_account_tags.md) | Account-side `tags` set Gate 11 evaluates |
| [feature/076_identity_tag_filter.md](../feature/076_identity_tag_filter.md) | Gate 11 (Identity tag-filter mismatch, unconditional, 📋 planned) — filter semantics and loud exclusion reporting |

### Algorithms

| File | Relationship |
|------|-------------|
| [algorithm/003](003_quota_status_groups.md) | Status groups — same 5h/7d thresholds; cancelled gate parallel |
| [algorithm/005](005_next_account_selection.md) | Positive selection uses these gates |

### Invariants

| File | Relationship |
|------|-------------|
| [invariant/011](../invariant/011_shared_predicate_consistency.md) | `billing_type=="none"` must pair with `result` when answering "no active subscription" (BUG-332) — this file's Gate 3b is a correctly-scoped exception |
