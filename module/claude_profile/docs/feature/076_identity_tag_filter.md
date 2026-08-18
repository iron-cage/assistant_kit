# Feature: Identity Tag Filter

### Scope

- **Purpose**: Let each Identity (`user@host`) declare which slice of the account fleet its automatic selection may draw from — an include/exclude pair of tag sets — so that rotation on a given machine can be pinned to (or steered away from) named pools without touching any account, and without ever blocking an explicit `.account.use`.
- **Responsibility**: Documents the per-Identity Tag Filter file in the credential store (`_filter_{hostname}_{user}`, [schema/009](../schema/009_identity_filter_json.md)); the `.identity.filter` get/set/clear command; the `.identities` listing command; Gate 11 — the unconditional tag-mismatch eligibility gate in `find_first_eligible()`; the write-time contradiction rejection and typo guard; and the loud exclusion reporting on selection paths.
- **In Scope**: Filter structure and eligibility predicate `T ⊇ include ∧ T ∩ exclude = ∅` per [type/004](../type/004_tag_filter.md); `_filter_{hostname}_{user}` file format and sibling-to-active-marker derivation; `.identity.filter` (get mode, set via `include::`/`exclude::`, delete via `clear::1`, cross-identity targeting via `identity::`); `.identities` fleet-wide identity listing; Gate 11 in `find_first_eligible()` (unconditional, after Gate 10); `include ∩ exclude` write rejection; zero-tagged-match typo warning; `N excluded by tag filter` reporting on `.usage` selection paths.
- **Out of Scope**: Tag value rules, normalization, and account-side tag storage — that is [Feature 075](075_account_tags.md) and [type/003](../type/003_tag.md). Explicit switching — `.account.use name::X` is never filtered (naming an account is explicit intent), exactly as Gate 10 binds only automatic selection. Any per-account "who may act" control — ownership (G5–G8, [Feature 036](036_account_ownership.md)) and `claim_lock` (Gate 9/G9, [Feature 070](070_account_claim_and_reservation_control.md)) already own that; a filter is a "which pool" concern.

### Design

**Why per-Identity, not per-machine or global:** the acting unit in this system is already `user@host` — active markers ([Feature 025](025_per_machine_active_marker.md)), `owner` fields, and `assignee::` all use it. Two users on one host (or one user across hosts) legitimately want different pools. The filter attaches to the same Identity value ([type/002](../type/002_identity.md)) the rest of the system keys on.

**Why store-resident:** filters live in the credential store as one file per Identity — sibling convention to the active marker ([schema/005](../schema/005_active_marker.md)), same filename derivation and sanitization — so they sync across machines with the store and are centrally administrable (`identity::` lets an operator set another seat's filter). Unlike `_active_*` markers (machine-local, gitignored), filter files are meant to sync; the filename prefix `_filter_` deliberately does not match the `_active_*` ignore pattern. Format: [schema/009](../schema/009_identity_filter_json.md).

**Why include AND exclude:** include-only cannot express "anything but the CI pool" without enumerating every other tag; exclude-only cannot express "only the Kimi pool". The pair covers both directions; the predicate over an account's tag set `T` is `T ⊇ include ∧ T ∩ exclude = ∅`. Empty include = no requirement; empty exclude = nothing blocked; absent file = permit-all — today's behavior exactly, giving a zero-migration adoption path.

**Why Gate 11 is unconditional (no `force::1`):** same doctrine as Gate 10 ([Feature 072](072_inference_provider_selection.md)) — `force::1` bypasses "who may act" gates (ownership G8, claim-lock G9 on explicit targets), never "which pool" gates. A filter mismatch means the account is outside the pool this Identity selects from; forcing rotation into it would silently violate the operator's declared partition. The escape hatch is explicit: `.account.use name::X` names the account and is never filtered.

**Why loud exclusion reporting:** a filter that silently empties the eligible pool is the primary operational hazard of this design — rotation would report "no eligible account" with no hint why. Two mitigations: (1) at write time, an `include` set matching zero currently-tagged accounts succeeds but warns (typo guard); (2) at selection time, when Gate 11 excluded ≥1 account, `.usage` reports `N excluded by tag filter include=[…] exclude=[…]` so the cause is visible in the output that surprised the operator.

**Why contradiction is rejected at write:** `include ∩ exclude ≠ ∅` makes the predicate unsatisfiable for any account carrying the overlapping tag — always a mistake, never a valid intent. Rejecting at write (exit 1 naming the overlap) beats debugging an empty pool later.

**`.identities` listing:** one row per Identity observed anywhere in the store — active markers (`_active_*`), filter files (`_filter_*`), and `owner` fields in `{name}.json` — showing each Identity's active account (or `—`), owned-account count, and filter sets. This is the fleet-seat overview the `user@host` concept never had.

**Gate ordering:** Gate 11 runs after Gate 10 inside `find_first_eligible()` — both unconditional, both "which pool" gates; provider partitions by credential origin, tags partition by operator intent. They apply independently ([algorithm/004](../algorithm/004_eligibility_gates.md)).

### Acceptance Criteria

- **AC-01**: `.identity.filter` with no operation params prints the current Identity's filter; with no filter file present it prints `include=[] exclude=[] (permit-all)` and exits 0.
- **AC-02**: `.identity.filter include::a,b` writes `{"include": ["a", "b"], "exclude": []}` (sorted, deduplicated) to `_filter_{hostname}_{user}` for the current Identity. Exits 0.
- **AC-03**: `include::` and `exclude::` may be set in one invocation; each given set fully replaces that side of any existing filter.
- **AC-04**: A write where `include ∩ exclude ≠ ∅` exits 1 naming the overlapping tags; nothing is written.
- **AC-05**: A tag in either set failing [type/003](../type/003_tag.md) validation (after lowercasing) exits 1 naming the offending tag; nothing is written.
- **AC-06**: `.identity.filter clear::1` deletes the Identity's filter file; when no file exists it is an idempotent success (exit 0). `clear::1` combined with `include::` or `exclude::` exits 1.
- **AC-07**: `identity::USER@MACHINE` targets that Identity's filter for get, set, and clear; omitted, the current Identity (`$USER@$HOSTNAME`, same resolution as [schema/005](../schema/005_active_marker.md)) is targeted.
- **AC-08**: A successful write whose `include` set matches zero currently-tagged accounts prints a warning to stderr naming the unmatched tags (typo guard); exit remains 0.
- **AC-09**: Gate 11 — automatic selection (`rotate::1`, auto-switch, footer `Next` recommendation) never selects an account failing `T ⊇ include ∧ T ∩ exclude = ∅` against the current Identity's filter, under any `force::1` combination.
- **AC-10**: `.account.use name::X` succeeds regardless of any filter — explicit selection is never filtered.
- **AC-11**: With no filter file for the current Identity, selection behavior is byte-identical to pre-feature behavior (permit-all, zero migration).
- **AC-12**: An untagged account (`tags` absent/empty) fails any non-empty `include` and trivially passes any `exclude`.
- **AC-13**: When Gate 11 excluded ≥1 account during a selection pass, `.usage` output includes `N excluded by tag filter include=[…] exclude=[…]`; when it excluded none, no such line appears.
- **AC-14**: `.identities` lists one row per Identity observed in active markers, filter files, or `owner` fields — columns: Identity, Active (account or `—`), Owned (count), Include, Exclude. Empty result prints `(no identities)` and exits 0.
- **AC-15**: `.identities format::json` and `.identity.filter format::json` emit the equivalent structured data (`text` default; `json` accepted; other formats exit 1).
- **AC-16**: Filter files are store-resident and named `_filter_{hostname}_{user}` with the same sanitization as active markers; the `_active_*` ignore convention does not match them.

### Bugs

_(none yet)_

### Domain Types

| File | Relationship |
|------|--------------|
| [type/004_tag_filter.md](../type/004_tag_filter.md) | Authoritative Tag Filter contract — structure, predicate, defaults, validation this feature implements |
| [type/002_identity.md](../type/002_identity.md) | Identity — the `user@host` owner of each filter |
| [type/003_tag.md](../type/003_tag.md) | Tag — the value type both filter sets contain |

### Features

| File | Relationship |
|------|--------------|
| [075_account_tags.md](075_account_tags.md) | Companion feature — provides the account-side tag sets this filter evaluates |
| [072_inference_provider_selection.md](072_inference_provider_selection.md) | Gate 10 — the unconditional "which pool" gate doctrine Gate 11 mirrors; both bind automatic selection only |
| [070_account_claim_and_reservation_control.md](070_account_claim_and_reservation_control.md) | Gate 9 — the first unconditional eligibility gate; contrast: `claim_lock` is per-account "who may act", filters are per-Identity "which pool" |
| [025_per_machine_active_marker.md](025_per_machine_active_marker.md) | `_active_{hostname}_{user}` — the sibling per-Identity store file whose naming/sanitization convention filter files reuse |
| [038_usage_strategy_rotate.md](038_usage_strategy_rotate.md) | `rotate::1` — the selection path Gate 11 binds, via `find_next_for_strategy()`/`find_first_eligible()` |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/085_include.md](../cli/param/085_include.md) | `include::` — tags an account must all carry |
| [cli/param/086_exclude.md](../cli/param/086_exclude.md) | `exclude::` — tags an account must carry none of |
| [cli/param/087_identity.md](../cli/param/087_identity.md) | `identity::` — target Identity (default: current `$USER@$HOSTNAME`) |
| [cli/param/051_clear.md](../cli/param/051_clear.md) | `clear::` — delete the Identity's filter file |
| [cli/param/002_format.md](../cli/param/002_format.md) | `format::` — text/json on `.identities` and `.identity.filter` get mode |

### Commands

| File | Relationship |
|------|--------------|
| [cli/command/011_identity.md](../cli/command/011_identity.md) | `.identities` (fleet-seat listing) and `.identity.filter` (get/set/clear) — full command specifications |
| [cli/command/006_usage.md](../cli/command/006_usage.md) | `.usage` — Gate 11 binds its `rotate::1`/recommendation path; loud exclusion reporting lands in its output |

### Algorithm Docs

| File | Relationship |
|------|--------------|
| [algorithm/004_eligibility_gates.md](../algorithm/004_eligibility_gates.md) | Gate 11 — unconditional tag-mismatch exclusion inside `find_first_eligible()`, after Gate 10 |

### Schema

| File | Relationship |
|------|--------------|
| [schema/009_identity_filter_json.md](../schema/009_identity_filter_json.md) | `_filter_{hostname}_{user}` file — name derivation, JSON shape, absent-file semantics |
| [schema/005_active_marker.md](../schema/005_active_marker.md) | Active marker — the sibling convention (filename derivation, sanitization) filter files follow |

### Sources

*(planned locations — implementation pending)*

| File | Role |
|------|------|
| `../claude_profile_core/src/account/filter.rs` | Filter file read/write, predicate evaluation, filename derivation (new) |
| `src/commands/identity.rs` | `.identities` and `.identity.filter` command routines (new) |
| `src/usage/sort_next.rs` | Gate 11 in `find_first_eligible()`; exclusion counting for loud reporting |
| `src/registry.rs` | Command and parameter registration |

### Tests

| File | Role |
|------|------|
| [tests/docs/feature/076_identity_tag_filter.md](../../tests/docs/feature/076_identity_tag_filter.md) | FT-level AC coverage plan |
| [tests/docs/cli/command/23_identities.md](../../tests/docs/cli/command/23_identities.md) | `.identities` integration test cases |
| [tests/docs/cli/command/24_identity_filter.md](../../tests/docs/cli/command/24_identity_filter.md) | `.identity.filter` integration test cases |
| `tests/cli/identity_filter_test.rs`, `tests/usage/sort_next_tests_b.rs` | Planned integration test implementations (Gate 11 cases join the existing gate test file) |
