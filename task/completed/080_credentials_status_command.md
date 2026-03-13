# TSK-080 — Add `.credentials.status` command and differentiate live credentials from account store

## Status

✅ (Complete)

## Metadata

- **Value:** 7
- **Easiness:** 7
- **Priority:** 2
- **Safety:** 7
- **Advisability:** 686

## Goal

Add `.credentials.status` command to `claude_profile` that reads `~/.claude/.credentials.json`
directly — with no dependency on the account store (`_active` marker or `accounts/` dir).
Motivated by: `clp .account.status` fails with "no active account set" on fresh Claude Code
installations (no account management initialized), even though valid credentials exist.
Observable: `clp .credentials.status v::2` shows subscriptionType, rateLimitTier, email,
org, and token expiry on any authenticated Claude Code machine regardless of account store
state. Scoped to `claude_profile` only. Testable: `ctest3` green; integration tests cred01–cred05
pass on a temp HOME containing only `.credentials.json` with no `accounts/` directory.

**MOST criteria:**
- **Motivated:** `clp .account.status` is the only "current account" command but requires
  `_active` marker (account management initialized). Machines running Claude Code without
  `claude_profile` account setup have no way to inspect credentials via `clp`. This blocks
  diagnostics and tooling on fresh installations.
- **Observable:** `.credentials.status` succeeds and returns subscriptionType, rateLimitTier,
  token state, email, org without any `~/.claude/accounts/` setup.
- **Scoped:** `claude_profile` crate only — `src/commands.rs`, `src/lib.rs`,
  `unilang.commands.yaml`, `spec.md`, `docs/cli/commands.md`, new integration test file.
- **Testable:** `ctest3` green after implementation; cred01–cred05 all pass.

## Description

Two concerns are conflated in `.account.status`: it reads `_active` (account store concept)
to determine the account name, then reads `~/.claude/.credentials.json` (live credentials) for
metadata. If `_active` is absent the command errors unconditionally — even though the credential
data it would display is available and readable.

Root cause analysis: `status_active()` at `commands.rs:202-210` exits with `?` if `_active`
is absent, before ever reaching the credential-reading logic at lines 244–266.

Fix: introduce a separate command that reads credentials directly with no account store
dependency. The two domains remain cleanly separated:

| Domain | Source | Requires setup |
|---|---|---|
| Live credentials | `~/.claude/.credentials.json` + `.claude.json` | No |
| Account store | `~/.claude/accounts/` + `_active` | Yes |

Also improve `.account.status` error message to point users at the new command.

Related: root cause investigation in `src/-default_topic/-root_cause_analysis.md`.

## In Scope

- `src/commands.rs`:
  - Extract `read_live_cred_meta(paths, verbosity)` helper from `status_active()` (DRY)
  - Add `credentials_status_routine` handler for `.credentials.status`
  - Improve `status_active()` error message at line 209 to reference `.credentials.status`
- `src/lib.rs`:
  - Register `.credentials.status` in `register_commands()`
- `unilang.commands.yaml`:
  - Add `.credentials.status` command block with `v::`, `format::` params
- `spec.md`:
  - Add FR-17: `.credentials.status` requirement
  - Update Vocabulary: add "Live Credentials" term; refine "Active Account" definition
  - Update CLI command table: add `.credentials.status` row
  - Add conformance checklist row for FR-17
- `docs/cli/commands.md`:
  - Add `.credentials.status` documentation block
- `tests/integration/credentials_test.rs` (new file):
  - cred01–cred05 integration tests
- `tests/integration/readme.md`:
  - Add row for new test file

## Out of Scope

- Changes to `claude_profile_core` (no new core module needed; `parse_string_field` is
  already accessible via `crate::account::parse_string_field`)
- Changing `.account.status` behavior (backward compatible; only error message improves)
- Auto-detecting unmanaged state in `.account.status` and falling back (keep domains separate)
- `credentials.rs` module in `claude_profile_core` (premature; single use case)
- OAuth token refresh or authentication (network dep, forbidden)
- `.claude.json` full profile parsing beyond email/org (deferred)

## Requirements

- All work must strictly adhere to all applicable rulebooks
  (discover via `kbase .rulebooks`)
- No `cargo fmt` — 2-space indent, custom codestyle
- No process execution (`std::process::Command` forbidden in `claude_profile`)
- No third-party deps in library path; `serde_json`/`unilang`/`error_tools` only under `enabled` feature

## Test Matrix

| # | Input | Config Under Test | Expected |
|---|-------|-------------------|----------|
| cred01 | `clp .credentials.status` | temp HOME: `.credentials.json` only, no `accounts/` | Exit 0; shows subscriptionType + token state |
| cred02 | `clp .credentials.status v::2` | temp HOME: `.credentials.json` + `.claude.json` | Exit 0; shows sub, tier, email, org, expiry |
| cred03 | `clp .credentials.status format::json` | temp HOME: `.credentials.json` | Exit 0; valid JSON with `subscription`, `tier`, `token` fields |
| cred04 | `clp .credentials.status` | temp HOME: no `.credentials.json` | Exit non-zero; actionable error naming missing file |
| cred05 | `clp .credentials.status v::1` | temp HOME: `.credentials.json`, no `.claude.json` | Exit 0; shows `N/A` for email and org |

## Acceptance Criteria

- `.credentials.status` exits 0 on a temp HOME with only `.credentials.json` (no `accounts/`)
- `.credentials.status v::2` shows all fields: subscription, tier, email/org, expiry
- `.credentials.status format::json` returns parseable JSON
- `.credentials.status` exits non-zero with actionable error when `.credentials.json` absent
- `.account.status` error message references `.credentials.status`
- `read_live_cred_meta()` helper is used by both `status_active()` and `credentials_status_routine()`
- FR-17 present in `spec.md` with conformance row
- `.credentials.status` registered in `unilang.commands.yaml` and `lib.rs`
- All existing tests still pass; `ctest3` green; zero warnings

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. Read applicable rulebooks via `kbase .rulebooks`
2. Write Test Matrix (already above) — confirm all rows are specific and verifiable
3. **RED** — create `tests/integration/credentials_test.rs` with cred01–cred05; add row to
   `tests/integration/readme.md`; confirm tests fail (binary has no `.credentials.status`)
4. Add `.credentials.status` to `unilang.commands.yaml`
5. Register handler in `src/lib.rs`
6. Extract `read_live_cred_meta(paths: &ClaudePaths, verbosity: u8) -> (sub, tier, email, org)`
   from `status_active()` in `src/commands.rs`
7. Implement `credentials_status_routine` in `src/commands.rs` using `read_live_cred_meta`
8. Improve `status_active()` error message at line 209 to mention `.credentials.status`
9. **GREEN** — `w3 .test level::3` passes; all cred01–cred05 pass
10. Refactor if needed — `credentials_status_routine` under 50 lines; no duplication
11. Update `spec.md`: FR-17, Vocabulary, CLI table, conformance row
12. Update `docs/cli/commands.md`: add `.credentials.status` block
13. Walk Validation Checklist — every answer must be YES
14. Update task status → ✅ in `task/readme.md`

## Validation Checklist

Desired answer for every question is YES.

**`.credentials.status` command**
- [ ] Does `clp .credentials.status` succeed on a temp HOME with no `accounts/` dir?
- [ ] Does `clp .credentials.status v::2` output subscriptionType, rateLimitTier, email, org, expiry?
- [ ] Does `clp .credentials.status format::json` return valid parseable JSON?
- [ ] Does `clp .credentials.status` exit non-zero with actionable message when `.credentials.json` absent?
- [ ] Does `clp .credentials.status v::1` show `N/A` for email/org when `.claude.json` absent?

**Code quality**
- [ ] Is `read_live_cred_meta()` shared by both `status_active()` and `credentials_status_routine()`?
- [ ] Is `credentials_status_routine` under 50 lines?
- [ ] Does `status_active()` error message at line 209 reference `.credentials.status`?

**Registration**
- [ ] Is `.credentials.status` present in `unilang.commands.yaml`?
- [ ] Is `.credentials.status` registered in `src/lib.rs::register_commands()`?

**Spec and docs**
- [ ] Is FR-17 present in `spec.md` with the no-account-store requirement?
- [ ] Is "Live Credentials" in the `spec.md` Vocabulary section?
- [ ] Is `.credentials.status` in the CLI command table in `spec.md`?
- [ ] Is the conformance row for FR-17 added to `spec.md`?
- [ ] Is `.credentials.status` documented in `docs/cli/commands.md`?

**Out of scope confirmation**
- [ ] Is `claude_profile_core` unchanged?
- [ ] Does `.account.status` behavior remain backward compatible (same outputs, same error on missing `_active`)?
- [ ] Is there no auto-fallback in `.account.status` (domains stay separate)?

**Full suite**
- [ ] Does `ctest3` pass with zero failures and zero warnings?

## Validation Procedure

### Measurements

| # | Metric | Before | After | Command |
|---|--------|--------|-------|---------|
| M1 | Commands registered in `lib.rs` | 8 | 9 | `grep -c 'register\|routine' src/lib.rs` |
| M2 | `.credentials.status` entry in YAML | 0 | 1 | `grep -c '\.credentials\.status' unilang.commands.yaml` |
| M3 | `ctest3` result | passing | passing | `ctest3` in `claude_profile/` |
| M4 | Integration test files | N | N+1 | `ls tests/integration/*.rs \| wc -l` |

### Anti-faking Checks

- **AF1:** Run `cred01` binary test against a temp HOME with ONLY `.credentials.json` and no
  `accounts/` dir — confirms the command truly has no `_active` dependency, not just a path
  that happens to work when both exist.
- **AF2:** Verify `grep read_live_cred_meta src/commands.rs` appears in BOTH `status_active`
  and `credentials_status_routine` function bodies — confirms DRY refactor is real.
- **AF3:** Run old `.account.status` test (no `_active`) and verify error message now contains
  the string `.credentials.status` — confirms message improvement is not just a claimed change.

## Outcomes

**Completed:** 2026-04-04
**Result:** Done — implemented FR-17 via TDD with 5 passing integration tests (cred01–cred05); extracted `read_live_cred_meta()` helper shared by `status_active()` and `credentials_status_routine()`; improved `status_active()` error message to reference `.credentials.status`; registered command in `lib.rs`, `unilang.commands.yaml`; all 284 tests pass at level 3 (nextest + doc tests + clippy clean).
**Files changed:** `src/commands.rs`, `src/lib.rs`, `unilang.commands.yaml`, `tests/cli_integration_test.rs`, `tests/integration/credentials_test.rs` (new), `tests/integration/readme.md`, `docs/cli/commands.md`, `spec.md`
