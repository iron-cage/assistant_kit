# REMOVED — `next::` Parameter

> **REMOVED:** The `next::` parameter has been removed (Feature 037/038). Passing `next::`
> to `.usage` exits 1 with "next:: parameter has been removed; use sort:: instead".
> The `sort::` parameter now drives both row ordering and the `-> Next` footer recommendation.

Rejection test: `it253_next_param_removed_exit_1` in `tests/cli/usage_test.rs`.
