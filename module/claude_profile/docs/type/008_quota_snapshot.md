# Type: Quota Snapshot

### Scope

- **Purpose**: Define Quota Snapshot — the point-in-time rate-limit measurement carried per account.
- **Responsibility**: Documents what the snapshot carries, its staleness semantics, and which consumers read it.
- **In Scope**: Carried measurements, snapshot nature, staleness, consumer list.
- **Out of Scope**: Fetch/approximation logic (→ [algorithm/006](../algorithm/006_quota_approximation.md)); classification thresholds (→ [algorithm/003](../algorithm/003_quota_status_groups.md)); measurement lifecycle (→ [state_machine/005](../state_machine/005_quota_measurement_lifecycle.md)); boundary hazards (→ [algorithm/011](../algorithm/011_rounding_boundary_classification_hazards.md)).

### Definition

A plain data carrier (no domain behavior of its own) holding one account's rate-limit state as measured at a moment: five-hour window remaining, seven-day window remaining, utilization, billing type, expiry, provider, ownership/occupancy flags. Produced by usage fetch/refresh against the provider API or served from cache; consumed read-only by display, status grouping, and eligibility gating.

A snapshot answers "what was true when measured", never "what is true now" — every consumer must treat it as potentially stale and behave per its own staleness policy.

### Validation

- Window-remaining values are percentages interpreted through the rounding rules of [algorithm/011](../algorithm/011_rounding_boundary_classification_hazards.md) — consumers compare via the shared `five_hour_left`/`seven_day_left` predicates, never raw utilization (see [invariant/011](../invariant/011_shared_predicate_consistency.md)).
- `billing_type == "none"` means no active subscription and must pair with the `result` field when answering subscription questions ([invariant/011](../invariant/011_shared_predicate_consistency.md)).
- A fetch error is carried as an error `result`, not silently as zeros — gates treat errored snapshots as ineligible ([algorithm/004](../algorithm/004_eligibility_gates.md) Gate 4).

### Relationships

Measured for [Account (001)](001_account.md); read by eligibility gates 4–7 ([algorithm/004](../algorithm/004_eligibility_gates.md)), status groups ([algorithm/003](../algorithm/003_quota_status_groups.md)), and sort strategies ([algorithm/007](../algorithm/007_sort_strategies.md)).

### Serialization

Cached per account in the quota cache (see [state_machine/005](../state_machine/005_quota_measurement_lifecycle.md) for cache lifecycle); rendered by `.usage` output formats.
