# task/claude_profile/

Task registry for the `claude_profile` crate.

## Tasks Index

| ID | Slug | State | Summary |
|----|------|-------|---------|
| 001 | feature_037_accounts_param_unification | 🎯 Verified | Absorb .account.assign/.account.unclaim into .accounts with 32-param unified set and force:: bypass |
| 002 | force_bypass_g5_g6_g7 | 🎯 Verified | Add force::1 bypass to .account.use, .account.delete, .account.relogin for G5–G7 ownership gates |
| 003 | drop_account_rotate_add_usage_rotate | 🎯 Verified | Drop .account.rotate; add rotate::1 to .usage with strategy-driven rotation, G5 gate, dry-run, touch reuse |
| 004 | sort_strategy_simplification | 🎯 Verified | Simplify sort to name/renew/renews, next to renew-only, implement 4-group status partition |

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `readme.md` | Tasks Index — registry of all crate-scoped tasks |
| `decisions.md` | Decision log — open questions and resolved decisions |
| `unverified/` | Tasks pending MAAV verification gate |
| `NNN_slug.md` | Verified task files |
