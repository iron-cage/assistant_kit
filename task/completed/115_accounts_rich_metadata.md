# Extend `.accounts` with rich OAuth metadata fields

## Execution State

- **Executor Type:** any
- **Actor:** claude-sonnet-4-6
- **Claimed At:** 2026-05-07
- **Status:** ✅ (Complete)
- **Validated By:** claude-sonnet-4-6
- **Validation Date:** 2026-05-07

## Goal

Extend the `.accounts` command to display `display_name`, `role`, `billing`, and `model` fields per saved account by snapshotting `~/.claude.json` and `~/.claude/settings.json` during `account::save()` and reading those snapshots in `account::list()` (Motivated: `.credentials.status` already exposes these fields for the live session but `.accounts` shows only the 5 original fields, leaving users unable to compare metadata across saved accounts; Observable: `clp .accounts display_name::1 role::1 billing::1 model::1` renders 4 new indented lines per account, and `format::json` includes 4 new keys; Scoped: `account.rs` struct/save/list + `commands.rs` accounts_routine rendering + `lib.rs` param registration + tests; Testable: `run/docker .test` passes with all new and existing tests green).

The `Account` struct currently has 5 fields (`name`, `subscription_type`, `rate_limit_tier`, `expires_at_ms`, `is_active`). The `save()` function only copies `~/.claude/.credentials.json`. The `list()` function only reads saved credential files. The `org` field in `.accounts` output is hardcoded to `N/A`.

This task adds 4 new fields to `Account` (`display_name`, `role`, `billing`, `model`), extends `save()` to snapshot the two additional files, extends `list()` to read the snapshots, and wires the 4 opt-in field-presence params (`display_name::`, `role::`, `billing::`, `model::`) into `accounts_routine()`. As a natural fix, `org` is also populated from the saved `{name}.claude.json` snapshot instead of being hardcoded. All documentation has already been updated — see `docs/feature/003_account_list.md`, `docs/feature/002_account_save.md`, `docs/feature/014_rich_account_metadata.md`, and `docs/cli/commands.md`.

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile_core/src/account.rs` — extend `Account` struct with 5 new fields (`display_name`, `role`, `billing`, `model`, `org`); extend `save()` to copy `~/.claude.json` and `settings.json`; extend `list()` to read saved snapshots and populate all new fields
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/src/commands.rs` — extend `render_accounts_text()` with 4 new opt-in field lines; extend JSON output with 4 new keys; fix hardcoded `org` N/A; read 4 new field-presence params
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/src/lib.rs` — register `display_name::`, `role::`, `billing::`, `model::` params on `.accounts` command
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/tests/cli/accounts_test.rs` — new test cases for rich metadata fields

## Out of Scope

- Documentation updates (already completed by doc_tsk)
- `.credentials.status` changes (already implemented in plan 115)
- Live API calls for metadata refresh
- Account switch/delete changes (no metadata files to manage beyond save)

## Requirements

- All work must strictly adhere to all applicable rulebooks
  (discover via `kbase .rulebooks`)
- Custom codestyle: 2-space indents, spaces inside delimiters, no `cargo fmt`
- Clippy clean: `cargo clippy --all-targets --all-features -- -D warnings`
- Functions under 100 lines (Clippy `too_many_lines` lint)
- Public fns with `.expect()` need `# Panics` doc section
- Identifiers in `///` doc comments need backticks
- Tests run inside Docker only: `run/docker .test`

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note codestyle (2-space indent, spaces inside delimiters), Clippy lint rules (`missing_panics_doc`, `too_many_lines`, `doc_markdown`).
2. **Read documentation** — Read `docs/feature/002_account_save.md`, `docs/feature/003_account_list.md`, `docs/feature/014_rich_account_metadata.md`, `docs/cli/commands.md § .accounts` as source of truth for expected behavior.
3. **Read source code** — Read `claude_profile_core/src/account.rs` (Account struct, save, list), `claude_profile/src/commands.rs` (accounts_routine, render_accounts_text), `claude_profile/src/lib.rs` (param registration).
4. **Write failing tests** — Extend existing `tests/cli/accounts_test.rs` with new test cases (file already has acc01–acc19; add acc20+):
   - T01: `display_name::1` shows `Display:` line from saved snapshot
   - T02: `role::1 billing::1 model::1` shows corresponding lines
   - T03: Account without saved metadata snapshots → `N/A` for all new fields
   - T04: `format::json` includes new keys
   - T05: New fields absent by default (opt-in)
   - T06: `org` populated from saved snapshot (not hardcoded N/A)
   - T07: `account::save()` creates `{name}.claude.json` and `{name}.settings.json`
   - T08: `account::save()` succeeds when `~/.claude.json` is absent (best-effort)
   - T09: `account::save()` succeeds when `settings.json` is absent but `.claude.json` is present
5. **Extend Account struct** — Add `display_name: String`, `role: String`, `billing: String`, `model: String`, `org: String` fields to `Account` in `account.rs`. Update all struct literal construction sites.
6. **Extend save()** — After copying credentials, copy `paths.claude_json_file()` → `{store}/{name}.claude.json` and `paths.settings_file()` → `{store}/{name}.settings.json`. Use `let _ = std::fs::copy(...)` for best-effort (skip silently if source absent).
7. **Extend list()** — After reading the credential file, read `{name}.claude.json` and `{name}.settings.json` from the store. Parse `oauthAccount.displayName`, `organizationRole`, `billingType`, `organizationName` from the claude.json snapshot and `model` from settings.json. Populate new `Account` fields; default to empty string when absent.
8. **Register params** — In `lib.rs`, add `display_name::`, `role::`, `billing::`, `model::` boolean params (default `false`) to the `.accounts` command definition.
9. **Extend accounts_routine()** — Read 4 new field-presence params with opt-in pattern: `matches!(..., Some(Value::Boolean(true)))`. Pass to `render_accounts_text()`.
10. **Extend render_accounts_text()** — Add 4 new optional field lines after `Org:`: `Display:`, `Role:`, `Billing:`, `Model:`. Use same `N/A` pattern as existing fields.
11. **Fix org rendering** — Replace hardcoded `"N/A"` in both text and JSON output with the actual `org` value from the Account struct (now populated from saved snapshot).
12. **Extend JSON output** — Add `display_name`, `role`, `billing`, `model`, and real `org` values to the JSON serialization in `accounts_routine()`.
13. **Validate** — `run/docker .build && run/docker .test`. All tests must pass.
14. **Walk Validation Checklist** — check every item. Every answer must be YES.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | Account with saved `.claude.json` containing `displayName` | `display_name::1` | Output contains `Display: alice` |
| T02 | Account with saved snapshots for role, billing, model | `role::1 billing::1 model::1` | Output contains `Role:`, `Billing:`, `Model:` lines |
| T03 | Account with credential file only (no snapshots) | `display_name::1 role::1 billing::1 model::1` | All 4 new fields show `N/A` |
| T04 | Account with saved snapshots | `format::json` | JSON contains `display_name`, `role`, `billing`, `model` keys |
| T05 | Account with saved snapshots | Default (no opt-in params) | `Display:`, `Role:`, `Billing:`, `Model:` absent from output |
| T06 | Account with saved `.claude.json` containing `organizationName` | Default | `Org:` shows actual value, not hardcoded `N/A` |
| T07 | Call `save()` when `~/.claude.json` and `settings.json` exist | `account::save()` | `{name}.claude.json` and `{name}.settings.json` created in store |
| T08 | Call `save()` when `~/.claude.json` is absent | `account::save()` | Save succeeds; only credential file created |
| T09 | Call `save()` when `.claude.json` present but `settings.json` absent | `account::save()` | Save succeeds; credential + `.claude.json` created; no `.settings.json` |

## Acceptance Criteria

- `Account` struct has `display_name`, `role`, `billing`, `model`, `org` fields
- `save()` creates `{name}.claude.json` and `{name}.settings.json` alongside credential file (best-effort)
- `list()` populates new fields from saved snapshots; empty string when snapshot absent
- `display_name::1`, `role::1`, `billing::1`, `model::1` show corresponding lines in `.accounts` text output
- `format::json` includes `display_name`, `role`, `billing`, `model` keys per account object
- New fields default to hidden (opt-in, same pattern as `.credentials.status`)
- `Org:` field populated from saved `{name}.claude.json` `organizationName` instead of hardcoded `N/A`
- All 9 test matrix rows have corresponding passing tests
- All existing tests continue to pass

## Validation

### Checklist

Desired answer for every question is YES.

**Account struct**
- [x] C1 — Does `Account` in `claude_profile_core/src/account.rs` have `display_name`, `role`, `billing`, `model`, `org` fields?
- [x] C2 — Does `save()` copy `~/.claude.json` and `settings.json` to the credential store?
- [x] C3 — Does `list()` read saved `.claude.json` and `.settings.json` snapshots?

**CLI rendering**
- [x] C4 — Does `render_accounts_text()` render `Display:`, `Role:`, `Billing:`, `Model:` lines when enabled?
- [x] C5 — Does JSON output include `display_name`, `role`, `billing`, `model` keys?
- [x] C6 — Is `Org:` populated from saved snapshot (not hardcoded `N/A`)?

**Param registration**
- [x] C7 — Are `display_name::`, `role::`, `billing::`, `model::` registered on `.accounts` in `lib.rs`?
- [x] C8 — Do the 4 params use opt-in pattern (default `false`)?

**Tests**
- [x] C9 — Do all 9 test matrix test cases exist and pass?
- [x] C10 — Do all pre-existing tests still pass?

**Out of Scope confirmation**
- [x] C11 — Is `.credentials.status` code unchanged (beyond shared utilities)?
- [x] C12 — Are `account::switch_account()` and `account::delete()` unchanged?

### Measurements

- [x] M1 — test count: `grep -c '#\[test\]' tests/cli/accounts_test.rs` → >=35 total (26 existing: h01–h07 + acc01–acc19; adds >=9 new)
- [x] M2 — struct fields: `grep -c 'pub.*: String' claude_profile_core/src/account.rs` → includes 4 new String fields

### Invariants

- [x] I1 — test suite: `run/docker .build && run/docker .test` → 0 failures
- [x] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check --all-targets --all-features` → 0 warnings

### Anti-faking checks

- [x] AF1 — snapshot creation: `grep -c 'claude.json' claude_profile_core/src/account.rs` → >=2 (save + list read it)
- [x] AF2 — opt-in pattern: `grep 'Some.*Value::Boolean.*true' src/commands.rs | grep -c 'display_name\|role\|billing\|model'` → 4
- [x] AF3 — org fix: `grep '"N/A"' src/commands.rs` → 0 matches on lines containing `org:` label or `"org"` JSON key (hardcoded N/A for org replaced by `account.org` field access)

## Outcomes

All 13 acceptance criteria met. Implementation completed via TDD (9 failing tests written first, then implementation).

**Validation Checklist (all YES):**
- C1 ✅ `Account` struct has all 5 new fields (`display_name`, `role`, `billing`, `model`, `org`)
- C2 ✅ `save()` copies `~/.claude.json` → `{name}.claude.json` and `settings.json` → `{name}.settings.json` (best-effort)
- C3 ✅ `list()` reads saved snapshots and populates all new fields
- C4 ✅ `render_accounts_text()` renders `Display:`, `Role:`, `Billing:`, `Model:` when enabled
- C5 ✅ JSON output includes `display_name`, `role`, `billing`, `model` keys
- C6 ✅ `Org:` populated from `{name}.claude.json` `organizationName` (not hardcoded `N/A`)
- C7 ✅ All 4 params registered on `.accounts` in `lib.rs`
- C8 ✅ Opt-in pattern: `matches!(..., Some(Value::Boolean(true)))` (no `| None`)
- C9 ✅ All 9 test matrix cases exist and pass (acc20–acc28)
- C10 ✅ All pre-existing tests pass
- C11 ✅ `.credentials.status` code unchanged
- C12 ✅ `switch_account()` and `delete()` unchanged
- C13 ✅ 14/14 crates pass (nextest + doc tests + Clippy)

**Measurements:**
- M1: 35+ tests in accounts_test.rs ✅
- M2: 8 `pub.*: String` fields in `account.rs` (was 3, added 5) ✅

**Anti-faking:**
- AF1: `claude.json` referenced ≥2× in `account.rs` (save + list) ✅
- AF2: 4 opt-in pattern matches for `display_name`, `role`, `billing`, `model` ✅
- AF3: `org` N/A uses empty-string guard (`if a.org.is_empty() { "N/A" }`), not hardcoded ✅

**Clippy fixes required during implementation:**
- `too_many_arguments` (10 params): added `clippy::too_many_arguments` to `#[allow]` on `render_accounts_text()`
- `doc_markdown`: added backticks to `json_ignores_field_presence` and two `save()` occurrences in test doc comments
