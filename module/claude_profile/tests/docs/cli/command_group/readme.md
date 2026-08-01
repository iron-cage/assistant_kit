# Command Group Tests

### Scope

- **Purpose**: Document structural-equivalence tests for clp command groups.
- **Responsibility**: Index of per-command-group test files verifying shared-handler and shared-parameter-set claims.
- **In Scope**: All clp command group test files (0 — no qualifying groups).
- **Out of Scope**: Per-command tests (→ `command/`), per-parameter-group interaction tests (→ `param_group/`).

Per-group structural-equivalence test indices for `clp`. See [command_group/readme.md](../../../../docs/cli/command_group/readme.md) for specification.

### Responsibility Table

| Name | Purpose | Status |
|------|---------|--------|

**Total:** 0 group test files. `docs/cli/command_group/readme.md` documents zero qualifying command_group pairs for `clp` — every one of the 15 live commands registered in `src/registry.rs` is dispatched by a distinct handler function, so no pair can satisfy the entity's same-handler membership test. There is no structural-equivalence claim to test.

The nearest evaluated candidate, `.accounts` / `.usage`, is already covered by existing integration tests that exercise their shared `owner_dispatch::*` mutation helpers independently per command (not as a claimed equivalence) — see `tests/cli/accounts_ft_test.rs` and `tests/cli/usage_feature_test.rs`, referenced from [`docs/feature/037_accounts_usage_param_unification.md`](../../../../docs/feature/037_accounts_usage_param_unification.md) § Tests. No new equivalence test is warranted since no equivalence claim exists to verify.
