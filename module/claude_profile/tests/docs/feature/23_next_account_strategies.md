# DEPRECATED — Feature 023 — Next Account Recommendation Strategies

> **DEPRECATED:** `next::` parameter and `NextStrategy` enum removed (Feature 037/038). The
> `sort::` parameter now drives both row ordering and the `-> Next` footer recommendation.
> Surviving test cases have been migrated to `tests/docs/feature/20_usage_sort_strategies.md`.
> Test functions in `src/usage/sort_next_tests.rs` that previously referenced `NextStrategy`
> now reference `SortStrategy` directly.

No active test cases. All coverage lives in `20_usage_sort_strategies.md`.
