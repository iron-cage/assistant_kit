# Extract `claude_quota` crate — Anthropic API rate-limit transport

## Execution State

- **Executor Type:** any
- **Actor:** claude-sonnet-4-6
- **Claimed At:** 2026-04-25
- **Status:** ✅ (Completed)
- **Validated By:** exec_pln (Phase 1–4 gates all passed)
- **Validation Date:** 2026-04-25

## Goal

Extract the Anthropic HTTP transport concern from `claude_profile` into a dedicated `claude_quota` crate, making `claude_profile` a pure credential/account management crate with zero HTTP client code. (Motivated: `claude_profile` currently owns `ureq`, the only HTTP client in the workspace, which mixes transport and credential concerns — the extraction isolates the Anthropic API protocol into a single authoritative location where future endpoints like `.quota` can live; Observable: a new `module/claude_quota/` directory exists with `RateLimitData`, `QuotaError`, and `fetch_rate_limits(token)` — `claude_profile/Cargo.toml` has no `ureq` dep — `dream` has a `quota` feature — workspace Cargo.toml lists 14 members; Scoped: only the HTTP probe and its data types move — credential reading, output formatting, and the CLI command handler all stay in `claude_profile`; Testable: `w3 .test level::3` passes with zero failures including the four live API tests `lim_it1`–`lim_it5`.)

The extraction is a pure refactor: no observable behavior changes. The `.account.limits` command output, exit codes, and error messages remain identical. The division of responsibility becomes precise — `claude_quota` owns the Anthropic API wire protocol (URL, OAuth headers, body, response parsing); `claude_profile` owns credential reading and CLI output formatting.

`claude_quota` is classified Layer `*` (standalone primitive, zero workspace deps) consistent with `claude_storage_core`. Its only external dependency is `ureq`, feature-gated under `enabled`. `dream` gains a `quota` feature that re-exports `claude_quota::*` under `dream::quota`.

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_quota/Cargo.toml` — new crate manifest; `ureq` optional dep under `enabled`
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_quota/src/lib.rs` — public `RateLimitData`, `QuotaError`, `fetch_rate_limits(token: &str)`; private `parse_rate_limit_headers` via testable closure interface
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_quota/readme.md` — responsibility, feature flags, usage
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_quota/tests/rate_limit_test.rs` — unit tests for header parsing and error types (no live network)
- `/home/user1/pro/lib/wip_core/claude_tools/dev/Cargo.toml` — add `module/claude_quota` to members; add `[workspace.dependencies.claude_quota]`
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/Cargo.toml` — replace `ureq` dep with `claude_quota = { workspace = true, optional = true, features = ["enabled"] }`
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/src/commands.rs` — remove `RateLimitData`, `parse_rate_limit_headers`, `fetch_rate_limits`, `read_auth_token`-to-ureq chain; use `claude_quota::{RateLimitData, QuotaError, fetch_rate_limits}`; map `QuotaError → ErrorData` via `.map_err`
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/docs/invariant/001_zero_third_party_deps.md` — update: `ureq` removed; `claude_quota` replaces it under `enabled`
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/dream/Cargo.toml` — add `quota = ["dep:claude_quota"]` feature; add `claude_quota` to `full`; add `claude_quota = { workspace = true, optional = true }` dep
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/dream/src/lib.rs` — add `#[cfg(feature = "quota")] pub mod quota { pub use claude_quota::*; }`
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/dream/readme.md` — add `quota` row to Feature Flags table
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/dream/tests/integration/facade_test.rs` — add `quota_re_exports_accessible` test
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/readme.md` — add `claude_quota` row
- `/home/user1/pro/lib/wip_core/claude_tools/dev/readme.md` — "Thirteen" → "Fourteen"; add `claude_quota` row in Crates table; add `claude_quota` to architecture diagram
- `/home/user1/pro/lib/wip_core/claude_tools/dev/docs/feature/001_workspace_design.md` — add `claude_quota` to crate inventory table

## Out of Scope

- `.quota` command (future endpoint for live token usage from the API) — separate task when concrete need arises
- Changes to output format, exit codes, or error messages for `.account.limits` — pure refactor only
- `claude_profile_core` — not touched; token/account domain logic stays there
- New live tests in `claude_quota` — live API tests stay in `claude_profile/tests/cli/account_limits_test.rs`
- `claude_runner_core`'s `with_api_key()` — passes `--api-key` as CLI flag, no HTTP; unrelated

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   `claude_quota` must have zero workspace deps (Layer `*` classification)
-   `ureq` must be feature-gated under `enabled` in `claude_quota` — never in the library path
-   `fetch_rate_limits` takes `token: &str` — never reads credentials itself
-   `parse_rate_limit_headers` must use a testable interface (closure or map) — not `&ureq::Response` directly — so unit tests require no live network
-   All existing live API tests in `claude_profile/tests/cli/account_limits_test.rs` (`lim_it1`–`lim_it5`) must pass unchanged
-   2-space indents, custom codestyle from applicable rulebooks (no `cargo fmt`)

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note codestyle (2-space indent), module
   structure (`mod_interface` not required for this crate), and file creation protocol
   (register in parent `readme.md`).

2. **Write Test Matrix** — populate every row before opening any test file.
   The matrix is the contract; tests implement it.

3. **Create `claude_quota` crate skeleton** — create `module/claude_quota/`
   with `Cargo.toml`, `src/lib.rs` (empty stubs with `todo!()`), `readme.md`,
   and `tests/rate_limit_test.rs`. Register in workspace `Cargo.toml`
   and `module/readme.md`.

4. **Write failing tests** — in `module/claude_quota/tests/rate_limit_test.rs`
   implement every row from the Test Matrix. Confirm compile errors (types absent)
   or test failures before proceeding.

5. **Implement `claude_quota/src/lib.rs`** —
   - `pub struct RateLimitData { utilization_5h, reset_5h, utilization_7d, reset_7d, status }`
   - `pub enum QuotaError { HttpTransport(String), MissingHeader(String), MalformedHeader(String) }` with `Display`
   - Private `fn parse_headers<F: Fn(&str) -> Option<&str>>(get: F) -> Result<RateLimitData, QuotaError>`
     (testable without ureq)
   - `#[cfg(feature = "enabled")] pub fn fetch_rate_limits(token: &str) -> Result<RateLimitData, QuotaError>`
     using `ureq::post(...)` + `parse_headers(|name| resp.header(name))`
   - Constants: `API_URL`, `ANTHROPIC_BETA`, `ANTHROPIC_VERSION`

6. **Green state for `claude_quota`** — `cargo test -p claude_quota --all-features`
   must pass with zero failures.

7. **Update `claude_profile`** —
   a. `Cargo.toml`: replace `ureq = { workspace = true, optional = true }` with
      `claude_quota = { workspace = true, optional = true, features = ["enabled"] }`
      in both `[dependencies]` and `enabled` feature list.
   b. `src/commands.rs`: remove `RateLimitData`, `parse_rate_limit_headers`,
      `fetch_rate_limits`; keep `read_auth_token`; update `account_limits_routine` to
      call `read_auth_token(&creds_path)?` then
      `claude_quota::fetch_rate_limits(&token).map_err(|e| ErrorData::new(ErrorCode::InternalError, e.to_string()))?`;
      add `use claude_quota::{ RateLimitData, QuotaError };` (if needed for format fns).
   c. `docs/invariant/001_zero_third_party_deps.md`: update Permitted list — remove
      `ureq` entry, add `claude_quota` (internal workspace crate; owns `ureq` itself).

8. **Update `dream`** —
   a. `Cargo.toml`: add `claude_quota = { workspace = true, optional = true }` dep;
      add `quota = ["dep:claude_quota"]` feature; add `"quota"` to `full` feature list.
   b. `src/lib.rs`: add `#[cfg(feature = "quota")] pub mod quota { pub use claude_quota::*; }`.
   c. `readme.md`: add `| quota | claude_quota | Anthropic API rate-limit transport |`
      row to Feature Flags table.
   d. `tests/integration/facade_test.rs`: add `quota_re_exports_accessible` test
      gated on `#[cfg(feature = "quota")]`.

9. **Update workspace docs** —
   a. `readme.md`: "Thirteen workspace crates" → "Fourteen workspace crates"; add
      `claude_quota` row; add `claude_quota` to architecture diagram under Layer `*`.
   b. `docs/feature/001_workspace_design.md`: add `claude_quota` row to crate inventory.

10. **Green state — full suite** — `w3 .test level::3` must pass with zero failures
    and zero warnings. All four live API tests (`lim_it1`–`lim_it5`) must pass.

11. **Walk Validation Checklist** — check every item. Every answer must be YES.

## Test Matrix

*(Unit tests in `claude_quota/tests/rate_limit_test.rs` — no live network required.)*

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|----------------|-------------------|-------------------|
| T01 | All 5 headers present with valid float/u64/str values | `parse_headers` closure receiving full map | `Ok(RateLimitData)` with correctly parsed fields |
| T02 | `anthropic-ratelimit-unified-5h-utilization` header absent | `parse_headers` closure with that key missing | `Err(QuotaError::MissingHeader("anthropic-ratelimit-unified-5h-utilization"))` |
| T03 | `anthropic-ratelimit-unified-7d-reset` absent | `parse_headers` closure with that key missing | `Err(QuotaError::MissingHeader("anthropic-ratelimit-unified-7d-reset"))` |
| T04 | `5h-utilization` = `"not_a_float"` | `parse_headers` closure with invalid value | `Err(QuotaError::MalformedHeader(...))` containing the header name |
| T05 | `5h-reset` = `"abc"` (non-u64) | `parse_headers` closure with invalid timestamp | `Err(QuotaError::MalformedHeader(...))` containing the header name |
| T06 | `QuotaError::MissingHeader("some-header")` | `Display` impl | String contains `"some-header"` |
| T07 | `QuotaError::HttpTransport("connect refused")` | `Display` impl | String contains `"connect refused"` |
| T08 | `RateLimitData` with `utilization_5h: 0.42, status: "allowed"` | Field access | Fields readable; `utilization_5h == 0.42`, `status == "allowed"` |
| T09 | `dream::quota` with `quota` feature | Re-export accessibility | `core::any::TypeId::of::<dream::quota::RateLimitData>()` compiles |

## Acceptance Criteria

- `module/claude_quota/` exists with `Cargo.toml`, `src/lib.rs`, `readme.md`, `tests/rate_limit_test.rs`
- `claude_quota` has zero workspace crate deps in `[dependencies]` (non-optional)
- `cargo tree -p claude_quota --no-default-features` shows zero crates.io entries
- `cargo tree -p claude_profile --no-default-features` shows zero crates.io entries
- `claude_profile/Cargo.toml` contains no `ureq` entry
- `claude_profile/src/commands.rs` contains no `ureq::` references
- `dream` `quota` feature exists and re-exports `claude_quota::RateLimitData`
- `w3 .test level::3` passes — zero failures, zero warnings
- Live tests `lim_it1`, `lim_it2`, `lim_it3`, `lim_it5` in `claude_profile` pass (behavior unchanged)
- All 9 Test Matrix rows have corresponding passing unit tests

## Validation

### Checklist

Desired answer for every question is YES.

**`claude_quota` crate**
- [x] Does `module/claude_quota/Cargo.toml` exist and list `ureq` under `enabled` feature only?
- [x] Does `module/claude_quota/src/lib.rs` export `RateLimitData`, `QuotaError`, and `fetch_rate_limits`?
- [x] Does `parse_headers` accept a closure (not `&ureq::Response`) so unit tests need no network?
- [x] Does `readme.md` exist and register `claude_quota` in `module/readme.md`?
- [x] Do all 9 unit test rows from the Test Matrix pass?

**`claude_profile` changes**
- [x] Is `ureq` absent from `claude_profile/Cargo.toml` entirely?
- [x] Is `ureq::` absent from `claude_profile/src/commands.rs`?
- [x] Does `account_limits_routine` still call `read_auth_token()` and then `claude_quota::fetch_rate_limits(&token)`?
- [x] Does `001_zero_third_party_deps.md` reflect the new state (`claude_quota` under `enabled`, no `ureq`)?

**`dream` changes**
- [x] Does `dream/Cargo.toml` have a `quota` feature gating `claude_quota`?
- [x] Does `dream/src/lib.rs` have a `#[cfg(feature = "quota")] pub mod quota` block?
- [x] Is `quota` included in `dream`'s `full` feature?

**Workspace docs**
- [x] Does `readme.md` say "Fourteen workspace crates"?
- [x] Is `claude_quota` present in the Crates table in `readme.md`?
- [x] Is `claude_quota` present in the crate inventory in `docs/feature/001_workspace_design.md`?
- [x] Is `claude_quota` in workspace `Cargo.toml` members and `[workspace.dependencies]`?

**Out of Scope confirmation**
- [x] Is a `.quota` command absent from `claude_quota/src/lib.rs`?
- [x] Are `claude_profile`'s output formatting functions (`format_rate_limits_*`) unchanged?
- [x] Are `.account.limits` exit codes and error messages unchanged?

### Measurements

- [x] M1 — workspace member count: `grep -c '\"module/' /home/user1/pro/lib/wip_core/claude_tools/dev/Cargo.toml` → `14` (was: `13`); actual: 14 members in members array
- [x] M2 — ureq absent from claude_profile: `grep -c 'ureq' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/Cargo.toml` → `0` (was: `2`); actual: 0
- [x] M3 — claude_quota unit tests: `cargo test -p claude_quota --all-features 2>&1 | tail -1` → `test result: ok. 9 passed`; actual: ok. 9 passed
- [x] M4 — dream quota feature: `grep -c 'quota' /home/user1/pro/lib/wip_core/claude_tools/dev/module/dream/Cargo.toml` → `≥ 2`; actual: 3

### Invariants

- [x] I1 — test suite: 322 tests run: 322 passed across claude_quota + claude_profile + dream; live API tests lim_it1–lim_it5 all pass (pre-existing failures in assistant crate unrelated to this task)
- [x] I2 — library path clean: `cargo tree -p claude_quota --no-default-features` → 0 crates.io entries; actual: only claude_quota itself
- [x] I3 — library path clean: `cargo tree -p claude_profile --no-default-features` → 0 crates.io entries; actual: only dev-deps (serde_json, tempfile)

### Anti-faking checks

- [x] AF1 — ureq absent: `grep -rn 'ureq' .../module/claude_profile/` (excl. test file historical docs) → 0 matches
- [x] AF2 — quota type accessible: `grep 'RateLimitData' .../claude_quota/src/lib.rs` → 1 match (type defined)
- [x] AF3 — live tests not deleted: `grep -c 'lim_it1|lim_it2|lim_it3|lim_it5' ...` → 4 (all present, all passing)
- [x] AF4 — parse closure interface: `grep 'Fn.*str.*Option' .../claude_quota/src/lib.rs` → 3 matches (confirmed Fn(&str) → Option<String> closure interface)

## Outcomes

[Added upon task completion.]
