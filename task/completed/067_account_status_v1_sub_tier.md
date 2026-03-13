# TSK-067: Fix `.account.status` v::1 Sub/Tier/Email/Org spec compliance

## Goal

Bring `.account.status` v::1 into full spec compliance per FR-16 (spec.md line 283):
add `Sub:`, `Tier:`, `Email:`, `Org:` to `status_active` output and `Sub:`, `Tier:` to
`status_named` output at verbosity level 1.

## Motivation

`spec.md` FR-16 line 283 explicitly requires `subscriptionType` and `rateLimitTier`
at `v::1`. Both `status_active` and `status_named` omitted these fields entirely.
Additionally, `status_active` omitted `Email:` and `Org:` at `v::1` despite `status_named`
already showing them correctly. This gap was carried forward from Plan 002.

## In Scope

- `src/commands.rs`: `status_active` — add Sub/Tier from live `.credentials.json`,
  Email/Org from `.claude.json` at `v::1+`; update v::1 and v::2 format strings
- `src/commands.rs`: `status_named` — add Sub/Tier from account struct at `v::1+`;
  update v::1 and v::2 format strings
- `tests/integration/account_list_status_test.rs`: add `astat11`
- `tests/integration/account_status_name_test.rs`: add `astname11`, `astname12`
- `docs/cli/commands.md`: update `.account.status` v::1 examples
- `docs/cli/testing/command/account_status.md`: add IT-21..IT-23
- `spec.md`: update FR-16 conformance row test references

## Validation List

- [x] `astat11` passes: active path v::1 shows Sub/Tier/Email/Org
- [x] `astname11` passes: named active v::1 shows Sub/Tier
- [x] `astname12` passes: named non-active v::1 shows own Sub/Tier (not active's)
- [x] All 20 prior astat + astname tests pass unmodified
- [x] `status_active` ≤80 lines after change
- [x] `status_named` ≤80 lines after change
- [x] Output field order: Account, Token, Sub, Tier, [Expires at v::2], Email, Org
- [x] `w3 .test level::3` on `claude_profile`: 276/276 passed, clippy clean

## Outcomes

**Completed:** 2026-03-31
**Result:** Done — added Sub/Tier/Email/Org to `status_active` at v::1 (reads live
`.credentials.json` and `.claude.json`); added Sub/Tier to `status_named` at v::1
(from account struct, no extra I/O); 3 new passing tests; CLI docs and spec updated.
All 276 tests pass, clippy clean.
**Files changed:** `src/commands.rs`, `tests/integration/account_list_status_test.rs`
(astat11), `tests/integration/account_status_name_test.rs` (astname11+12),
`docs/cli/commands.md`, `docs/cli/testing/command/account_status.md`,
`spec.md` (conformance row FR-16)
