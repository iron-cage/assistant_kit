# TSK-086: Implement `.account.limits` command

## Goal

Implement the `.account.limits` CLI command in `claude_profile` so that running
`clp .account.limits` shows plan and rate-limit utilization for the selected account
(session 5-hour window, weekly all-model, weekly Sonnet usage) with percentages and
reset times.

## Motivation

FR-18 is fully documented in `docs/feature/013_account_limits.md` and the CLI docs
(`docs/cli/commands.md` row 12, `docs/cli/testing/command/account_limits.md`).
This task converts that documentation into working code verified against all 9 IT cases.

## Current State

**Already done — do not re-implement:**

- `.account.limits` registered in `src/lib.rs:register_commands()` with `name::`, `v::`, `format::` args
- `.account.limits` defined in `unilang.commands.yaml` (currently `status: "planned"`)
- `account_limits_routine()` skeleton in `src/commands.rs` — account selection and all error-path routing implemented
- `require_active_credentials()` helper implemented
- `fetch_rate_limits()` stub present (`src/commands.rs` — always returns `Err`; replace with real HTTP implementation)
- IT-6–IT-9 error-path tests exist as `lim01`–`lim05` in `tests/integration/account_limits_test.rs` (all passing)

**Not yet done — this task:**

- `ureq` HTTP client not in workspace — hard blocker for happy path
- `RateLimitData` struct not defined
- `fetch_rate_limits()` is a stub — needs real POST `/v1/messages` + header extraction
- `account_limits_routine()` output formatting not implemented (`_opts` currently unused)
- IT-1–IT-5 happy-path tests not written

## Data Source

Rate-limit utilization comes exclusively from HTTP response headers on live API calls.
Headers confirmed from Claude Code binary analysis:

| Header | Value | Meaning |
|--------|-------|---------|
| `anthropic-ratelimit-unified-5h-utilization` | `0.0–1.0` | 5-hour session window consumed |
| `anthropic-ratelimit-unified-5h-reset` | Unix timestamp | Reset time for 5h window |
| `anthropic-ratelimit-unified-7d-utilization` | `0.0–1.0` | Weekly all-model consumed |
| `anthropic-ratelimit-unified-7d-reset` | Unix timestamp | Reset time for 7d window |
| `anthropic-ratelimit-unified-status` | `allowed` / `allowed_warning` / `rejected` | Rate-limit state |

Claude Code fetches these by making a lightweight `POST /v1/messages` with
`max_tokens: 1`, `messages: [{role: "user", content: "quota"}]`. Headers are
**never cached locally** — no local file contains them.

**HTTP client:** Use `ureq` (no async, minimal deps). Gate behind `enabled` feature.
Zero default-build deps.

**Responsibility test constraint:** `responsibility_no_process_execution_test.rs` greps
`src/` for `status()`, `spawn()`, `output()`, `.wait()`, `ExitStatus`. Do NOT call
`resp.status()` on a ureq response — use ureq's `Result` error destructuring
(`Err(ureq::Error::Status(code, resp))`) to access HTTP status instead.

## In Scope

- Add `ureq` to workspace `Cargo.toml` and `claude_profile/Cargo.toml` under `enabled` feature
- Define `RateLimitData` struct in `src/commands.rs` (5h utilization + reset, 7d utilization + reset, status string)
- Implement `fetch_rate_limits(creds_path: &Path) -> Result<RateLimitData, ErrorData>` — POST `/v1/messages`, extract `anthropic-ratelimit-unified-*` headers; read API key from credentials file
- Complete `account_limits_routine()` output formatting — use `opts.format` / `opts.verbosity` for `v::0` / `v::1` / `v::2` and `format::json`
- Write IT-1–IT-5 happy-path tests in `tests/integration/account_limits_test.rs`
- Update `readme.md` — fix "9 profile commands" → "10 profile commands" in YAML file description
- Update `unilang.commands.yaml` — change `.account.limits` status from `"planned"` to `"stable"`

## Out of Scope

- Changes to `docs/feature/013_account_limits.md` or CLI docs (already done)
- Changes to other commands
- Changes to `docs/invariant/`
- Re-implementing command registration (already in `src/lib.rs` and `unilang.commands.yaml`)
- Re-implementing error-path routing (already done in `account_limits_routine()`)
- Re-writing lim01–lim05 tests (already done; cover IT-6–IT-9)

## Test Matrix

| # | Input | Config Under Test | Expected | Status |
|---|-------|-------------------|----------|--------|
| IT-1 | `clp .account.limits` | Valid credentials, live API | Exit 0; shows session (5h), weekly all-model, weekly Sonnet utilization | write |
| IT-2 | `clp .account.limits v::0` | Valid credentials | Exit 0; compact: bare percentages, no reset times | write |
| IT-3 | `clp .account.limits v::2` | Valid credentials | Exit 0; verbose: all fields including raw values and reset times | write |
| IT-4 | `clp .account.limits format::json` | Valid credentials | Exit 0; valid JSON with utilization percentage fields | write |
| IT-5 | `clp .account.limits name::work` | Named account `work` exists | Exit 0; shows limits for `work` account | write |
| IT-6 | `clp .account.limits name::ghost` | Account `ghost` does not exist | Exit 2; stderr mentions "ghost" or "not found" | `lim01` ✅ |
| IT-7 | `clp .account.limits` | No active credentials configured | Exit 2; actionable error about authentication | `lim02` ✅ |
| IT-8 | `clp .account.limits` | Credentials present, data unavailable | Exit 2; actionable error, not silent zero | `lim03` ✅ |
| IT-9 | `clp .account.limits name::foo/bar` | Invalid chars in name | Exit 1; error mentions invalid/character | `lim04` ✅ |

## Acceptance Criteria

- **AC-01**: `.account.limits` shows session, weekly all-model, and weekly Sonnet utilization for the active account
- **AC-02**: `.account.limits name::work` shows limits for the named account
- **AC-03**: `format::json` returns structured JSON with utilization fields
- **AC-04**: Missing/unavailable data → exits 2 with actionable error — never silent zero

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)
- No `cargo fmt` — 2-space indent, custom codestyle
- No `std::process::Command` in `claude_profile/src/` (see `responsibility_no_process_execution_test.rs`)
- No `.status()` call on ureq response objects in `src/` — use ureq error destructuring
- `ureq` gated behind `enabled` feature; zero default-build deps
- TDD: write failing tests before implementing each feature area

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. Read applicable rulebooks via `kbase .rulebooks`
2. Add `ureq` to workspace `Cargo.toml` and `claude_profile/Cargo.toml` under `enabled` feature
3. Define `RateLimitData` struct in `src/commands.rs` (5h utilization + reset, 7d utilization + reset, status)
4. Implement `fetch_rate_limits(creds_path: &Path) -> Result<RateLimitData, ErrorData>` — POST `/v1/messages`, extract `anthropic-ratelimit-unified-*` headers; read API key from credentials file
5. **RED**: write IT-1–IT-5 happy-path tests; confirm they fail (fetch_rate_limits not yet connected to output)
6. Complete `account_limits_routine()` output formatting for `v::0` / `v::1` / `v::2` and `format::json`
7. **GREEN**: run `ctest3` — all 9 tests (IT-1–IT-5 new + lim01–lim05 existing) pass, zero warnings
8. Refactor if needed — `account_limits_routine` ≤50 lines; `fetch_rate_limits` ≤50 lines
9. Update `readme.md` — fix "9 profile commands" → "10 profile commands"
10. Update `unilang.commands.yaml` — change `.account.limits` status `"planned"` → `"stable"`
11. Walk Validation Checklist — every answer must be YES

## Validation Checklist

Desired answer for every question is YES.

**Happy Path**
- [ ] Does `clp .account.limits` exit 0 and show session (5h), weekly all-model, and weekly Sonnet utilization?
- [ ] Does `clp .account.limits v::0` show compact output with bare percentages only (no reset times)?
- [ ] Does `clp .account.limits v::2` show all fields including raw values and reset times?
- [ ] Does `clp .account.limits format::json` return valid parseable JSON with utilization fields?
- [ ] Does `clp .account.limits name::work` show limits for the named account?

**Error Handling**
- [ ] Does `clp .account.limits name::ghost` exit 2 mentioning "ghost" or "not found"?
- [ ] Does `clp .account.limits` exit 2 with actionable error when no credentials configured?
- [ ] Does `clp .account.limits` exit 2 (not silent 0) when data is unavailable?
- [ ] Does `clp .account.limits name::foo/bar` exit 1 for invalid name characters?

**Code Quality**
- [ ] Is `fetch_rate_limits()` ≤50 lines?
- [ ] Is `account_limits_routine()` ≤50 lines?
- [ ] Is `ureq` gated behind `enabled` feature (zero deps in default build)?
- [ ] Does `ctest3` pass with zero failures and zero warnings?
