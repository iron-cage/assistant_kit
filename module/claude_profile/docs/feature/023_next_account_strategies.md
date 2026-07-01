# Feature: Next Account Recommendation Strategies

### Scope

- **Purpose**: Redirect from the removed `next::` parameter to the current `sort::` parameter driving footer recommendations.
- **Responsibility**: Documents migration from three-strategy footer to single-line `sort::`-driven recommendation; lists all fixed bugs in the deprecated feature.
- **In Scope**: `next::` → `sort::` migration mapping; removed strategies; single-line footer change.
- **Out of Scope**: Current sort strategy logic (→ feature/020); current footer format (→ feature/009).

> Superseded by [feature/020_usage_sort_strategies.md](020_usage_sort_strategies.md).

**Migration:**
- `next::renew` → `sort::renew` (default — no change needed)
- `next::endurance` → no replacement (strategy removed)
- `next::drain` → no replacement (strategy removed)
- 3-strategy footer → 1-line footer for the active `sort::` strategy

### Features

| File | Relationship |
|------|--------------|
| [020_usage_sort_strategies.md](020_usage_sort_strategies.md) | Current canonical replacement — `sort::` parameter |
